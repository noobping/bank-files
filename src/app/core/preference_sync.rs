use super::*;

pub(super) fn connect_preference_sync(ui: &Rc<UiHandles>) {
    let preferences_for_tab = ui.preferences.clone();
    ui.stack.connect_visible_child_name_notify(move |stack| {
        if let Some(name) = stack.visible_child_name() {
            preferences_for_tab.set_active_tab(name.as_str());
        }
    });

    let preferences_for_window = ui.preferences.clone();
    ui.window.connect_close_request(move |window| {
        let maximized = window.is_maximized();
        let (width, height) = if maximized {
            (
                preferences_for_window.window_width(),
                preferences_for_window.window_height(),
            )
        } else {
            (window.width(), window.height())
        };
        preferences_for_window.set_window_state(width, height, maximized);
        gtk::glib::Propagation::Proceed
    });
}
