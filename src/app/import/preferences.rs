use super::reload::reload_state_with_status;
use super::sources::current_sources_for_reload;
use super::status::status_with_cache;
use super::*;
use adw::glib::variant::ToVariant;

pub(in crate::app) fn set_remember_mode(
    remember_mode: RememberMode,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let previous_remember_mode = ui.remember_mode.get();
    if previous_remember_mode == remember_mode {
        ui.preferences.set_remember_mode(remember_mode);
        show_status(ui, remember_mode.description());
        return;
    }

    let cache_cleanup_message =
        clear_cache_for_lower_remember_mode(previous_remember_mode, remember_mode, ui);
    ui.remember_mode.set(remember_mode);
    ui.preferences.set_remember_mode(remember_mode);
    {
        let mut data = state.borrow_mut();
        data.remember_mode = remember_mode;
        if remember_mode.opens_live_files() {
            data.transaction_sources.retain(TransactionSource::is_live);
        }
    }
    refresh_menu(ui, &state.borrow());
    reload_state_with_status(
        state,
        ui,
        "Applying Remember preference...",
        remember_mode_success_message(remember_mode, cache_cleanup_message),
        "Could not apply Remember preference: {error}",
        Vec::new(),
    );
}

fn clear_cache_for_lower_remember_mode(
    previous: RememberMode,
    current: RememberMode,
    ui: &UiHandles,
) -> Option<String> {
    if !current.retains_less_than(previous) {
        return None;
    }

    match data::clear_processed_app_data_cache() {
        Ok(true) => {
            show_verbose_status(
                ui,
                format!(
                    "remember lowered from {previous:?} to {current:?}; processed cache removed"
                ),
            );
            Some(tr("Data and analytics cache removed."))
        }
        Ok(false) => {
            show_verbose_status(
                ui,
                format!(
                    "remember lowered from {previous:?} to {current:?}; no processed cache present"
                ),
            );
            None
        }
        Err(error) => {
            show_verbose_status(
                ui,
                format!(
                    "remember lowered from {previous:?} to {current:?}; processed cache cleanup failed: {error:#}"
                ),
            );
            Some(trf(
                "Could not remove data and analytics cache: {error}",
                &[("error", format!("{error:#}"))],
            ))
        }
    }
}

fn remember_mode_success_message(
    remember_mode: RememberMode,
    cache_message: Option<String>,
) -> String {
    let mut message = trf(
        "Remember is set to {mode}.",
        &[("mode", tr(remember_mode.label()))],
    );
    if let Some(cache_message) = cache_message {
        message.push(' ');
        message.push_str(&cache_message);
    }
    message
}

pub(in crate::app) fn set_dedupe_enabled(
    enabled: bool,
    action: gtk::gio::SimpleAction,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let mode = DedupeMode::from_enabled(enabled);
    if state.borrow().dedupe_mode == mode {
        ui.preferences.set_dedupe_enabled(enabled);
        action.set_state(&enabled.to_variant());
        show_status(
            ui,
            &trf(
                "Duplicate filtering is {state}. {description}",
                &[
                    ("state", tr(mode.label())),
                    ("description", tr(mode.description())),
                ],
            ),
        );
        return;
    }

    let borrowed = state.borrow();
    let remember_mode = ui.remember_mode.get();
    let sources = current_sources_for_reload(&borrowed, remember_mode);
    let scope = current_transaction_load_scope(&borrowed, ui.as_ref());
    drop(borrowed);
    let state_for_dedupe = Rc::clone(state);
    let ui_for_dedupe = Rc::clone(ui);
    show_verbose_status(
        ui.as_ref(),
        format!(
            "dedupe reload started; enabled={enabled}; scope={scope:?}; remember={remember_mode:?}; sources={}",
            sources.len(),
        ),
    );
    show_status(ui, "Updating duplicate filtering...");
    begin_background_operation(ui.as_ref());
    action.set_enabled(false);

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            data::load_app_data_with_sources(mode, scope, remember_mode, &sources)
        });
        match task.await {
            Ok(Ok((mut new_data, capabilities))) => {
                new_data.dedupe_mode = mode;
                *state_for_dedupe.borrow_mut() = new_data;
                set_storage_capabilities(&ui_for_dedupe, capabilities);
                ui_for_dedupe.preferences.set_dedupe_enabled(enabled);
                action.set_state(&enabled.to_variant());
                render_views(
                    &state_for_dedupe.borrow(),
                    &ui_for_dedupe,
                    &state_for_dedupe,
                );
                refresh_menu(&ui_for_dedupe, &state_for_dedupe.borrow());
                show_verbose_status(
                    ui_for_dedupe.as_ref(),
                    format!(
                        "dedupe reload finished; transactions={}; reports={}",
                        state_for_dedupe.borrow().transactions.len(),
                        state_for_dedupe.borrow().reports.len(),
                    ),
                );
                let message = status_with_cache(
                    trf(
                        "Duplicate filtering is {state}. {description}",
                        &[
                            ("state", tr(mode.label())),
                            ("description", tr(mode.description())),
                        ],
                    ),
                    &state_for_dedupe.borrow(),
                );
                show_status(&ui_for_dedupe, &message);
            }
            Ok(Err(err)) => show_status(
                &ui_for_dedupe,
                &trf(
                    "Could not update duplicate filtering: {error}",
                    &[("error", format!("{err:#}"))],
                ),
            ),
            Err(_) => show_status(
                &ui_for_dedupe,
                "Duplicate filtering canceled: the background task stopped unexpectedly.",
            ),
        }
        finish_background_operation(ui_for_dedupe.as_ref());
        action.set_enabled(
            ui_for_dedupe
                .preferences
                .action_is_writable("dedupe-enabled"),
        );
    });
}
