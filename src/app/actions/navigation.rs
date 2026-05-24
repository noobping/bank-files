use super::*;

pub(super) fn register_navigation_actions(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    register_back_action(app, state, ui);
    register_find_action(app, ui);
    register_search_preset_action(app, state, ui);
}

fn register_back_action(app: &adw::Application, state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
    let state_for_back = Rc::clone(state);
    let ui_for_back = Rc::clone(ui);
    let back_action = gtk::gio::SimpleAction::new("go-back", None);
    back_action.connect_activate(move |_, _| {
        navigate_back(&ui_for_back, &state_for_back);
    });
    app.add_action(&back_action);
}

fn register_find_action(app: &adw::Application, ui: &Rc<UiHandles>) {
    let ui_for_find = Rc::clone(ui);
    let find_action = gtk::gio::SimpleAction::new("find", None);
    find_action.connect_activate(move |_, _| {
        if focus_fake_transaction_search(&ui_for_find) {
            return;
        }
        ui::toggle_search_bar(&ui_for_find.search_bar, &ui_for_find.search_entry);
    });
    app.add_action(&find_action);
}

fn register_search_preset_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_search_preset = Rc::clone(state);
    let ui_for_search_preset = Rc::clone(ui);
    let search_preset_action =
        gtk::gio::SimpleAction::new(SEARCH_PRESET_ACTION, Some(&String::static_variant_type()));
    search_preset_action.connect_activate(move |_, parameter| {
        let Some(preset) = parameter.and_then(|value| value.get::<String>()) else {
            return;
        };
        apply_search_preset(&state_for_search_preset, &ui_for_search_preset, &preset);
    });
    app.add_action(&search_preset_action);
}
