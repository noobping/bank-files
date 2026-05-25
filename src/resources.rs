use crate::app_info::RESOURCE_ID;

#[cfg(bank_files_installed_resources)]
pub fn register() -> Result<(), adw::glib::Error> {
    let resource = adw::gio::Resource::load(installed_resource_path())?;
    adw::gio::resources_register(&resource);
    Ok(())
}

#[cfg(not(bank_files_installed_resources))]
pub fn register() -> Result<(), adw::glib::Error> {
    adw::gio::resources_register_include!("compiled.gresource")
}

#[cfg(bank_files_installed_resources)]
fn installed_resource_path() -> &'static str {
    option_env!("BANK_FILES_GRESOURCE")
        .unwrap_or("/usr/local/share/bank-files/bank-files.gresource")
}

pub fn add_icon_theme_path(display: &adw::gtk::gdk::Display) {
    adw::gtk::IconTheme::for_display(display).add_resource_path(RESOURCE_ID);
}

#[cfg(all(test, not(bank_files_installed_resources)))]
mod tests {
    use super::*;
    use crate::app_info::APP_ID;

    fn assert_embedded_resource(relative_path: &str, message: &str) {
        register().expect("register embedded resources");
        let path = format!("{RESOURCE_ID}/{relative_path}");
        adw::gio::resources_lookup_data(&path, adw::gio::ResourceLookupFlags::NONE).expect(message);
    }

    #[test]
    fn embedded_app_icon_is_available() {
        assert_embedded_resource(
            &format!("scalable/apps/{APP_ID}.svg"),
            "embedded app icon should be available",
        );
    }

    #[test]
    fn embedded_symbolic_action_icon_is_available() {
        assert_embedded_resource(
            "symbolic/actions/document-save-symbolic.svg",
            "embedded action icon should be available",
        );
    }

    #[test]
    fn embedded_status_bar_ui_is_available() {
        assert_embedded_resource(
            "ui/status-bar.ui",
            "embedded status bar UI should be available",
        );
    }

    #[test]
    fn embedded_status_history_dialog_ui_is_available() {
        assert_embedded_resource(
            "ui/status-history-dialog.ui",
            "embedded status history dialog UI should be available",
        );
    }

    #[test]
    fn embedded_style_css_is_available() {
        assert_embedded_resource("css/style.css", "embedded style CSS should be available");
    }

    #[test]
    fn embedded_loading_placeholder_ui_is_available() {
        assert_embedded_resource(
            "ui/loading-placeholder.ui",
            "embedded loading placeholder UI should be available",
        );
    }

    #[test]
    fn embedded_management_dialog_ui_is_available() {
        assert_embedded_resource(
            "ui/management-dialog.ui",
            "embedded management dialog UI should be available",
        );
    }

    #[test]
    fn embedded_main_window_ui_is_available() {
        assert_embedded_resource(
            "ui/main-window.ui",
            "embedded main window UI should be available",
        );
    }

    #[test]
    fn embedded_settings_dialog_ui_is_available() {
        assert_embedded_resource(
            "ui/settings-dialog.ui",
            "embedded settings dialog UI should be available",
        );
    }

    #[test]
    fn embedded_shortcuts_dialog_ui_is_available() {
        assert_embedded_resource(
            "ui/shortcuts-dialog.ui",
            "embedded shortcuts dialog UI should be available",
        );
    }

    #[test]
    fn embedded_action_dialog_ui_is_available() {
        assert_embedded_resource(
            "ui/action-dialog.ui",
            "embedded action dialog UI should be available",
        );
    }

    #[test]
    fn embedded_fake_transactions_dialog_ui_is_available() {
        assert_embedded_resource(
            "ui/fake-transactions-dialog.ui",
            "embedded fake transactions dialog UI should be available",
        );
    }

    #[test]
    fn embedded_operation_queue_dialog_ui_is_available() {
        assert_embedded_resource(
            "ui/operation-queue-dialog.ui",
            "embedded operation queue dialog UI should be available",
        );
    }

    #[test]
    fn embedded_partial_load_notice_ui_is_available() {
        assert_embedded_resource(
            "ui/partial-load-notice.ui",
            "embedded partial load notice UI should be available",
        );
    }

    #[test]
    fn embedded_rule_search_chips_ui_is_available() {
        assert_embedded_resource(
            "ui/rule-search-chips.ui",
            "embedded rule search chips UI should be available",
        );
    }
}
