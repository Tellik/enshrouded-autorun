use super::read_window_focus::{foreground_is_shroudforge_window, foreground_process};
use super::window::UiEvent;
use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    thread,
    time::Duration,
};
use tao::event_loop::EventLoopProxy;
use windows_sys::Win32::{
    System::Threading::GetCurrentProcessId,
    UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_F9},
};

pub(crate) fn watch_toggle_key(
    hotkey_proxy: EventLoopProxy<UiEvent>,
    hotkey_key_watcher: Arc<AtomicU32>,
    game_pid: u32,
    desktop: bool,
) {
    thread::spawn(move || {
        let mut was_down = false;
        loop {
            let down = unsafe {
                GetAsyncKeyState(hotkey_key_watcher.load(Ordering::Relaxed) as i32)
                    | GetAsyncKeyState(VK_F9 as i32)
            } < 0;
            if down && !was_down {
                let foreground = foreground_process();
                let focused = desktop
                    || foreground == game_pid
                    || foreground == unsafe { GetCurrentProcessId() }
                    || foreground_is_shroudforge_window();
                if focused && hotkey_proxy.send_event(UiEvent::Toggle).is_err() {
                    break;
                }
            }
            was_down = down;
            thread::sleep(Duration::from_millis(5));
        }
    });
}
