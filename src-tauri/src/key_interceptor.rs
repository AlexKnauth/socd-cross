use once_cell::sync::Lazy;
use rdev::{listen, simulate, Event, EventType, Key};
use serde::Deserialize;
use std::cmp;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::mem::replace;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use crate::{get_config_path, Settings};

#[derive(Debug, Deserialize)]
struct ConfigJsonData {
    keycode: String,
    result_type: String,
    result_value: i32,
}

struct KeyState {
    is_pressed: bool,
    result_type: String,
    result_value: i32,
}

#[derive(Clone)]
struct OppositeKey {
    is_pressed: bool,
    is_virtual_pressed: bool,
    opposite_key_type: String,
    opposite_key_value: u32,
    opposite_key_mapping: Option<u16>,
}

static KEY_STATES: Lazy<Arc<RwLock<HashMap<u32, KeyState>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

static OPPOSITE_KEY_STATES: Lazy<Arc<RwLock<HashMap<u32, OppositeKey>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

struct SharedState {
    is_running: bool,
    allowed_programs: Option<Vec<String>>,
}

static SHARED_STATE: Lazy<Arc<RwLock<SharedState>>> = Lazy::new(|| {
    Arc::new(RwLock::new(SharedState {
        is_running: false,
        allowed_programs: None,
    }))
});

pub(crate) struct KeyInterceptor {
    pub should_run: Arc<AtomicBool>,
}

