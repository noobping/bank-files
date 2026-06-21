use adw::{Application, ApplicationWindow};
use std::ffi::OsString;

pub fn register_app_actions(_app: &Application) {}

pub const fn supports_update_checks() -> bool {
    false
}

pub fn after_window_presented(_app: &Application, _window: &ApplicationWindow) {}

pub fn shutdown(_app: &Application) {}

pub fn handle_special_command(_args: &[OsString]) -> Option<i32> {
    None
}
