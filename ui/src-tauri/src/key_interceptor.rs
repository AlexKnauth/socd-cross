use once_cell::sync::Lazy;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::ptr::null;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use vigem_client::Client;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{GetLastError, HWND};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExA, CreateWindowExW, DestroyWindow, HWND_MESSAGE, WS_DISABLED, WS_EX_NOACTIVATE,
};

use windows::Win32::{
    Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
        UnhookWindowsHookEx, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN,
        WM_KEYUP,
    },
};

static STICK_NEUTRAL: i16 = 0;
static STICK_LEFT: i16 = -29000;
static STICK_RIGHT: i16 = 29000;
static STICK_UP: i16 = -29000;
static STICK_DOWN: i16 = 29000;
static KEYBIND_LEFT_STICK_LEFT: usize = 0;
static KEYBIND_LEFT_STICK_RIGHT: usize = 1;
static KEYBIND_RIGHT_STICK_UP: usize = 2;

static KEYBINDS: Lazy<Arc<Mutex<Vec<u32>>>> = Lazy::new(|| Arc::new(Mutex::new(vec![])));
static mut KEY_HELD: [bool; 3] = [false; 3];

struct SharedState {
    target: Option<vigem_client::Xbox360Wired<Client>>,
    gamepad: Option<vigem_client::XGamepad>,
    hook_handle: Option<HHOOK>,
}

static SHARED_STATE: Lazy<Arc<Mutex<SharedState>>> = Lazy::new(|| {
    Arc::new(Mutex::new(SharedState {
        target: None,
        gamepad: None,
        hook_handle: None,
    }))
});

/// Wrapper around a HWND windows pointer that destroys the window on drop
struct HwndDropper(HWND);
unsafe impl Send for HwndDropper {}
impl Drop for HwndDropper {
    fn drop(&mut self) {
        if !(self.0 == HWND::default()) {
            let _ = unsafe { DestroyWindow(self.0) };
        }
    }
}

/// Try to create a hidden "message-only" window
///
fn create_hidden_window() -> Result<HwndDropper, ()> {
    let class_name = "Static\0".encode_utf16().collect::<Vec<u16>>();
    let window_name = "\0".encode_utf16().collect::<Vec<u16>>();

    let hwnd = unsafe {
        // Get the current module handle
        CreateWindowExW(
            WS_EX_NOACTIVATE,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_name.as_ptr()),
            WS_DISABLED,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            None,
            HINSTANCE::default(),
            None,
        )
    };
    if hwnd == HWND::default() {
        let error = unsafe { GetLastError() };
        println!("Failed to create hidden window. Error code: {:?}", error);
        Err(())
    } else {
        println!("Succeeded in creating hidden window");
        Ok(HwndDropper(hwnd))
    }
}

pub(crate) struct KeyInterceptor {
    should_run: Arc<AtomicBool>,
}

impl KeyInterceptor {
    pub fn new() -> Self {
        Self {
            should_run: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        // Connect to the ViGEmBus driver
        let client = vigem_client::Client::connect().map_err(|e| e.to_string())?;
        // Create the virtual controller target
        let id = vigem_client::TargetId::XBOX360_WIRED;
        let mut target = vigem_client::Xbox360Wired::new(client, id);

        // Plugin the virtual controller
        target.plugin().map_err(|e| e.to_string())?;
        // Wait for the virtual controller to be ready to accept updates
        target.wait_ready().map_err(|e| e.to_string())?;

        // Initialize the gamepad state
        let gamepad = vigem_client::XGamepad {
            buttons: vigem_client::XButtons!(UP | RIGHT | LB | A | X),
            ..Default::default()
        };

        // Set the hook and store the handle
        let hook = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(low_level_keyboard_proc_callback),
                HINSTANCE::default(),
                GetCurrentThreadId(),
            )
        }
        .map_err(|e| e.to_string())?;

        let mut shared_state = SHARED_STATE.lock().unwrap();
        shared_state.target = Some(target);
        shared_state.gamepad = Some(gamepad);
        shared_state.hook_handle = Some(hook);

        Ok(())
    }

    pub fn start(&mut self) -> Result<(), String> {
        // Read keybindings from file
        let path = Path::new("OverBind_conf.txt");
        let file = File::open(&path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let keybinds = reader
            .lines()
            .map(|line| {
                line.expect("Unable to read line")
                    .parse::<u32>()
                    .expect("Failed to parse keybinding")
            })
            .collect::<Vec<u32>>();
        *KEYBINDS.lock().unwrap() = keybinds;

        self.should_run.store(true, Ordering::SeqCst);

        // Message loop
        let mut message: MSG = unsafe { std::mem::zeroed() };
        while self.should_run.load(Ordering::SeqCst)
            && unsafe { GetMessageW(&mut message, None, 0, 0).into() }
        {
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }

        Ok(())
    }

    pub fn stop(&self) {
        self.should_run.store(false, Ordering::SeqCst);
    }
}

unsafe extern "system" fn low_level_keyboard_proc_callback(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    let keybinds = KEYBINDS.lock().unwrap();
    let mut shared_state = SHARED_STATE.lock().unwrap();
    let mut temp_target = shared_state.target.take();
    let mut temp_gamepad = shared_state.gamepad.take();
    let hook_handle = shared_state.hook_handle.unwrap();

    if n_code != HC_ACTION as i32 {
        return CallNextHookEx(HHOOK::default(), n_code, w_param, l_param);
    }

    let kbd_struct = l_param.0 as *const KBDLLHOOKSTRUCT;
    println!("Key {:?} {:?}", w_param, (*kbd_struct).vkCode);

    if w_param.0 as u32 == WM_KEYDOWN {
        for i in 0..2 {
            if (*kbd_struct).vkCode == keybinds[i] {
                KEY_HELD[i] = true;
            }
        }
    }

    if w_param.0 as u32 == WM_KEYDOWN {
        for i in 0..2 {
            if (*kbd_struct).vkCode == keybinds[i] {
                KEY_HELD[i] = true;
            }
        }
    }

    let mut left_stick_x: i16 = STICK_NEUTRAL;
    let mut left_stick_y: i16 = STICK_NEUTRAL;
    let mut right_stick_x: i16 = STICK_NEUTRAL;
    let mut right_stick_y: i16 = STICK_NEUTRAL;

    if KEY_HELD[KEYBIND_LEFT_STICK_RIGHT] {
        left_stick_x = STICK_RIGHT;
    } else if KEY_HELD[KEYBIND_LEFT_STICK_LEFT] {
        left_stick_x = STICK_LEFT;
    } else {
        left_stick_x = STICK_NEUTRAL;
    }

    if KEY_HELD[KEYBIND_RIGHT_STICK_UP] {
        right_stick_y = STICK_UP;
    } else {
        right_stick_y = STICK_NEUTRAL;
    }

    if let (Some(ref mut target), Some(ref mut gamepad)) = (&mut temp_target, &mut temp_gamepad) {
        gamepad.thumb_lx = left_stick_x;
        gamepad.thumb_ly = left_stick_y;
        gamepad.thumb_rx = right_stick_x;
        gamepad.thumb_ry = right_stick_y;
        let _ = target.update(&gamepad);

        shared_state.target = temp_target;
        shared_state.gamepad = temp_gamepad;
    }

    return CallNextHookEx(hook_handle, n_code, w_param, l_param);
}
