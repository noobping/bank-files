use super::*;

pub(in crate::updater::standalone) fn platform_update_checks_enabled() -> bool {
    #[cfg(target_os = "linux")]
    {
        setup::is_installed_locally() && setup::is_current_executable_installed_locally()
    }

    #[cfg(not(target_os = "linux"))]
    {
        true
    }
}

#[cfg(target_os = "linux")]
pub(in crate::updater::standalone) fn auto_install_cleanup_dir(
    args: &[OsString],
) -> Option<PathBuf> {
    if args.get(1).is_none_or(|arg| arg != AUTO_INSTALL_ARG) {
        return None;
    }

    args.get(2).map(PathBuf::from)
}

#[cfg(target_os = "linux")]
pub(in crate::updater::standalone) fn run_auto_install(cleanup_dir: &Path) -> i32 {
    crate::i18n::init();

    if let Err(error) = crate::resources::register() {
        eprintln!("Resource registration failed before Linux update install: {error}");
    }

    match auto_install_update(cleanup_dir) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("Linux update install failed: {error}");
            show_auto_install_error_dialog(&error);
            1
        }
    }
}

#[cfg(target_os = "linux")]
fn auto_install_update(cleanup_dir: &Path) -> Result<(), String> {
    let current_exe = std::env::current_exe()
        .map_err(|error| format!("Failed to resolve the update helper executable: {error}"))?;
    let Some(parent) = current_exe.parent() else {
        return Err("The update helper executable has no parent directory.".to_string());
    };
    if parent != cleanup_dir {
        return Err(
            "The update helper cleanup directory does not match the downloaded update location."
                .to_string(),
        );
    }

    setup::install_locally()
        .map_err(|error| format!("Failed to install the downloaded update: {error}"))?;
    if let Err(error) = fs::remove_dir_all(cleanup_dir) {
        eprintln!(
            "Update cleanup failed for '{}': {error}",
            cleanup_dir.display()
        );
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn show_auto_install_error_dialog(error: &str) {
    if adw::init().is_err() {
        return;
    }

    let body = format!(
        "{}\n\n{}",
        gettext("The downloaded Linux update couldn't be installed."),
        error
    );
    let dialog = AlertDialog::new(Some(&gettext("Couldn't install the update.")), Some(&body));
    dialog.add_response("close", &gettext("Close"));
    dialog.set_close_response("close");
    dialog.set_default_response(Some("close"));

    let loop_ = glib::MainLoop::new(None, false);
    let loop_for_response = loop_.clone();
    dialog.connect_response(None, move |dialog, _| {
        dialog.close();
        loop_for_response.quit();
    });

    dialog.present(None::<&adw::gtk::Widget>);
    loop_.run();
}

pub(in crate::updater::standalone) fn active_window(
    app: &Application,
) -> Option<ApplicationWindow> {
    app.active_window()
        .and_then(|window| window.downcast::<ApplicationWindow>().ok())
}
