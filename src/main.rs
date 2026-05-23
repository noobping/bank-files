#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#![deny(unused, dead_code, unreachable_code)]

#[cfg(all(feature = "flatpak", feature = "setup"))]
compile_error!("The flatpak feature is an offline build mode and cannot be combined with setup.");

mod analytics;
mod app;
mod app_info;
mod csv_detect;
mod data;
mod i18n;
mod local_ai;
mod model;
mod resources;
mod rules;
#[cfg(target_os = "linux")]
mod search_provider;
#[cfg(all(target_os = "linux", feature = "setup", not(feature = "flatpak")))]
mod setup;
mod summary;
mod ui;
mod updater;
mod util;

fn main() {
    let args = std::env::args_os().collect::<Vec<_>>();
    if let Some(code) = updater::handle_special_command(&args) {
        std::process::exit(code);
    }

    #[cfg(target_os = "linux")]
    if search_provider::is_search_provider_command(&args) {
        std::process::exit(search_provider::run());
    }

    app::run();
}
