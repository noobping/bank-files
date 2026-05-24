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
    register_auto_clean_config_action(app, ui);
}

fn register_advanced_features_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_advanced_features = Rc::clone(state);
    let ui_for_advanced_features = Rc::clone(ui);
    #[cfg(feature = "smart-insights")]
    let app_for_advanced_features = app.clone();
    #[cfg(not(feature = "smart-insights"))]
    let app_for_advanced_features = app.clone();
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
            update_smart_dependent_actions(
                enabled,
                &app_for_advanced_features,
                &ui_for_advanced_features,
            );
            clear_smart_filter_when_disabled(enabled, &ui_for_advanced_features);

            let success_message = tr(if enabled {
                "Advanced Features enabled. Rule editing and direction controls are available."
            } else {
                "Simple mode enabled. Rule editing and direction controls are hidden."
            });
            if ui_for_advanced_features.show_predictions.get() {
                reload_state_with_status(
                    &state_for_advanced_features,
                    &ui_for_advanced_features,
                    "Updating Advanced Features...",
                    success_message,
                    "Could not update Advanced Features: {error}",
                    Vec::new(),
                );
            } else {
                render_views(
                    &state_for_advanced_features.borrow(),
                    &ui_for_advanced_features,
                    &state_for_advanced_features,
                );
                show_status(&ui_for_advanced_features, &success_message);
            }
        },
    );
    advanced_features_action.set_enabled(ui.preferences.action_is_writable("advanced-features"));
}

#[cfg(feature = "smart-insights")]
fn update_smart_dependent_actions(enabled: bool, app: &adw::Application, ui: &UiHandles) {
    set_simple_action_enabled(
        app,
        "show-predictions",
        enabled && ui.preferences.action_is_writable("show-predictions"),
    );
    set_simple_action_enabled(
        app,
        "hide-canceled-transactions",
        smart_dependent_action_enabled(
            enabled,
            ui.show_predictions.get(),
            ui.preferences
                .action_is_writable("hide-canceled-transactions"),
        ),
    );
    #[cfg(not(feature = "flatpak"))]
    set_simple_action_enabled(
        app,
        "online-smart-insights",
        smart_dependent_action_enabled(
            enabled,
            ui.show_predictions.get(),
            ui.preferences.action_is_writable("online-smart-insights"),
        ),
    );
}

#[cfg(not(feature = "smart-insights"))]
fn update_smart_dependent_actions(_enabled: bool, _app: &adw::Application, _ui: &UiHandles) {}

#[cfg(feature = "smart-insights")]
fn clear_smart_filter_when_disabled(enabled: bool, ui: &UiHandles) {
    if !enabled
        && matches!(
            ui.active_transaction_filter.borrow().as_ref(),
            Some(TransactionFilter::Pattern(_))
        )
    {
        *ui.active_transaction_filter.borrow_mut() = None;
    }
}

#[cfg(not(feature = "smart-insights"))]
fn clear_smart_filter_when_disabled(_enabled: bool, _ui: &UiHandles) {}

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

fn register_auto_clean_config_action(app: &adw::Application, ui: &Rc<UiHandles>) {
    let ui_for_auto_clean_config = Rc::clone(ui);
    let auto_clean_config_action = add_bool_toggle_action(
        app,
        "auto-clean-config",
        ui.auto_clean_config.get(),
        true,
        move |enabled| {
            ui_for_auto_clean_config.auto_clean_config.set(enabled);
            ui_for_auto_clean_config
                .preferences
                .set_auto_clean_config(enabled);
            show_status(
                &ui_for_auto_clean_config,
                if enabled {
                    "Auto Clean Config enabled. Orphaned rules are removed during reload and import."
                } else {
                    "Auto Clean Config disabled. Orphaned rules stay visible in Diagnostics."
                },
            );
        },
    );
    auto_clean_config_action.set_enabled(ui.preferences.action_is_writable("auto-clean-config"));
}
