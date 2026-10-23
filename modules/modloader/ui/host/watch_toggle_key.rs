use super::read_window_focus::{foreground_is_shroudforge_window, foreground_process};
use super::window::UiEvent;
use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};
use tao::event_loop::EventLoopProxy;
use windows_sys::Win32::System::Threading::GetCurrentProcessId;

pub(crate) fn watch_toggle_key(
    hotkey_proxy: EventLoopProxy<UiEvent>,
    hotkey_key_watcher: Arc<RwLock<Option<shroudforge_hotkeys::Hotkey>>>,
    game_pid: u32,
    desktop: bool,
) {
    thread::spawn(move || {
        let mut was_down = false;
        let mut previous_binding = None;
        loop {
            let binding = hotkey_key_watcher
                .read()
                .ok()
                .and_then(|binding| binding.clone());
            let down = binding
                .as_ref()
                .is_some_and(shroudforge_hotkeys::is_hotkey_down);
            if binding != previous_binding {
                was_down = down;
                previous_binding = binding;
            }
            if down && !was_down && !shroudforge_hotkeys::is_capture_active() {
                let foreground = foreground_process();
                let focused = !desktop
                    && (foreground == game_pid
                        || foreground == unsafe { GetCurrentProcessId() }
                        || foreground_is_shroudforge_window());
                if focused && hotkey_proxy.send_event(UiEvent::Toggle).is_err() {
                    break;
                }
            }
            was_down = down;
            thread::sleep(Duration::from_millis(5));
        }
    });
}
