use super::*;

pub(super) fn archive_configuration(
    ui_handles: Rc<UiHandles>,
    status: StatusHandle,
    restore_row: adw::ActionRow,
) {
    if !begin_configuration_task(
        &ui_handles,
        &status,
        "Backing up current configuration...",
        "configuration backup started",
    ) {
        return;
    }

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(data::archive_configuration);
        match task.await {
            Ok(Ok(path)) => {
                set_config_widget_base_sensitive(&ui_handles, &restore_row, true);
                let message = trf(
                    "Configuration backed up in {path}.",
                    &[("path", path.display().to_string())],
                );
                show_dialog_status_text(ui_handles.as_ref(), &status, &message);
                show_verbose_status(ui_handles.as_ref(), "configuration backup finished");
            }
            Ok(Err(error)) => {
                let message = trf(
                    "Could not back up configuration: {error}",
                    &[("error", format!("{error:#}"))],
                );
                show_dialog_status_text(ui_handles.as_ref(), &status, &message);
                show_verbose_status(
                    ui_handles.as_ref(),
                    format!("configuration backup failed; error={error:#}"),
                );
            }
            Err(_) => {
                show_dialog_status(
                    ui_handles.as_ref(),
                    &status,
                    "Configuration backup canceled: the background task stopped unexpectedly.",
                );
                show_verbose_status(ui_handles.as_ref(), "configuration backup task canceled");
            }
        }
        finish_configuration_task(&ui_handles, &status);
    });
}

pub(super) fn restore_configuration_archive(
    state: Rc<RefCell<AppData>>,
    ui_handles: Rc<UiHandles>,
    status: StatusHandle,
) {
    run_configuration_reload_task(
        state,
        ui_handles,
        status,
        ConfigurationTaskMessages {
            progress: "Restoring configuration backup...",
            success: "Configuration backup restored.",
            failure: "Could not restore configuration backup: {error}",
            canceled: "Configuration restore canceled: the background task stopped unexpectedly.",
        },
        || data::restore_configuration_archive().map(|_| ()),
    );
}

pub(super) fn restore_default_configuration(
    state: Rc<RefCell<AppData>>,
    ui_handles: Rc<UiHandles>,
    status: StatusHandle,
) {
    run_configuration_reload_task(
        state,
        ui_handles,
        status,
        ConfigurationTaskMessages {
            progress: "Applying default configuration...",
            success: "Default configuration applied.",
            failure: "Could not apply default configuration: {error}",
            canceled:
                "Applying default configuration canceled: the background task stopped unexpectedly.",
        },
        || data::restore_default_configuration().map(|_| ()),
    );
}

pub(super) fn restore_empty_configuration(
    state: Rc<RefCell<AppData>>,
    ui_handles: Rc<UiHandles>,
    status: StatusHandle,
) {
    run_configuration_reload_task(
        state,
        ui_handles,
        status,
        ConfigurationTaskMessages {
            progress: "Applying empty configuration...",
            success: "Empty configuration applied.",
            failure: "Could not apply empty configuration: {error}",
            canceled:
                "Applying empty configuration canceled: the background task stopped unexpectedly.",
        },
        || data::restore_empty_configuration().map(|_| ()),
    );
}

struct ConfigurationTaskMessages {
    progress: &'static str,
    success: &'static str,
    failure: &'static str,
    canceled: &'static str,
}

fn run_configuration_reload_task<F>(
    state: Rc<RefCell<AppData>>,
    ui_handles: Rc<UiHandles>,
    status: StatusHandle,
    messages: ConfigurationTaskMessages,
    operation: F,
) where
    F: FnOnce() -> anyhow::Result<()> + Send + 'static,
{
    let borrowed = state.borrow();
    let mode = borrowed.dedupe_mode;
    let remember_mode = ui_handles.remember_mode.get();
    let sources = current_sources_for_reload(&borrowed, remember_mode);
    let scope = current_transaction_load_scope(&borrowed, ui_handles.as_ref());
    drop(borrowed);
    if !begin_configuration_task(
        &ui_handles,
        &status,
        messages.progress,
        format!("configuration task started; progress={}", messages.progress),
    ) {
        return;
    }

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            operation()?;
            data::load_app_data_with_sources(mode, scope, remember_mode, &sources)
                .map(|loaded| loaded.0)
        });

        match task.await {
            Ok(Ok(data)) => {
                *state.borrow_mut() = data;
                render_views(&state.borrow(), &ui_handles, &state);
                show_dialog_status(ui_handles.as_ref(), &status, messages.success);
                show_verbose_status(
                    ui_handles.as_ref(),
                    format!("configuration task finished; success={}", messages.success),
                );
            }
            Ok(Err(error)) => {
                let message = trf(messages.failure, &[("error", format!("{error:#}"))]);
                show_dialog_status_text(ui_handles.as_ref(), &status, &message);
                show_verbose_status(
                    ui_handles.as_ref(),
                    format!("configuration task failed; error={error:#}"),
                );
            }
            Err(_) => {
                show_dialog_status(ui_handles.as_ref(), &status, messages.canceled);
                show_verbose_status(ui_handles.as_ref(), "configuration task canceled");
            }
        }
        finish_configuration_task(&ui_handles, &status);
    });
}

fn begin_configuration_task(
    ui_handles: &Rc<UiHandles>,
    status: &StatusHandle,
    progress_message: &str,
    debug_message: impl AsRef<str>,
) -> bool {
    if !try_begin_config_operation(ui_handles, CONFIGURATION_BUSY_MESSAGE) {
        status.set_text(&tr(CONFIGURATION_BUSY_MESSAGE));
        return false;
    }

    show_dialog_status(ui_handles.as_ref(), status, progress_message);
    show_verbose_status(ui_handles.as_ref(), debug_message);
    status.set_loading(true);
    begin_background_operation(ui_handles.as_ref());
    true
}

fn finish_configuration_task(ui_handles: &Rc<UiHandles>, status: &StatusHandle) {
    status.set_loading(false);
    finish_background_operation(ui_handles.as_ref());
    finish_config_operation(ui_handles);
}

fn show_dialog_status(ui_handles: &UiHandles, status: &StatusHandle, message: &str) {
    let message = tr(message);
    show_dialog_status_text(ui_handles, status, &message);
}

fn show_dialog_status_text(ui_handles: &UiHandles, status: &StatusHandle, message: &str) {
    status.set_text(message);
    show_status(ui_handles, message);
}
