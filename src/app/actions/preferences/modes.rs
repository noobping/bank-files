use super::*;

pub(super) fn register_mode_preference_actions(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    register_dedupe_action(app, state, ui);
    register_remember_mode_action(app, state, ui);
}

fn register_dedupe_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let dedupe_action = gtk::gio::SimpleAction::new_stateful(
        "dedupe-enabled",
        None,
        &state.borrow().dedupe_mode.is_enabled().to_variant(),
    );
    dedupe_action.connect_activate(move |action, _| {
        let enabled = action
            .state()
            .and_then(|state| state.get::<bool>())
            .unwrap_or(true);
        action.change_state(&(!enabled).to_variant());
    });
    dedupe_action.set_enabled(ui.preferences.action_is_writable("dedupe-enabled"));

    let state_for_dedupe = Rc::clone(state);
    let ui_for_dedupe = Rc::clone(ui);
    dedupe_action.connect_change_state(move |action, value| {
        let Some(enabled) = value.and_then(|value| value.get::<bool>()) else {
            return;
        };
        set_dedupe_enabled(enabled, action.clone(), &state_for_dedupe, &ui_for_dedupe);
    });
    app.add_action(&dedupe_action);
}

fn register_remember_mode_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_remember_mode = Rc::clone(state);
    let ui_for_remember_mode = Rc::clone(ui);
    let remember_mode_action = gtk::gio::SimpleAction::new_stateful(
        "remember-mode",
        Some(&String::static_variant_type()),
        &ui.remember_mode.get().as_settings().to_variant(),
    );
    remember_mode_action.connect_activate(move |action, _| {
        let current = action
            .state()
            .and_then(|state| state.get::<String>())
            .map(|state| RememberMode::from_settings(&state))
            .unwrap_or_default();
        let next_index = RememberMode::SETTINGS_VALUES
            .iter()
            .position(|mode| *mode == current)
            .map(|index| (index + 1) % RememberMode::SETTINGS_VALUES.len())
            .unwrap_or(0);
        action.change_state(
            &RememberMode::SETTINGS_VALUES[next_index]
                .as_settings()
                .to_variant(),
        );
    });
    remember_mode_action.set_enabled(ui.preferences.action_is_writable("remember-mode"));
    remember_mode_action.connect_change_state(move |action, value| {
        let Some(value) = value.and_then(|value| value.get::<String>()) else {
            return;
        };
        let remember_mode = RememberMode::from_settings(&value);
        action.set_state(&remember_mode.as_settings().to_variant());
        set_remember_mode(
            remember_mode,
            &state_for_remember_mode,
            &ui_for_remember_mode,
        );
    });
    app.add_action(&remember_mode_action);
}
