use crate::app_info::RESOURCE_ID;

pub fn register() -> Result<(), adw::glib::Error> {
    adw::gio::resources_register_include!("compiled.gresource")
}

pub fn add_icon_theme_path(display: &adw::gtk::gdk::Display) {
    adw::gtk::IconTheme::for_display(display).add_resource_path(RESOURCE_ID);
}

#[cfg(test)]
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
}
