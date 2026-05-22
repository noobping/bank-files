use super::*;
use adw::glib::variant::ToVariant;
use std::path::{Path, PathBuf};

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

        show_status(&ui_for_drop, "CSV drop received. Opening files...");

        let state_for_import = Rc::clone(&state_for_drop);
        let ui_for_import = Rc::clone(&ui_for_drop);
        gtk::glib::MainContext::default().spawn_local(async move {
            open_uris_in_background(uris, state_for_import, ui_for_import).await;
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
    show_status(&ui, "CSV files received. Opening files...");
    gtk::glib::MainContext::default().spawn_local(async move {
        open_uris_in_background(uris, state, ui).await;
    });
}

pub(in crate::app) async fn open_paths_in_background(
    files: Vec<PathBuf>,
    state: Rc<RefCell<AppData>>,
    ui: Rc<UiHandles>,
) {
    let mode = state.borrow().dedupe_mode;
    if should_copy_to_app_storage(ui.as_ref()) {
        import_and_reload_in_background(
            move || data::copy_files_to_app_storage(&files),
            mode,
            state,
            ui,
        )
        .await;
        return;
    }

    let (sources, skipped) = live_sources_from_paths(files);
    open_live_sources_in_background(sources, skipped, state, ui).await;
}

pub(in crate::app) async fn open_uris_in_background(
    uris: Vec<String>,
    state: Rc<RefCell<AppData>>,
    ui: Rc<UiHandles>,
) {
    let mode = state.borrow().dedupe_mode;
    if should_copy_to_app_storage(ui.as_ref()) {
        import_and_reload_in_background(
            move || data::copy_uris_to_app_storage(&uris),
            mode,
            state,
            ui,
        )
        .await;
        return;
    }

    if !ui.remember_mode.get().opens_live_files() && !ui.storage_capabilities.borrow().data_writable
    {
        show_status(
            &ui,
            "CSV storage is read-only. Opening local CSV files live for this session.",
        );
    }
    let (paths, unresolved) = local_paths_from_uris(&uris);
    let (sources, skipped) = live_sources_from_paths(paths);
    open_live_sources_in_background(sources, skipped + unresolved, state, ui).await;
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
    let remember_mode = ui.remember_mode.get();
    render_loading_placeholder(ui.as_ref());
    begin_background_operation(ui.as_ref());
    let task = gtk::gio::spawn_blocking(move || {
        let result = copy_files()?;
        let reload = if result.imported() > 0 {
            Some(
                data::load_app_data_with_sources(
                    mode,
                    auto_clean_config,
                    scope,
                    remember_mode,
                    &[],
                )
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
            let message = status_with_cache(import_status(result), &state.borrow());
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
                "No CSV files found. Only files with the .csv extension are opened.",
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
                &trf("Open CSV error: {error}", &[("error", format!("{err:#}"))]),
            );
            render_views(&state.borrow(), &ui, &state);
        }
        Err(_) => {
            show_status(
                &ui,
                "Open CSV canceled: the background task stopped unexpectedly.",
            );
            render_views(&state.borrow(), &ui, &state);
        }
    }
    finish_background_operation(ui.as_ref());
}

async fn open_live_sources_in_background(
    sources: Vec<TransactionSource>,
    skipped: usize,
    state: Rc<RefCell<AppData>>,
    ui: Rc<UiHandles>,
) {
    if sources.is_empty() {
        show_status(
            &ui,
            if skipped > 0 {
                "No local CSV files found. Live opening needs readable .csv files."
            } else {
                "No files chosen."
            },
        );
        render_views(&state.borrow(), &ui, &state);
        return;
    }

    let mode = state.borrow().dedupe_mode;
    let auto_clean_config = ui.preferences.auto_clean_config();
    let scope = current_transaction_load_scope(&state.borrow(), ui.as_ref());
    let remember_mode = ui.remember_mode.get();
    let sources = live_source_set(&state.borrow(), remember_mode, sources);
    render_loading_placeholder(ui.as_ref());
    begin_background_operation(ui.as_ref());
    let task_sources = sources.clone();
    let task = gtk::gio::spawn_blocking(move || {
        data::load_app_data_with_sources(
            mode,
            auto_clean_config,
            scope,
            remember_mode,
            &task_sources,
        )
    });

    match task.await {
        Ok(Ok((new_data, capabilities))) => {
            let opened = sources.iter().filter(|source| source.is_live()).count();
            *state.borrow_mut() = new_data;
            set_storage_capabilities(&ui, capabilities);
            render_views(&state.borrow(), &ui, &state);
            refresh_menu(&ui, &state.borrow());
            let mut message = trf(
                "{count} live transaction CSV file(s) opened for this session.",
                &[("count", opened.to_string())],
            );
            if skipped > 0 {
                message.push_str(&trf(
                    " {count} file(s) skipped because they were not local CSV files.",
                    &[("count", skipped.to_string())],
                ));
            }
            show_status(&ui, &status_with_cache(message, &state.borrow()));
        }
        Ok(Err(err)) => {
            show_status(
                &ui,
                &trf(
                    "Could not open live CSV files: {error}",
                    &[("error", format!("{err:#}"))],
                ),
            );
            render_views(&state.borrow(), &ui, &state);
        }
        Err(_) => {
            show_status(
                &ui,
                "Live CSV opening canceled: the background task stopped unexpectedly.",
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
            "{count} transaction CSV file(s) were opened and remembered.",
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
    let borrowed = state.borrow();
    let mode = borrowed.dedupe_mode;
    let remember_mode = ui.remember_mode.get();
    let sources = current_sources_for_reload(&borrowed, remember_mode);
    drop(borrowed);
    let auto_clean_config = ui.preferences.auto_clean_config();
    let state_for_reload = Rc::clone(state);
    let ui_for_reload = Rc::clone(ui);
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
                let message = status_with_cache(success_message, &state_for_reload.borrow());
                show_status(&ui_for_reload, &message);
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

pub(in crate::app) fn set_remember_mode(
    remember_mode: RememberMode,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    if ui.remember_mode.get() == remember_mode {
        ui.preferences.set_remember_mode(remember_mode);
        show_status(ui, remember_mode.description());
        return;
    }

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
        trf(
            "Remember is set to {mode}.",
            &[("mode", tr(remember_mode.label()))],
        ),
        "Could not apply Remember preference: {error}",
        Vec::new(),
    );
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
    let auto_clean_config = ui.preferences.auto_clean_config();
    let state_for_dedupe = Rc::clone(state);
    let ui_for_dedupe = Rc::clone(ui);
    show_status(ui, "Updating duplicate filtering...");
    begin_background_operation(ui.as_ref());
    action.set_enabled(false);

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

fn should_copy_to_app_storage(ui: &UiHandles) -> bool {
    !ui.remember_mode.get().opens_live_files() && ui.storage_capabilities.borrow().data_writable
}

fn live_sources_from_paths(files: Vec<PathBuf>) -> (Vec<TransactionSource>, usize) {
    let mut skipped = 0;
    let mut sources = Vec::new();
    for file in files {
        if file.is_file() && path_is_csv(&file) {
            sources.push(TransactionSource::live_file(file));
        } else {
            skipped += 1;
        }
    }
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    sources.dedup_by(|left, right| left.path == right.path);
    (sources, skipped)
}

fn local_paths_from_uris(uris: &[String]) -> (Vec<PathBuf>, usize) {
    let mut unresolved = 0;
    let mut paths = Vec::new();
    for uri in uris {
        let file = gtk::gio::File::for_uri(uri);
        if let Some(path) = file.path() {
            paths.push(path);
        } else {
            unresolved += 1;
        }
    }
    (paths, unresolved)
}

fn path_is_csv(path: &Path) -> bool {
    path.extension()
        .map(|extension| extension.eq_ignore_ascii_case("csv"))
        .unwrap_or(false)
}

fn live_source_set(
    data: &AppData,
    remember_mode: RememberMode,
    mut new_sources: Vec<TransactionSource>,
) -> Vec<TransactionSource> {
    if remember_mode.opens_live_files() {
        return new_sources;
    }

    let mut sources = data.transaction_sources.clone();
    sources.append(&mut new_sources);
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    sources.dedup_by(|left, right| left.path == right.path && left.kind == right.kind);
    sources
}

pub(in crate::app) fn current_sources_for_reload(
    data: &AppData,
    remember_mode: RememberMode,
) -> Vec<TransactionSource> {
    if remember_mode.opens_live_files()
        || data
            .transaction_sources
            .iter()
            .any(TransactionSource::is_live)
    {
        data.transaction_sources.clone()
    } else {
        Vec::new()
    }
}

fn status_with_cache(mut message: String, data: &AppData) -> String {
    match &data.cache_status {
        DataCacheStatus::Disabled | DataCacheStatus::Skipped => {}
        DataCacheStatus::Hit => message.push_str(&tr(" Loaded from the data and analytics cache.")),
        DataCacheStatus::Updated => message.push_str(&tr(" Data and analytics cache updated.")),
        DataCacheStatus::Failed(error) => message.push_str(&trf(
            " Data and analytics cache was skipped: {error}",
            &[("error", error.clone())],
        )),
    }
    message
}
