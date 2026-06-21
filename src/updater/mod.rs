#[cfg(not(any(
    target_os = "windows",
    all(target_os = "linux", feature = "setup", not(feature = "flatpak"))
)))]
mod disabled;
#[cfg(any(
    target_os = "windows",
    all(target_os = "linux", feature = "setup", not(feature = "flatpak"))
))]
mod logic;
#[cfg(any(
    target_os = "windows",
    all(target_os = "linux", feature = "setup", not(feature = "flatpak"))
))]
mod standalone;

#[cfg(not(any(
    target_os = "windows",
    all(target_os = "linux", feature = "setup", not(feature = "flatpak"))
)))]
pub use disabled::{
    after_window_presented, handle_special_command, register_app_actions, shutdown,
    supports_update_checks,
};
#[cfg(any(
    target_os = "windows",
    all(target_os = "linux", feature = "setup", not(feature = "flatpak"))
))]
pub use standalone::{
    after_window_presented, handle_special_command, register_app_actions, shutdown,
    supports_update_checks,
};
