use super::*;

#[cfg(feature = "smart-insights")]
pub(super) fn register_smart_preference_actions(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    register_show_predictions_action(app, state, ui);
    register_online_smart_insights_action(app, ui);
    register_hide_canceled_action(app, state, ui);
}

#[cfg(not(feature = "smart-insights"))]
pub(super) fn register_smart_preference_actions(
    _app: &adw::Application,
    _state: &Rc<RefCell<AppData>>,
    _ui: &Rc<UiHandles>,
) {
}

#[cfg(feature = "smart-insights")]
fn register_show_predictions_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_predictions = Rc::clone(state);
    let ui_for_predictions = Rc::clone(ui);
    let app_for_predictions = app.clone();
    let show_predictions_action = add_bool_toggle_action(
        app,
        "show-predictions",
        ui.show_predictions.get(),
        false,
        move |enabled| {
            ui_for_predictions.show_predictions.set(enabled);
            ui_for_predictions.preferences.set_show_predictions(enabled);
            set_simple_action_enabled(
                &app_for_predictions,
                "hide-canceled-transactions",
                smart_dependent_action_enabled(
                    ui_for_predictions.advanced_features.get(),
                    enabled,
                    ui_for_predictions
                        .preferences
                        .action_is_writable("hide-canceled-transactions"),
                ),
            );
            update_online_action_enabled(&app_for_predictions, enabled, &ui_for_predictions);
            clear_pattern_filter_when_predictions_disabled(enabled, &ui_for_predictions);

            let success_message = tr(if enabled {
                "Smart Insights enabled. Forecast cards, transaction pattern detection, and smart transfer detection are available."
            } else {
                "Smart Insights disabled. Forecast cards, transaction pattern detection, and smart transfer detection are off."
            });
            reload_state_with_status(
                &state_for_predictions,
                &ui_for_predictions,
                "Updating Smart Insights...",
                success_message,
                "Could not update Smart Insights: {error}",
                Vec::new(),
            );
        },
    );
    show_predictions_action.set_enabled(
        ui.advanced_features.get() && ui.preferences.action_is_writable("show-predictions"),
    );
}

#[cfg(all(feature = "smart-insights", not(feature = "flatpak")))]
fn update_online_action_enabled(app: &adw::Application, enabled: bool, ui: &UiHandles) {
    set_simple_action_enabled(
        app,
        "online-smart-insights",
        smart_dependent_action_enabled(
            ui.advanced_features.get(),
            enabled,
            ui.preferences.action_is_writable("online-smart-insights"),
        ),
    );
}

#[cfg(all(feature = "smart-insights", feature = "flatpak"))]
fn update_online_action_enabled(_app: &adw::Application, _enabled: bool, _ui: &UiHandles) {}

#[cfg(feature = "smart-insights")]
fn clear_pattern_filter_when_predictions_disabled(enabled: bool, ui: &UiHandles) {
    if !enabled
        && matches!(
            ui.active_transaction_filter.borrow().as_ref(),
            Some(TransactionFilter::Pattern(_))
        )
    {
        *ui.active_transaction_filter.borrow_mut() = None;
    }
}

#[cfg(all(feature = "smart-insights", not(feature = "flatpak")))]
fn register_online_smart_insights_action(app: &adw::Application, ui: &Rc<UiHandles>) {
    let ui_for_online_smart_insights = Rc::clone(ui);
    let online_smart_insights_action = add_bool_toggle_action(
        app,
        "online-smart-insights",
        ui.online_smart_insights.get(),
        false,
        move |enabled| {
            ui_for_online_smart_insights
                .online_smart_insights
                .set(enabled);
            ui_for_online_smart_insights
                .preferences
                .set_online_smart_insights(enabled);
            show_status(
                &ui_for_online_smart_insights,
                if enabled {
                    "Online Smart Insights enabled. Only privacy-filtered company labels may be used for online lookups."
                } else {
                    "Online Smart Insights disabled. Smart Insights use local transactions only."
                },
            );
        },
    );
    online_smart_insights_action.set_enabled(smart_dependent_action_enabled(
        ui.advanced_features.get(),
        ui.show_predictions.get(),
        ui.preferences.action_is_writable("online-smart-insights"),
    ));
}

#[cfg(all(feature = "smart-insights", feature = "flatpak"))]
fn register_online_smart_insights_action(_app: &adw::Application, _ui: &Rc<UiHandles>) {}

#[cfg(feature = "smart-insights")]
fn register_hide_canceled_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_hide_canceled = Rc::clone(state);
    let ui_for_hide_canceled = Rc::clone(ui);
    let hide_canceled_action = add_bool_toggle_action(
        app,
        "hide-canceled-transactions",
        ui.hide_canceled_transactions.get(),
        false,
        move |enabled| {
            if !smart_pattern_detection_enabled(
                ui_for_hide_canceled.advanced_features.get(),
                ui_for_hide_canceled.show_predictions.get(),
            ) {
                show_status(
                    &ui_for_hide_canceled,
                    "Hide Refunded Transactions needs Smart Insights. Enable Smart Insights to use refunded transaction hiding.",
                );
                return;
            }
            ui_for_hide_canceled.hide_canceled_transactions.set(enabled);
            ui_for_hide_canceled
                .preferences
                .set_hide_canceled_transactions(enabled);
            render_views(
                &state_for_hide_canceled.borrow(),
                &ui_for_hide_canceled,
                &state_for_hide_canceled,
            );
            show_status(
                &ui_for_hide_canceled,
                if enabled {
                    "Hide Refunded Transactions enabled. Detected refunds and offsetting groups are excluded from normal views."
                } else {
                    "Hide Refunded Transactions disabled. Detected refunds and offsetting groups are included again."
                },
            );
        },
    );
    hide_canceled_action.set_enabled(smart_dependent_action_enabled(
        ui.advanced_features.get(),
        ui.show_predictions.get(),
        ui.preferences
            .action_is_writable("hide-canceled-transactions"),
    ));
}