struct OpposedKeysState {
    left: OpposedKeyState,
    right: OpposedKeyState,
    up: OpposedKeyState,
    down: OpposedKeyState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OpposedKeyState {
    Released,
    OpposedSeen,
    OpposedFresh,
    Pressed,
}

impl KeyInterceptor {
    pub fn new() -> Self {
        Self {
            should_run: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn initialize(&mut self, settings: &Settings) -> Result<(), String> {
        std::thread::spawn(|| {
            let state = RwLock::new(OpposedKeysState {
                left: OpposedKeyState::Released,
                right: OpposedKeyState::Released,
                up: OpposedKeyState::Released,
                down: OpposedKeyState::Released,
            });
            let listen_callback = move |event: Event| {
                match event.event_type {
                    EventType::KeyPress(key) => {
                        // println!("{:?}", event.event_type);
                        let mut s = state.write().unwrap();
                        match key {
                            Key::KeyA => {
                                let prev = replace(&mut s.left, OpposedKeyState::Pressed);
                                if let OpposedKeyState::Pressed = prev {
                                    return;
                                }
                                match s.right {
                                    OpposedKeyState::Pressed => {
                                        println!("user press: A, simulate release: E");
                                        s.right = OpposedKeyState::OpposedFresh;
                                        drop(s);
                                        simulate(&EventType::KeyRelease(Key::KeyE)).ok();
                                    }
                                    OpposedKeyState::Released => {
                                        if let OpposedKeyState::Released = prev {
                                            println!("user press: A, fresh");
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            Key::KeyE => {
                                let prev = replace(&mut s.right, OpposedKeyState::Pressed);
                                if let OpposedKeyState::Pressed = prev {
                                    return;
                                }
                                match s.left {
                                    OpposedKeyState::Pressed => {
                                        println!("user press: E, simulate release: A");
                                        s.left = OpposedKeyState::OpposedFresh;
                                        drop(s);
                                        simulate(&EventType::KeyRelease(Key::KeyA)).ok();
                                    }
                                    OpposedKeyState::Released => {
                                        if let OpposedKeyState::Released = prev {
                                            println!("user press E, fresh");
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            Key::Comma => {
                                let prev = replace(&mut s.up, OpposedKeyState::Pressed);
                                if let OpposedKeyState::Pressed = prev {
                                    return;
                                }
                                match s.down {
                                    OpposedKeyState::Pressed => {
                                        println!("user press: Comma, simulate release: O");
                                        s.down = OpposedKeyState::OpposedFresh;
                                        drop(s);
                                        simulate(&EventType::KeyRelease(Key::KeyO)).ok();
                                    }
                                    OpposedKeyState::Released => {
                                        if let OpposedKeyState::Released = prev {
                                            println!("user press Comma, fresh");
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            Key::KeyO => {
                                let prev = replace(&mut s.down, OpposedKeyState::Pressed);
                                if let OpposedKeyState::Pressed = prev {
                                    return;
                                }
                                match s.up {
                                    OpposedKeyState::Pressed => {
                                        println!("user press: O, simulate release: E");
                                        s.up = OpposedKeyState::OpposedFresh;
                                        drop(s);
                                        simulate(&EventType::KeyRelease(Key::Comma)).ok();
                                    }
                                    OpposedKeyState::Released => {
                                        if let OpposedKeyState::Released = prev {
                                            println!("user press O, fresh");
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            _ => (),
                        }
                    }
                    EventType::KeyRelease(key) => {
                        // println!("{:?}", event.event_type);
                        let mut s = state.write().unwrap();
                        match key {
                            Key::KeyA => {
                                match s.left {
                                    OpposedKeyState::Released => (),
                                    OpposedKeyState::OpposedSeen => s.left = OpposedKeyState::Released,
                                    OpposedKeyState::OpposedFresh => s.left = OpposedKeyState::OpposedSeen,
                                    OpposedKeyState::Pressed => s.left = OpposedKeyState::Released,
                                }
                                match s.right {
                                    OpposedKeyState::Released => (),
                                    OpposedKeyState::Pressed => (),
                                    _ => {
                                        println!("user release: A, simulate press: E");
                                        s.right = OpposedKeyState::Pressed;
                                        drop(s);
                                        simulate(&EventType::KeyPress(Key::KeyE)).ok();
                                    }
                                }
                            }
                            Key::KeyE => {
                                match s.right {
                                    OpposedKeyState::Released => (),
                                    OpposedKeyState::OpposedSeen => s.right = OpposedKeyState::Released,
                                    OpposedKeyState::OpposedFresh => s.right = OpposedKeyState::OpposedSeen,
                                    OpposedKeyState::Pressed => s.right = OpposedKeyState::Released,
                                }
                                match s.left {
                                    OpposedKeyState::Released => (),
                                    OpposedKeyState::Pressed => (),
                                    _ => {
                                        println!("user release: E, simulate press: A");
                                        s.left = OpposedKeyState::Pressed;
                                        drop(s);
                                        simulate(&EventType::KeyPress(Key::KeyA)).ok();
                                    }
                                }
                            }
                            Key::Comma => {
                                match s.up {
                                    OpposedKeyState::Released => (),
                                    OpposedKeyState::OpposedSeen => s.up = OpposedKeyState::Released,
                                    OpposedKeyState::OpposedFresh => s.up = OpposedKeyState::OpposedSeen,
                                    OpposedKeyState::Pressed => s.up = OpposedKeyState::Released,
                                }
                                match s.down {
                                    OpposedKeyState::Released => (),
                                    OpposedKeyState::Pressed => (),
                                    _ => {
                                        println!("user release: Comma, simulate press: O");
                                        s.down = OpposedKeyState::Pressed;
                                        drop(s);
                                        simulate(&EventType::KeyPress(Key::KeyO)).ok();
                                    }
                                }
                            }
                            Key::KeyO => {
                                match s.down {
                                    OpposedKeyState::Released => (),
                                    OpposedKeyState::OpposedSeen => s.down = OpposedKeyState::Released,
                                    OpposedKeyState::OpposedFresh => s.down = OpposedKeyState::OpposedSeen,
                                    OpposedKeyState::Pressed => s.down = OpposedKeyState::Released,
                                }
                                match s.up {
                                    OpposedKeyState::Released => (),
                                    OpposedKeyState::Pressed => (),
                                    _ => {
                                        println!("user release: O, simulate press: Comma");
                                        s.up = OpposedKeyState::Pressed;
                                        drop(s);
                                        simulate(&EventType::KeyPress(Key::Comma)).ok();
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
            };
            // This will block.
            if let Err(error) = listen(listen_callback) {
                println!("Error: {:?}", error)
            }
        });

        let mut shared_state = SHARED_STATE.write().unwrap();
        if !settings.allowed_programs.is_empty() {
            println!("Allowed programs: {:?}", settings.allowed_programs);
            shared_state.allowed_programs = Some(settings.allowed_programs.clone());
        }

        Ok(())
    }

    pub fn start(&mut self) -> Result<(), String> {
        // Read keybindings from file
        let path = get_config_path()?;
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| e.to_string())?;

        let data: Vec<ConfigJsonData> =
            serde_json::from_str(&contents).map_err(|e| e.to_string())?;
        let mut key_states = HashMap::new();
        let mut opposite_key_states = HashMap::new();

        for item in &data {
            let keycode =
                u32::from_str_radix(&item.keycode, 16).expect("Invalid hexadecimal string");

            if item.result_type != "socd" {
                let key_state = key_states.entry(keycode).or_insert_with(|| KeyState {
                    is_pressed: false,
                    result_type: item.result_type.clone(),
                    result_value: item.result_value,
                });
                println!(
                    "Keycode: {:?}, ResultType: {:?}, ResultValue {:?}",
                    keycode, key_state.result_type, key_state.result_value
                );
            }
        }

        for item in &data {
            let keycode = u32::from_str_radix(&item.keycode, 16);

            if item.result_type == "socd" {
                let opposite_keycode = item.result_value as u32;

                // Check if key_state has a value for the opposite keycode and if so then use that value instead
                let key_state = key_states.get(&keycode.clone().unwrap());
                let mut key_type = String::from("keyboard");
                let mut key_mapping = None;

                if key_state.is_some()
                    && (key_state.unwrap().result_type == "keyboard"
                        || key_state.unwrap().result_type == "face_button")
                {
                    key_type = key_state.unwrap().result_type.clone();
                    key_mapping = Some(key_state.unwrap().result_value as u16);
                }

                let opposite_key_state = opposite_key_states
                    .entry(keycode.clone().unwrap())
                    .or_insert_with(|| OppositeKey {
                        is_pressed: false,
                        is_virtual_pressed: false,
                        opposite_key_value: opposite_keycode,
                        opposite_key_type: key_type,
                        opposite_key_mapping: key_mapping,
                    });

                println!(
                    "Keycode: {:?}, OppositeKeycode: {:?}, OppositeKeyMapping: {:?}",
                    keycode,
                    opposite_key_state.opposite_key_value,
                    opposite_key_state.opposite_key_mapping
                );
            }
        }

        *KEY_STATES.write().unwrap() = key_states;
        *OPPOSITE_KEY_STATES.write().unwrap() = opposite_key_states;

        self.should_run.store(true, Ordering::SeqCst);

        let allowed_programs = SHARED_STATE.read().unwrap().allowed_programs.clone();
        if allowed_programs.is_some() {
            let mut shared_state = SHARED_STATE.write().unwrap();
            shared_state.is_running = true;
        } else {
            let mut shared_state = SHARED_STATE.write().unwrap();
            shared_state.is_running = true;
        }

        Ok(())
    }

    pub fn stop(&self) {
        let mut shared_state = SHARED_STATE.write().unwrap();
        shared_state.is_running = false;
    }

    pub fn is_running(&self) -> bool {
        let shared_state = SHARED_STATE.read().unwrap();
        shared_state.is_running
    }
}

fn is_extended_key(virtual_keycode: u32) -> bool {
    let extended_keys: [u32; 14] = [
        0x21, //page up
        0x22, //page down
        0x23, //end
        0x24, //home
        0x25, //left arrow
        0x26, //up arrow
        0x27, //right arrow
        0x28, //down arrow
        0x2C, //print screen
        0x2D, //insert
        0x2E, //delete
        0x90, //numlock
        0xA3, //right CTRL
        0xA5, //right ALT
    ];
    extended_keys.contains(&virtual_keycode)
}

// Used for when multiple stick rebinds are set, first prioritizes right/up(positive) over left/down(negative), then larger values within those bands
// neutral(zero) is the absolute lowest priority
fn analog_priority_transform(n: i32) -> i32 {
    if n < 0 {
        return -1 * n;
    }
    if n > 0 {
        return n + i16::MAX as i32;
    }

    0
}

fn find_higher_priority(num1: i16, num2: i16) -> i16 {
    if num1 == 0 {
        return num2;
    }
    if num2 == 0 {
        return num1;
    }

    if analog_priority_transform(num1 as i32) >= analog_priority_transform(num2 as i32) {
        return num1;
    } else {
        return num2;
    }
}
