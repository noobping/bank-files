use super::*;

#[cfg(all(target_os = "linux", feature = "setup", not(feature = "flatpak")))]
pub(super) fn register_local_setup_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    menu_button: gtk::MenuButton,
) {
    let ui_for_install = Rc::clone(ui);
    let menu_button_for_install = menu_button.clone();
    let state_for_install_menu = Rc::clone(state);
    let install_action = gtk::gio::SimpleAction::new("install-locally", None);
    install_action.set_enabled(setup::can_install_locally());
    install_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }

        action.set_enabled(false);
        let installed = setup::is_installed_locally();
        let result = if installed {
            setup::uninstall_locally()
        } else {
            setup::install_locally()
        };

        match result {
            Ok(()) => {
                let message = if installed {
                    "Removed from app menu."
                } else {
                    "Added to app menu."
                };
                show_status(&ui_for_install, message);
                let storage_capabilities = ui_for_install.storage_capabilities.borrow();
                let menu = build_menu_model(
                    &state_for_install_menu.borrow(),
                    ui_for_install.advanced_features.get(),
                    &storage_capabilities,
                    &ui_for_install.preferences,
                );
                menu_button_for_install.set_menu_model(Some(&menu));
            }
            Err(_) => {
                show_status(&ui_for_install, "Couldn't update the app menu.");
            }
        }

        action.set_enabled(setup::can_install_locally());
    });
    app.add_action(&install_action);
}
