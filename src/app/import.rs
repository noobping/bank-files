use super::*;
use adw::glib::variant::ToVariant;

pub(in crate::app) fn connect_drop_target(
    root: &gtk::Box,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let target = gtk::DropTarget::new(
        gtk::gdk::FileList::static_type(),
        gtk::gdk::DragAction::COPY,
    );
    let state_for_drop = Rc::clone(state);
    let ui_for_drop = Rc::clone(ui);
    target.connect_drop(move |_, value, _, _| {
        if !ui_for_drop.storage_capabilities.borrow().data_writable {
            show_status(
                &ui_for_drop,
                ui_for_drop
                    .storage_capabilities
                    .borrow()
                    .data_write_reason(),
            );
            return true;
        }

        let Ok(file_list) = value.get::<gtk::gdk::FileList>() else {
            show_status(&ui_for_drop, "Drop contains no readable files.");
            return false;
        };
        let uris = file_list
            .files()
            .iter()
            .map(|file| file.uri().to_string())
            .collect::<Vec<_>>();
        if uris.is_empty() {
            show_status(&ui_for_drop, "Drop contains no files.");
            return true;
        }

        show_status(&ui_for_drop, "CSV drop received. Import is starting...");

        let state_for_import = Rc::clone(&state_for_drop);
        let ui_for_import = Rc::clone(&ui_for_drop);
        let mode = state_for_import.borrow().dedupe_mode;
        gtk::glib::MainContext::default().spawn_local(async move {
            import_and_reload_in_background(
                move || data::copy_uris_to_app_storage(&uris),
                mode,
                state_for_import,
                ui_for_import,
            )
            .await;
        });
        true
    });

    root.add_controller(target);
}

pub(in crate::app) fn import_uris_into_session(
    uris: Vec<String>,
    state: Rc<RefCell<AppData>>,
    ui: Rc<UiHandles>,
) {
    if !ui.storage_capabilities.borrow().data_writable {
        show_status(&ui, ui.storage_capabilities.borrow().data_write_reason());
        reload_state_with_status(
            &state,
            &ui,
            "Loading saved CSV files...",
            tr("Saved CSV files loaded."),
            "Reload error: {error}",
            Vec::new(),
        );
        return;
    }
    show_status(&ui, "CSV files received. Import is starting...");
    let mode = state.borrow().dedupe_mode;
    gtk::glib::MainContext::default().spawn_local(async move {
        import_and_reload_in_background(
            move || data::copy_uris_to_app_storage(&uris),
            mode,
            state,
            ui,
        )
        .await;
    });
}

pub(in crate::app) async fn import_and_reload_in_background<F>(
    copy_files: F,
    mode: DedupeMode,
    state: Rc<RefCell<AppData>>,
    ui: Rc<UiHandles>,
) where
    F: FnOnce() -> anyhow::Result<data::CsvCopyResult> + Send + 'static,
{
    let auto_clean_config = ui.preferences.auto_clean_config();
    let scope = current_transaction_load_scope(&state.borrow(), ui.as_ref());
    render_loading_placeholder(ui.as_ref());
    begin_background_operation(ui.as_ref());
    let task = gtk::gio::spawn_blocking(move || {
        let result = copy_files()?;
        let reload = if result.imported() > 0 {
            Some(
                data::load_app_data_read_only_aware(mode, auto_clean_config, scope)
                    .map_err(|err| format!("{err:#}")),
            )
        } else {
            None
        };
        anyhow::Ok((result, reload))
    });

    match task.await {
        Ok(Ok((result, Some(Ok((new_data, capabilities)))))) => {
            *state.borrow_mut() = new_data;
            set_storage_capabilities(&ui, capabilities);
            render_views(&state.borrow(), &ui, &state);
            refresh_menu(&ui, &state.borrow());
            let message = import_status(result);
            show_status(&ui, &message);
        }
        Ok(Ok((result, Some(Err(err))))) => {
            show_status(
                &ui,
                &trf(
                    "{count} CSV file(s) copied, but reload failed: {error}",
                    &[("count", result.imported().to_string()), ("error", err)],
                ),
            );
            render_views(&state.borrow(), &ui, &state);
        }
        Ok(Ok((result, None))) if result.skipped > 0 => {
            show_status(
                &ui,
                "No CSV files found. Only files with the .csv extension are imported.",
            );
            render_views(&state.borrow(), &ui, &state);
        }
        Ok(Ok((_result, None))) => {
            show_status(&ui, "No files chosen.");
            render_views(&state.borrow(), &ui, &state);
        }
        Ok(Err(err)) => {
            show_status(
                &ui,
                &trf("Import error: {error}", &[("error", format!("{err:#}"))]),
            );
            render_views(&state.borrow(), &ui, &state);
        }
        Err(_) => {
            show_status(
                &ui,
                "Import canceled: the background task stopped unexpectedly.",
            );
            render_views(&state.borrow(), &ui, &state);
        }
    }
    finish_background_operation(ui.as_ref());
}

pub(in crate::app) fn import_status(result: data::CsvCopyResult) -> String {
    let mut message = match (result.transaction_csvs, result.config_csvs) {
        (transactions, configs) if transactions > 0 && configs > 0 => trf(
            "{transactions} transaction CSV file(s) and {configs} configuration CSV file(s) were opened and applied.",
            &[
                ("transactions", transactions.to_string()),
                ("configs", configs.to_string()),
            ],
        ),
        (transactions, _) if transactions > 0 => trf(
            "{count} transaction CSV file(s) were opened and imported.",
            &[("count", transactions.to_string())],
        ),
        (_, configs) if configs > 0 => trf(
            "{count} configuration CSV file(s) were opened and applied.",
            &[("count", configs.to_string())],
        ),
        _ => tr("No CSV files were opened."),
    };
    if result.skipped > 0 {
        message.push_str(&trf(
            " {count} file(s) skipped because they were not CSV files.",
            &[("count", result.skipped.to_string())],
        ));
    }
    message
}

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
    let mode = state.borrow().dedupe_mode;
    let auto_clean_config = ui.preferences.auto_clean_config();
    let state_for_reload = Rc::clone(state);
    let ui_for_reload = Rc::clone(ui);
    show_status(ui, loading_message);
    render_loading_placeholder(ui.as_ref());
    begin_background_operation(ui.as_ref());

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            data::load_app_data_read_only_aware(mode, auto_clean_config, scope)
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
                show_status(&ui_for_reload, &success_message);
            }
            Ok(Err(err)) => {
                failure_replacements.push(("error", format!("{err:#}")));
                show_status(&ui_for_reload, &trf(failure_message, &failure_replacements));
                render_views(
                    &state_for_reload.borrow(),
                    &ui_for_reload,
                    &state_for_reload,
                );
            }
            Err(_) => {
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

    let auto_clean_config = ui.preferences.auto_clean_config();
    let scope = current_transaction_load_scope(&state.borrow(), ui.as_ref());
    let state_for_dedupe = Rc::clone(state);
    let ui_for_dedupe = Rc::clone(ui);
    show_status(ui, "Updating duplicate filtering...");
    begin_background_operation(ui.as_ref());
    action.set_enabled(false);

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            data::load_app_data_read_only_aware(mode, auto_clean_config, scope)
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
                show_status(
                    &ui_for_dedupe,
                    &trf(
                        "Duplicate filtering is {state}. {description}",
                        &[
                            ("state", tr(mode.label())),
                            ("description", tr(mode.description())),
                        ],
                    ),
                );
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
