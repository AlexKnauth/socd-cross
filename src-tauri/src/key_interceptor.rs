use once_cell::sync::Lazy;
use rdev::{listen, simulate, Event, EventType, Key};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use crate::{get_config_path, Settings};

#[derive(Debug, Deserialize)]
struct ConfigJsonData {
    keycode: String,
    result_type: String,
    result_value: String,
}

struct KeyState {
    is_pressed: bool,
    result_type: String,
    result_value: Key,
}

#[derive(Clone)]
struct OppositeKey {
    is_pressed: bool,
    is_virtual_pressed: bool,
    is_virtual_fresh: bool,
    opposite_key_type: String,
    opposite_key_value: Key,
    opposite_key_mapping: Option<Key>,
}

static KEY_STATES: Lazy<Arc<RwLock<HashMap<Key, KeyState>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

static OPPOSITE_KEY_STATES: Lazy<Arc<RwLock<HashMap<Key, OppositeKey>>>> =
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

impl KeyInterceptor {
    pub fn new() -> Self {
        Self {
            should_run: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn initialize(&mut self, settings: &Settings) -> Result<(), String> {
        std::thread::spawn(|| {
            #[cfg(target_os = "macos")]
            rdev::set_is_main_thread(false);
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
            let keycode: Key = serde_from_json_str(&item.keycode).expect("Invalid key string");

            if item.result_type != "socd" {
                let key_state = key_states.entry(keycode).or_insert_with(|| KeyState {
                    is_pressed: false,
                    result_type: item.result_type.clone(),
                    result_value: serde_from_json_str(&item.result_value).expect("Invalid key string"),
                });
                println!(
                    "Keycode: {:?}, ResultType: {:?}, ResultValue {:?}",
                    keycode, key_state.result_type, key_state.result_value
                );
            }
        }

        for item in &data {
            let keycode: Key = serde_from_json_str(&item.keycode).unwrap();

            if item.result_type == "socd" {
                let opposite_keycode: Key = serde_from_json_str(&item.result_value).expect("Invalid key string");

                // Check if key_state has a value for the opposite keycode and if so then use that value instead
                let key_state = key_states.get(&keycode);
                let mut key_type = String::from("keyboard");
                let mut key_mapping = None;

                if key_state.is_some()
                    && (key_state.unwrap().result_type == "keyboard"
                        || key_state.unwrap().result_type == "face_button")
                {
                    key_type = key_state.unwrap().result_type.clone();
                    key_mapping = Some(key_state.unwrap().result_value.clone());
                }

                let opposite_key_state = opposite_key_states
                    .entry(keycode.clone())
                    .or_insert_with(|| OppositeKey {
                        is_pressed: false,
                        is_virtual_pressed: false,
                        is_virtual_fresh: false,
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

fn listen_callback(event: Event) {
    let (key, key_is_down) = match event.event_type {
        EventType::KeyPress(key) => (key, true),
        EventType::KeyRelease(key) => (key, false),
        _ => return,
    };
    
    // Update Key State
    {
        let mut key_states = KEY_STATES.write().unwrap();
        match key_states.get_mut(&key) {
            Some(state) => state.is_pressed = key_is_down,
            _ => (),
        }
    }

    // Keyboard Rebinds
    {
        let key_states = KEY_STATES.read().unwrap();
        if let Some(key_state) = key_states.get(&key) {
            if key_state.result_type == "keyboard" {
                let result_value = key_state.result_value;
                drop(key_states);
                if key_is_down {
                    simulate(&EventType::KeyPress(result_value)).ok();
                } else {
                    simulate(&EventType::KeyRelease(result_value)).ok();
                }
                return;
            }
        }
    }

    // SOCD
    {
        let mut opposite_key_states = OPPOSITE_KEY_STATES.write().unwrap();

        let cloned_key_state;
        if opposite_key_states.contains_key(&key) {
            {
                let key_state = opposite_key_states.get_mut(&key).unwrap();
                if key_state.is_virtual_fresh {
                    if key_is_down == key_state.is_virtual_pressed {
                        key_state.is_virtual_fresh = false;
                    }
                    return;
                }
                // if key_is_down != key_state.is_virtual_pressed {
                //     println!("{:?} {}", key, key_is_down);
                // }
                key_state.is_pressed = key_is_down;
                key_state.is_virtual_pressed = key_is_down;

                cloned_key_state = key_state.clone();
            }

            let opposite_key_state = opposite_key_states
                .get_mut(&cloned_key_state.opposite_key_value)
                .unwrap();

            if key_is_down && opposite_key_state.is_pressed && opposite_key_state.is_virtual_pressed
            {
                opposite_key_state.is_virtual_pressed = false;
                opposite_key_state.is_virtual_fresh = true;

                {
                    let opposite_key_value = cloned_key_state.opposite_key_value;
                    drop(opposite_key_states);
                    // println!(" -> {:?} false", opposite_key_value);
                    simulate(&EventType::KeyRelease(opposite_key_value)).ok();
                    return;
                }
            } else if !key_is_down && opposite_key_state.is_pressed {
                opposite_key_state.is_virtual_pressed = true;
                opposite_key_state.is_virtual_fresh = true;

                {
                    let opposite_key_value = cloned_key_state.opposite_key_value;
                    drop(opposite_key_states);
                    // println!(" -> {:?} true", opposite_key_value);
                    simulate(&EventType::KeyPress(opposite_key_value)).ok();
                    return;
                }
            }
        }
    }
}

fn serde_from_json_str<T: DeserializeOwned>(s: &str) -> serde_json::Result<T> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
}
