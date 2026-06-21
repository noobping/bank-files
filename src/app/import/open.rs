use super::sources::{
    live_source_set, live_sources_from_paths, local_paths_from_uris, should_copy_to_app_storage,
};
use super::status::{import_status, status_with_cache};
use super::*;
use std::path::PathBuf;

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

pub(super) async fn open_uris_in_background(
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
            "Bank file storage is read-only. Opening local files live for this session.",
        );
    }
    let (paths, unresolved) = local_paths_from_uris(&uris);
    let (sources, skipped) = live_sources_from_paths(paths);
    open_live_sources_in_background(sources, skipped + unresolved, state, ui).await;
}

pub(super) async fn import_and_reload_in_background<F>(
    copy_files: F,
    mode: DedupeMode,
    state: Rc<RefCell<AppData>>,
    ui: Rc<UiHandles>,
) where
    F: FnOnce() -> anyhow::Result<data::CsvCopyResult> + Send + 'static,
{
    let scope = current_transaction_load_scope(&state.borrow(), ui.as_ref());
    let remember_mode = ui.remember_mode.get();
    render_loading_placeholder(ui.as_ref());
    begin_background_operation(ui.as_ref());
    let task = gtk::gio::spawn_blocking(move || {
        let result = copy_files()?;
        let reload = if result.imported() > 0 {
            Some(
                data::load_app_data_with_sources(mode, scope, remember_mode, &[])
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
                    "{count} bank file(s) copied, but reload failed: {error}",
                    &[("count", result.imported().to_string()), ("error", err)],
                ),
            );
            render_views(&state.borrow(), &ui, &state);
        }
        Ok(Ok((result, None))) if result.skipped > 0 => {
            show_status(
                &ui,
                "No supported bank files found. Open CSV, Excel, or Calc files.",
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
                &trf(
                    "Open bank file error: {error}",
                    &[("error", format!("{err:#}"))],
                ),
            );
            render_views(&state.borrow(), &ui, &state);
        }
        Err(_) => {
            show_status(
                &ui,
                "Bank file opening canceled: the background task stopped unexpectedly.",
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
                "No local bank files found. Live opening needs readable CSV, Excel, or Calc files."
            } else {
                "No files chosen."
            },
        );
        render_views(&state.borrow(), &ui, &state);
        return;
    }

    let mode = state.borrow().dedupe_mode;
    let scope = current_transaction_load_scope(&state.borrow(), ui.as_ref());
    let remember_mode = ui.remember_mode.get();
    let sources = live_source_set(&state.borrow(), remember_mode, sources);
    render_loading_placeholder(ui.as_ref());
    begin_background_operation(ui.as_ref());
    let task_sources = sources.clone();
    let task = gtk::gio::spawn_blocking(move || {
        data::load_app_data_with_sources(mode, scope, remember_mode, &task_sources)
    });

    match task.await {
        Ok(Ok((new_data, capabilities))) => {
            let opened = sources.iter().filter(|source| source.is_live()).count();
            *state.borrow_mut() = new_data;
            set_storage_capabilities(&ui, capabilities);
            render_views(&state.borrow(), &ui, &state);
            refresh_menu(&ui, &state.borrow());
            let mut message = trf(
                "{count} live transaction file(s) opened for this session.",
                &[("count", opened.to_string())],
            );
            if skipped > 0 {
                message.push_str(&trf(
                    " {count} file(s) skipped because they were not local bank files.",
                    &[("count", skipped.to_string())],
                ));
            }
            show_status(&ui, &status_with_cache(message, &state.borrow()));
        }
        Ok(Err(err)) => {
            show_status(
                &ui,
                &trf(
                    "Could not open live bank files: {error}",
                    &[("error", format!("{err:#}"))],
                ),
            );
            render_views(&state.borrow(), &ui, &state);
        }
        Err(_) => {
            show_status(
                &ui,
                "Live bank file opening canceled: the background task stopped unexpectedly.",
            );
            render_views(&state.borrow(), &ui, &state);
        }
    }
    finish_background_operation(ui.as_ref());
}
