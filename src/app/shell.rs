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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::app) struct MainMenuCounts {
    pub(in crate::app) fake_transactions: usize,
    pub(in crate::app) pending_operations: usize,
}

pub(in crate::app) fn build_menu_model(
    _data: &AppData,
    advanced_features: bool,
    storage_capabilities: &data::StorageCapabilities,
    preferences: &Preferences,
    counts: MainMenuCounts,
) -> gtk::gio::Menu {
    let menu = gtk::gio::Menu::new();
    menu.append(Some(&tr("Open Bank Files")), Some("app.import-csv"));
    menu.append(Some(&tr("Search")), Some("app.find"));
    menu.append(
        Some(&fake_transactions_menu_label(counts.fake_transactions)),
        Some("app.fake-transactions"),
    );
    menu.append(
        Some(&operation_queue_menu_label(counts.pending_operations)),
        Some("app.operation-queue"),
    );
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
        manage_section.append(Some(&tr("Categorization Rules")), Some("app.manage-rules"));
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

fn main_menu_counts(ui_handles: &UiHandles) -> MainMenuCounts {
    MainMenuCounts {
        fake_transactions: ui_handles.fake_transactions.count(),
        pending_operations: ui_handles.operation_queue.actionable_count(),
    }
}

fn fake_transactions_menu_label(count: usize) -> String {
    if count == 0 {
        tr("Fake transactions")
    } else {
        trf(
            "Fake transactions ({count})",
            &[("count", count.to_string())],
        )
    }
}

fn operation_queue_menu_label(count: usize) -> String {
    if count == 0 {
        tr("Processing queue")
    } else {
        trf(
            "Processing queue ({count})",
            &[("count", count.to_string())],
        )
    }
}

pub(in crate::app) fn refresh_menu(ui_handles: &UiHandles, data: &AppData) {
    let storage_capabilities = ui_handles.storage_capabilities.borrow();
    let menu = build_menu_model(
        data,
        ui_handles.advanced_features.get(),
        &storage_capabilities,
        &ui_handles.preferences,
        main_menu_counts(ui_handles),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_menu_labels_include_session_counts() {
        let menu = build_menu_model(
            &AppData::default(),
            false,
            &data::StorageCapabilities::default(),
            &Preferences::default(),
            MainMenuCounts {
                fake_transactions: 3,
                pending_operations: 2,
            },
        );

        let fake_label = trf("Fake transactions ({count})", &[("count", "3".to_string())]);
        let queue_label = trf("Processing queue ({count})", &[("count", "2".to_string())]);

        assert_eq!(menu_label(&menu, 2).as_deref(), Some(fake_label.as_str()));
        assert_eq!(menu_label(&menu, 3).as_deref(), Some(queue_label.as_str()));
    }

    #[test]
    fn main_menu_labels_hide_zero_counts() {
        let menu = build_menu_model(
            &AppData::default(),
            false,
            &data::StorageCapabilities::default(),
            &Preferences::default(),
            MainMenuCounts::default(),
        );

        let fake_label = tr("Fake transactions");
        let queue_label = tr("Processing queue");
        assert_eq!(menu_label(&menu, 2).as_deref(), Some(fake_label.as_str()));
        assert_eq!(menu_label(&menu, 3).as_deref(), Some(queue_label.as_str()));
    }

    fn menu_label(menu: &gtk::gio::Menu, index: i32) -> Option<String> {
        menu.item_attribute_value(index, "label", None)
            .and_then(|value| value.get::<String>())
    }
}
