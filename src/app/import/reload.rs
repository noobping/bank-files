use super::sources::current_sources_for_reload;
use super::status::status_with_cache;
use super::*;

pub(in crate::app) fn reload_state(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
    reload_state_with_status(
        state,
        ui,
        "Quick reloading current data...",
        tr("Current data reloaded."),
        "Reload error: {error}",
        Vec::new(),
    );
}

pub(in crate::app) fn clear_cache_and_reload_state(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let borrowed = state.borrow();
    let mode = borrowed.dedupe_mode;
    let remember_mode = ui.remember_mode.get();
    let sources = current_sources_for_reload(&borrowed, remember_mode);
    let scope = current_transaction_load_scope(&borrowed, ui.as_ref());
    drop(borrowed);
    let auto_clean_config = ui.preferences.auto_clean_config();
    let state_for_reload = Rc::clone(state);
    let ui_for_reload = Rc::clone(ui);

    show_verbose_status(
        ui.as_ref(),
        format!(
            "cache purge reload started; scope={scope:?}; remember={remember_mode:?}; sources={}; dedupe={mode:?}",
            sources.len(),
        ),
    );
    show_status(ui, "Clearing data and analytics cache...");
    render_loading_placeholder(ui.as_ref());
    begin_background_operation(ui.as_ref());

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            let removed = data::clear_processed_app_data_cache()?;
            let (new_data, capabilities) = data::load_app_data_with_sources(
                mode,
                auto_clean_config,
                scope,
                remember_mode,
                &sources,
            )?;
            anyhow::Ok((removed, new_data, capabilities))
        });

        match task.await {
            Ok(Ok((removed, new_data, capabilities))) => {
                *state_for_reload.borrow_mut() = new_data;
                set_storage_capabilities(&ui_for_reload, capabilities);
                render_views(
                    &state_for_reload.borrow(),
                    &ui_for_reload,
                    &state_for_reload,
                );
                refresh_menu(&ui_for_reload, &state_for_reload.borrow());
                show_verbose_status(
                    ui_for_reload.as_ref(),
                    format!(
                        "cache purge reload finished; removed={removed}; transactions={}; reports={}",
                        state_for_reload.borrow().transactions.len(),
                        state_for_reload.borrow().reports.len(),
                    ),
                );
                let base_message = if removed {
                    tr("Data and analytics cache cleared. Current data reloaded.")
                } else {
                    tr("No data and analytics cache was present. Current data reloaded.")
                };
                let message = status_with_cache(base_message, &state_for_reload.borrow());
                show_status(&ui_for_reload, &message);
            }
            Ok(Err(err)) => {
                show_verbose_status(
                    ui_for_reload.as_ref(),
                    format!("cache purge reload failed; error={err:#}"),
                );
                show_status(
                    &ui_for_reload,
                    &trf(
                        "Could not clear data and analytics cache: {error}",
                        &[("error", format!("{err:#}"))],
                    ),
                );
                render_views(
                    &state_for_reload.borrow(),
                    &ui_for_reload,
                    &state_for_reload,
                );
            }
            Err(_) => {
                show_verbose_status(ui_for_reload.as_ref(), "cache purge reload task canceled");
                show_status(
                    &ui_for_reload,
                    "Cache clear canceled: the background task stopped unexpectedly.",
                );
                render_views(
                    &state_for_reload.borrow(),
                    &ui_for_reload,
                    &state_for_reload,
                );
            }
        }
        finish_background_operation(ui_for_reload.as_ref());
    });
}

pub(in crate::app) fn reload_state_with_status(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    loading_message: &'static str,
    success_message: String,
    failure_message: &'static str,
    failure_replacements: Vec<(&'static str, String)>,
) {
    let scope = current_transaction_load_scope(&state.borrow(), ui.as_ref());
    reload_state_with_scope(
        state,
        ui,
        scope,
        loading_message,
        success_message,
        failure_message,
        failure_replacements,
    );
}

pub(in crate::app) fn reload_state_with_scope(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    scope: TransactionLoadScope,
    loading_message: &'static str,
    success_message: String,
    failure_message: &'static str,
    mut failure_replacements: Vec<(&'static str, String)>,
) {
    let borrowed = state.borrow();
    let mode = borrowed.dedupe_mode;
    let remember_mode = ui.remember_mode.get();
    let sources = current_sources_for_reload(&borrowed, remember_mode);
    drop(borrowed);
    let auto_clean_config = ui.preferences.auto_clean_config();
    let state_for_reload = Rc::clone(state);
    let ui_for_reload = Rc::clone(ui);
    show_verbose_status(
        ui.as_ref(),
        format!(
            "reload started; scope={scope:?}; remember={remember_mode:?}; sources={}; dedupe={mode:?}",
            sources.len(),
        ),
    );
    show_status(ui, loading_message);
    render_loading_placeholder(ui.as_ref());
    begin_background_operation(ui.as_ref());

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            data::load_app_data_with_sources(
                mode,
                auto_clean_config,
                scope,
                remember_mode,
                &sources,
            )
        });
        match task.await {
            Ok(Ok((new_data, capabilities))) => {
                *state_for_reload.borrow_mut() = new_data;
                set_storage_capabilities(&ui_for_reload, capabilities);
                render_views(
                    &state_for_reload.borrow(),
                    &ui_for_reload,
                    &state_for_reload,
                );
                refresh_menu(&ui_for_reload, &state_for_reload.borrow());
                show_verbose_status(
                    ui_for_reload.as_ref(),
                    format!(
                        "reload finished; transactions={}; reports={}",
                        state_for_reload.borrow().transactions.len(),
                        state_for_reload.borrow().reports.len(),
                    ),
                );
                let message = status_with_cache(success_message, &state_for_reload.borrow());
                show_status(&ui_for_reload, &message);
            }
            Ok(Err(err)) => {
                show_verbose_status(
                    ui_for_reload.as_ref(),
                    format!("reload failed; error={err:#}"),
                );
                failure_replacements.push(("error", format!("{err:#}")));
                show_status(&ui_for_reload, &trf(failure_message, &failure_replacements));
                render_views(
                    &state_for_reload.borrow(),
                    &ui_for_reload,
                    &state_for_reload,
                );
            }
            Err(_) => {
                show_verbose_status(ui_for_reload.as_ref(), "reload task canceled");
                show_status(
                    &ui_for_reload,
                    "Reload canceled: the background task stopped unexpectedly.",
                );
                render_views(
                    &state_for_reload.borrow(),
                    &ui_for_reload,
                    &state_for_reload,
                );
            }
        }
        finish_background_operation(ui_for_reload.as_ref());
    });
}
