use super::*;

pub(super) fn add_view_action(
    app: &adw::Application,
    ui: &Rc<UiHandles>,
    action_name: &str,
    page_name: &str,
) {
    let ui_for_action = Rc::clone(ui);
    let page_name = page_name.to_string();
    let action = gtk::gio::SimpleAction::new(action_name, None);
    action.connect_activate(move |_, _| {
        ui_for_action.stack.set_visible_child_name(&page_name);
    });
    app.add_action(&action);
}

#[cfg(feature = "smart-insights")]
pub(super) fn set_simple_action_enabled(app: &adw::Application, name: &str, enabled: bool) {
    if let Some(action) = app
        .lookup_action(name)
        .and_then(|action| action.downcast::<gtk::gio::SimpleAction>().ok())
    {
        action.set_enabled(enabled);
    }
}

pub(super) fn add_bool_toggle_action<F>(
    app: &adw::Application,
    name: &str,
    initial: bool,
    fallback: bool,
    on_change: F,
) -> gtk::gio::SimpleAction
where
    F: Fn(bool) + 'static,
{
    let action = gtk::gio::SimpleAction::new_stateful(name, None, &initial.to_variant());
    action.connect_activate(move |action, _| {
        let enabled = action
            .state()
            .and_then(|state| state.get::<bool>())
            .unwrap_or(fallback);
        action.change_state(&(!enabled).to_variant());
    });
    action.connect_change_state(move |action, value| {
        let Some(enabled) = value.and_then(|value| value.get::<bool>()) else {
            return;
        };
        on_change(enabled);
        action.set_state(&enabled.to_variant());
    });
    app.add_action(&action);
    action
}
