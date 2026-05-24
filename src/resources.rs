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

    #[test]
    fn embedded_app_icon_is_available() {
        register().expect("register embedded resources");
        let path = format!("{RESOURCE_ID}/scalable/apps/{APP_ID}.svg");
        adw::gio::resources_lookup_data(&path, adw::gio::ResourceLookupFlags::NONE)
            .expect("embedded app icon should be available");
    }

    #[test]
    fn embedded_symbolic_action_icon_is_available() {
        register().expect("register embedded resources");
        let path = format!("{RESOURCE_ID}/symbolic/actions/document-save-symbolic.svg");
        adw::gio::resources_lookup_data(&path, adw::gio::ResourceLookupFlags::NONE)
            .expect("embedded action icon should be available");
    }

    #[test]
    fn embedded_status_bar_ui_is_available() {
        register().expect("register embedded resources");
        let path = format!("{RESOURCE_ID}/ui/status-bar.ui");
        adw::gio::resources_lookup_data(&path, adw::gio::ResourceLookupFlags::NONE)
            .expect("embedded status bar UI should be available");
    }

    #[test]
    fn embedded_style_css_is_available() {
        register().expect("register embedded resources");
        let path = format!("{RESOURCE_ID}/css/style.css");
        adw::gio::resources_lookup_data(&path, adw::gio::ResourceLookupFlags::NONE)
            .expect("embedded style CSS should be available");
    }

    #[test]
    fn embedded_management_dialog_ui_is_available() {
        register().expect("register embedded resources");
        let path = format!("{RESOURCE_ID}/ui/management-dialog.ui");
        adw::gio::resources_lookup_data(&path, adw::gio::ResourceLookupFlags::NONE)
            .expect("embedded management dialog UI should be available");
    }

    #[test]
    fn embedded_settings_dialog_ui_is_available() {
        register().expect("register embedded resources");
        let path = format!("{RESOURCE_ID}/ui/settings-dialog.ui");
        adw::gio::resources_lookup_data(&path, adw::gio::ResourceLookupFlags::NONE)
            .expect("embedded settings dialog UI should be available");
    }

    #[test]
    fn embedded_action_dialog_ui_is_available() {
        register().expect("register embedded resources");
        let path = format!("{RESOURCE_ID}/ui/action-dialog.ui");
        adw::gio::resources_lookup_data(&path, adw::gio::ResourceLookupFlags::NONE)
            .expect("embedded action dialog UI should be available");
    }
}
