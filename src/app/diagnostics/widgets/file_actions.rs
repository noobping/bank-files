use super::*;

pub(super) fn force_reload_csv_file(
    source: &TransactionSource,
    name: &str,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    button: &gtk::Button,
) {
    if !csv_file_action_available(ui_handles.loading_count.get()) {
        show_status(ui_handles, "Data is still loading.");
        return;
    }

    let source = source.clone();
    let source_is_live = source.is_live();
    let name = name.to_string();
    let mode = state.borrow().dedupe_mode;
    let auto_clean_config = ui_handles.preferences.auto_clean_config();
    let remember_mode = ui_handles.remember_mode.get();
    let data = state.borrow().clone();
    let state_for_reload = Rc::clone(state);
    let ui_for_reload = Rc::clone(ui_handles);
    let button = button.clone();
    button.set_sensitive(false);
    show_status(&ui_for_reload, "Force reloading CSV file...");
    begin_background_operation(ui_for_reload.as_ref());

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            data::reload_transaction_source_file(
                data,
                &source,
                mode,
                auto_clean_config,
                remember_mode,
            )
        });

        match task.await {
            Ok(Ok(new_data)) => {
                *state_for_reload.borrow_mut() = new_data;
                set_storage_capabilities(&ui_for_reload, data::current_storage_capabilities());
                render_views(
                    &state_for_reload.borrow(),
                    &ui_for_reload,
                    &state_for_reload,
                );
                refresh_menu(&ui_for_reload, &state_for_reload.borrow());
                show_status(
                    &ui_for_reload,
                    &trf(
                        if source_is_live {
                            "{name} was reloaded from the live CSV file."
                        } else {
                            "{name} was reloaded from app storage."
                        },
                        &[("name", name.clone())],
                    ),
                );
            }
            Ok(Err(error)) => {
                show_status(
                    &ui_for_reload,
                    &trf(
                        "Could not reload {name}: {error}",
                        &[("name", name.clone()), ("error", format!("{error:#}"))],
                    ),
                );
                button.set_sensitive(true);
            }
            Err(_) => {
                show_status(
                    &ui_for_reload,
                    "CSV reload canceled: the background task stopped unexpectedly.",
                );
                button.set_sensitive(true);
            }
        }
        finish_background_operation(ui_for_reload.as_ref());
    });
}

pub(super) fn forget_or_unload_csv_file(
    source: &TransactionSource,
    name: &str,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    button: &gtk::Button,
) {
    if !csv_file_action_available(ui_handles.loading_count.get()) {
        show_status(ui_handles, "Data is still loading.");
        return;
    }
    if !source.is_live() && !ui_handles.storage_capabilities.borrow().data_writable {
        show_status(
            ui_handles,
            ui_handles.storage_capabilities.borrow().data_write_reason(),
        );
        return;
    }

    let source = source.clone();
    let source_is_live = source.is_live();
    let name = name.to_string();
    let mode = state.borrow().dedupe_mode;
    let auto_clean_config = ui_handles.preferences.auto_clean_config();
    let remember_mode = ui_handles.remember_mode.get();
    let mut sources = state.borrow().transaction_sources.clone();
    sources.retain(|existing| existing.path != source.path || existing.kind != source.kind);
    let scope = current_transaction_load_scope(&state.borrow(), ui_handles.as_ref());
    let state_for_unload = Rc::clone(state);
    let ui_for_unload = Rc::clone(ui_handles);
    let button = button.clone();
    button.set_sensitive(false);
    show_status(
        &ui_for_unload,
        "CSV removed from this session. Updating the overview...",
    );

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            if !source.is_live() {
                data::remove_inbox_file(source.path())?;
            }
            let reload_sources = if sources.iter().any(TransactionSource::is_live)
                || remember_mode.opens_live_files()
            {
                sources
            } else {
                Vec::new()
            };
            data::load_app_data_with_sources(
                mode,
                auto_clean_config,
                scope,
                remember_mode,
                &reload_sources,
            )
        });

        match task.await {
            Ok(Ok((new_data, capabilities))) => {
                *state_for_unload.borrow_mut() = new_data;
                set_storage_capabilities(&ui_for_unload, capabilities);
                request_render_views(&ui_for_unload, &state_for_unload);
                refresh_menu(&ui_for_unload, &state_for_unload.borrow());
                show_status(
                    &ui_for_unload,
                    &trf(
                        if source_is_live {
                            "{name} was forgotten for this session. The CSV file was not changed."
                        } else {
                            "{name} was unloaded. The original CSV remains where you chose it."
                        },
                        &[("name", name.clone())],
                    ),
                );
            }
            Ok(Err(err)) => {
                show_status(
                    &ui_for_unload,
                    &trf(
                        "Could not remove {name}: {error}",
                        &[("name", name.clone()), ("error", format!("{err:#}"))],
                    ),
                );
                button.set_sensitive(true);
            }
            Err(_) => {
                show_status(
                    &ui_for_unload,
                    "CSV removal canceled: the background task stopped unexpectedly.",
                );
                button.set_sensitive(true);
            }
        }
    });
}

pub(super) fn transaction_source_for_report(
    report: &ImportReport,
    data: &AppData,
) -> TransactionSource {
    data.transaction_sources
        .iter()
        .find(|source| source.path == report.source)
        .cloned()
        .unwrap_or_else(|| TransactionSource::inbox_file(report.source.clone()))
}

pub(super) fn csv_file_action_available(loading_count: u32) -> bool {
    loading_count == 0
}
