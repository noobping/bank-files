use super::*;

pub(in crate::app) fn open_files(app: &adw::Application, files: &[gtk::gio::File], _hint: &str) {
    let uris = files
        .iter()
        .map(|file| file.uri().to_string())
        .collect::<Vec<_>>();
    if uris.is_empty() {
        build_ui(app);
        return;
    }

    let active = ACTIVE_SESSION.with(|active| active.borrow().clone());
    if let Some(session) = active {
        session.ui.window.present();
        import_uris_into_session(uris, session.state, session.ui);
    } else {
        build_ui_with_opened_uris(app, uris);
    }
}

pub(in crate::app) fn build_menu_model(
    _data: &AppData,
    advanced_features: bool,
    storage_capabilities: &data::StorageCapabilities,
    preferences: &Preferences,
) -> gtk::gio::Menu {
    let menu = gtk::gio::Menu::new();
    menu.append(Some(&tr("Open CSV Files")), Some("app.import-csv"));
    menu.append(Some(&tr("Search")), Some("app.find"));
    if advanced_features {
        menu.append(Some(&tr("Quick Reload")), Some("app.reload"));
        menu.append(Some(&tr("Reload All")), Some("app.reload-all"));
        menu.append(
            Some(&tr("Clear Cache and Reload")),
            Some("app.clear-cache-and-reload"),
        );
    }
    menu.append(Some(&tr("Print Page")), Some("app.print-page"));

    let manage_section = gtk::gio::Menu::new();
    if storage_capabilities.config_writable || advanced_features {
        if advanced_features {
            manage_section.append(Some(&tr("Categorization Rules")), Some("app.manage-rules"));
        }
        manage_section.append(Some(&tr("Budgets")), Some("app.manage-budgets"));
        manage_section.append(
            Some(&tr("Normalize CSV Fields")),
            Some("app.manage-aliases"),
        );
    }
    menu.append_section(None, &manage_section);

    let app_section = gtk::gio::Menu::new();
    if storage_capabilities.config_writable || advanced_features {
        app_section.append(Some(&tr("Configuration")), Some("app.configuration"));
    }
    if preferences.any_writable() {
        app_section.append(Some(&tr("Preferences")), Some("app.preferences"));
    }
    #[cfg(all(target_os = "linux", feature = "setup", not(feature = "flatpak")))]
    if setup::can_install_locally() {
        app_section.append(
            Some(&tr(setup::local_menu_action_label(
                setup::is_installed_locally(),
            ))),
            Some("app.install-locally"),
        );
    }
    if updater::supports_update_checks() {
        app_section.append(
            Some(&tr("Check for Updates")),
            Some("app.check-for-updates"),
        );
    }
    menu.append_section(None, &app_section);

    let help_section = gtk::gio::Menu::new();
    help_section.append(Some(&tr("Keyboard Shortcuts")), Some("app.shortcuts"));
    help_section.append(Some(&tr("About")), Some("app.about"));
    help_section.append(Some(&tr("Quit")), Some("app.quit"));
    menu.append_section(None, &help_section);

    menu
}

pub(in crate::app) fn refresh_menu(ui_handles: &UiHandles, data: &AppData) {
    let storage_capabilities = ui_handles.storage_capabilities.borrow();
    let menu = build_menu_model(
        data,
        ui_handles.advanced_features.get(),
        &storage_capabilities,
        &ui_handles.preferences,
    );
    ui_handles.menu_button.set_menu_model(Some(&menu));
}

pub(in crate::app) fn add_responsive_switcher(
    window: &adw::ApplicationWindow,
    switcher: &adw::ViewSwitcher,
    switcher_bar: &adw::ViewSwitcherBar,
    mobile_title: &adw::WindowTitle,
) {
    let Ok(condition) = adw::BreakpointCondition::parse("max-width: 720sp") else {
        return;
    };
    let breakpoint = adw::Breakpoint::new(condition);
    breakpoint.add_setter(switcher, "visible", Some(&false.to_value()));
    breakpoint.add_setter(switcher_bar, "reveal", Some(&true.to_value()));
    breakpoint.add_setter(mobile_title, "visible", Some(&true.to_value()));
    window.add_breakpoint(breakpoint);
}

pub(in crate::app) fn add_responsive_page_margins(
    window: &adw::ApplicationWindow,
    switcher: &adw::ViewSwitcher,
    switcher_bar: &adw::ViewSwitcherBar,
    mobile_title: &adw::WindowTitle,
    pages: &[&gtk::Box],
) {
    let Ok(condition) = adw::BreakpointCondition::parse("max-width: 480sp") else {
        return;
    };
    let breakpoint = adw::Breakpoint::new(condition);
    breakpoint.add_setter(switcher, "visible", Some(&false.to_value()));
    breakpoint.add_setter(switcher_bar, "reveal", Some(&true.to_value()));
    breakpoint.add_setter(mobile_title, "visible", Some(&true.to_value()));
    for page in pages {
        breakpoint.add_setter(*page, "margin-start", Some(&0.to_value()));
        breakpoint.add_setter(*page, "margin-end", Some(&0.to_value()));
        breakpoint.add_setter(*page, "margin-top", Some(&12.to_value()));
        breakpoint.add_setter(*page, "margin-bottom", Some(&12.to_value()));
    }
    window.add_breakpoint(breakpoint);
}

pub(in crate::app) fn add_responsive_switcher_for_dialog(
    dialog: &adw::Dialog,
    switcher: &adw::ViewSwitcher,
    switcher_bar: &adw::ViewSwitcherBar,
) {
    let Ok(condition) = adw::BreakpointCondition::parse("max-width: 720sp") else {
        return;
    };
    let breakpoint = adw::Breakpoint::new(condition);
    breakpoint.add_setter(switcher, "visible", Some(&false.to_value()));
    breakpoint.add_setter(switcher_bar, "reveal", Some(&true.to_value()));
    dialog.add_breakpoint(breakpoint);
}
