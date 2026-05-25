use super::*;

pub(super) fn register_feature_preference_actions(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    register_advanced_features_action(app, state, ui);
    register_show_all_action(app, state, ui);
    register_compare_categories_action(app, state, ui);
    register_advanced_autofill_action(app, ui);
}

fn register_advanced_features_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_advanced_features = Rc::clone(state);
    let ui_for_advanced_features = Rc::clone(ui);
    let advanced_features_action = add_bool_toggle_action(
        app,
        "advanced-features",
        ui.advanced_features.get(),
        false,
        move |enabled| {
            ui_for_advanced_features.advanced_features.set(enabled);
            ui_for_advanced_features
                .preferences
                .set_advanced_features(enabled);
            refresh_write_actions(ui_for_advanced_features.as_ref());
            refresh_menu(
                &ui_for_advanced_features,
                &state_for_advanced_features.borrow(),
            );
            let success_message = tr(if enabled {
                "Advanced Features enabled. Budget codes, direction controls, and detailed move options are available."
            } else {
                "Simple mode enabled. Budget moves stay focused on matching categories."
            });
            render_views(
                &state_for_advanced_features.borrow(),
                &ui_for_advanced_features,
                &state_for_advanced_features,
            );
            show_status(&ui_for_advanced_features, &success_message);
        },
    );
    advanced_features_action.set_enabled(ui.preferences.action_is_writable("advanced-features"));
}

fn register_show_all_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_show_all = Rc::clone(state);
    let ui_for_show_all = Rc::clone(ui);
    let show_all_action =
        add_bool_toggle_action(app, "show-all", ui.show_all.get(), false, move |enabled| {
            ui_for_show_all.show_all.set(enabled);
            ui_for_show_all.preferences.set_show_all(enabled);
            render_views(
                &state_for_show_all.borrow(),
                &ui_for_show_all,
                &state_for_show_all,
            );
            show_status(
                &ui_for_show_all,
                if enabled {
                    "Full lists enabled. More rows are hidden."
                } else {
                    "Preview mode enabled. Sections show More rows again."
                },
            );
        });
    show_all_action.set_enabled(ui.preferences.action_is_writable("show-all"));
}

fn register_compare_categories_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_compare_categories = Rc::clone(state);
    let ui_for_compare_categories = Rc::clone(ui);
    let compare_categories_action = add_bool_toggle_action(
        app,
        "compare-categories-previous-period",
        ui.compare_categories_previous_period.get(),
        true,
        move |enabled| {
            ui_for_compare_categories
                .compare_categories_previous_period
                .set(enabled);
            ui_for_compare_categories
                .preferences
                .set_compare_categories_previous_period(enabled);
            render_views(
                &state_for_compare_categories.borrow(),
                &ui_for_compare_categories,
                &state_for_compare_categories,
            );
            show_status(
                &ui_for_compare_categories,
                if enabled {
                    "Spending comparison enabled. Spending is compared with the previous period."
                } else {
                    "Spending comparison disabled. Spending comparisons are hidden."
                },
            );
        },
    );
    compare_categories_action.set_enabled(
        ui.preferences
            .action_is_writable("compare-categories-previous-period"),
    );
}

fn register_advanced_autofill_action(app: &adw::Application, ui: &Rc<UiHandles>) {
    let ui_for_advanced_autofill = Rc::clone(ui);
    let advanced_autofill_action = add_bool_toggle_action(
        app,
        "advanced-autofill",
        ui.advanced_autofill.get(),
        true,
        move |enabled| {
            ui_for_advanced_autofill.advanced_autofill.set(enabled);
            ui_for_advanced_autofill
                .preferences
                .set_advanced_autofill(enabled);
            show_status(
                &ui_for_advanced_autofill,
                if enabled {
                    "Whole Form Autofill enabled. Forms can fill matching categories, budget codes, and directions."
                } else {
                    "Whole Form Autofill disabled. Forms only use values you select."
                },
            );
        },
    );
    advanced_autofill_action.set_enabled(ui.preferences.action_is_writable("advanced-autofill"));
}
