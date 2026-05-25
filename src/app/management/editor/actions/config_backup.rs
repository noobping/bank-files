use super::*;

#[derive(Clone)]
struct ConfigBackupActions {
    back_up: gtk::gio::SimpleAction,
    restore_latest: gtk::gio::SimpleAction,
}

pub(super) fn connect_config_backup_actions(actions: &ManagementDialogActions<'_>) {
    let config_actions = ConfigBackupActions {
        back_up: actions.back_up_configuration_action.clone(),
        restore_latest: actions.restore_latest_backup_action.clone(),
    };
    connect_back_up_configuration_action(actions, &config_actions);
    connect_restore_latest_backup_action(actions, &config_actions);
}

fn connect_back_up_configuration_action(
    actions: &ManagementDialogActions<'_>,
    config_actions: &ConfigBackupActions,
) {
    let status = actions.status.clone();
    let ui_handles = Rc::clone(actions.ui_handles);
    let dialog_closed = Rc::clone(&actions.dialog_closed);
    let save_running = Rc::clone(&actions.save_running);
    let finish_management_dialog = Rc::clone(&actions.finish_management_dialog);
    let config_actions_for_backup = config_actions.clone();

    config_actions.back_up.connect_activate(move |_, _| {
        set_config_backup_actions_enabled(&config_actions_for_backup, false);
        save_running.set(true);
        status.set_text(&tr("Backing up current configuration..."));
        show_status(&ui_handles, "Backing up current configuration...");

        let status = status.clone();
        let ui_handles = Rc::clone(&ui_handles);
        let dialog_closed = Rc::clone(&dialog_closed);
        let save_running = Rc::clone(&save_running);
        let finish_management_dialog = Rc::clone(&finish_management_dialog);
        let config_actions = config_actions_for_backup.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            let task = gtk::gio::spawn_blocking(data::archive_configuration);
            match task.await {
                Ok(Ok(path)) => {
                    config_actions.restore_latest.set_enabled(true);
                    let message = trf(
                        "Configuration backed up in {path}.",
                        &[("path", path.display().to_string())],
                    );
                    status.set_text(&message);
                    show_status(&ui_handles, &message);
                }
                Ok(Err(error)) => {
                    let message = trf(
                        "Could not back up configuration: {error}",
                        &[("error", format!("{error:#}"))],
                    );
                    status.set_text(&message);
                    show_status(&ui_handles, &message);
                }
                Err(_) => {
                    let message = tr(
                        "Configuration backup canceled: the background task stopped unexpectedly.",
                    );
                    status.set_text(&message);
                    show_status(&ui_handles, &message);
                }
            }
            save_running.set(false);
            if dialog_closed.get() {
                finish_management_dialog();
            } else {
                config_actions.back_up.set_enabled(true);
                set_restore_action_availability(&config_actions.restore_latest);
            }
        });
    });
}

fn connect_restore_latest_backup_action(
    actions: &ManagementDialogActions<'_>,
    config_actions: &ConfigBackupActions,
) {
    let management_dialog = actions.management_dialog.clone();
    let status = actions.status.clone();
    let state = Rc::clone(actions.state);
    let ui_handles = Rc::clone(actions.ui_handles);
    let dialog_closed = Rc::clone(&actions.dialog_closed);
    let save_running = Rc::clone(&actions.save_running);
    let finish_management_dialog = Rc::clone(&actions.finish_management_dialog);
    let config_actions_for_restore = config_actions.clone();

    config_actions.restore_latest.connect_activate(move |_, _| {
        let borrowed = state.borrow();
        let mode = borrowed.dedupe_mode;
        let remember_mode = ui_handles.remember_mode.get();
        let sources = current_sources_for_reload(&borrowed, remember_mode);
        let scope = current_transaction_load_scope(&borrowed, ui_handles.as_ref());
        drop(borrowed);

        set_config_backup_actions_enabled(&config_actions_for_restore, false);
        save_running.set(true);
        status.set_text(&tr("Restoring configuration backup..."));
        show_status(&ui_handles, "Restoring configuration backup...");

        let management_dialog = management_dialog.clone();
        let status = status.clone();
        let state = Rc::clone(&state);
        let ui_handles = Rc::clone(&ui_handles);
        let dialog_closed = Rc::clone(&dialog_closed);
        let save_running = Rc::clone(&save_running);
        let finish_management_dialog = Rc::clone(&finish_management_dialog);
        let config_actions = config_actions_for_restore.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            let task = gtk::gio::spawn_blocking(move || {
                data::restore_configuration_archive()?;
                data::load_app_data_with_sources(mode, scope, remember_mode, &sources)
                    .map(|loaded| loaded.0)
            });

            match task.await {
                Ok(Ok(new_data)) => {
                    *state.borrow_mut() = new_data;
                    render_views(&state.borrow(), &ui_handles, &state);
                    let message = tr("Configuration backup restored.");
                    status.set_text(&message);
                    show_status(&ui_handles, &message);
                    save_running.set(false);
                    if dialog_closed.get() {
                        finish_management_dialog();
                    } else {
                        management_dialog.close();
                    }
                }
                Ok(Err(error)) => {
                    let message = trf(
                        "Could not restore configuration backup: {error}",
                        &[("error", format!("{error:#}"))],
                    );
                    finish_restore_after_error(
                        &status,
                        &ui_handles,
                        &dialog_closed,
                        &save_running,
                        &finish_management_dialog,
                        &config_actions,
                        &message,
                    );
                }
                Err(_) => {
                    let message = tr(
                        "Configuration restore canceled: the background task stopped unexpectedly.",
                    );
                    finish_restore_after_error(
                        &status,
                        &ui_handles,
                        &dialog_closed,
                        &save_running,
                        &finish_management_dialog,
                        &config_actions,
                        &message,
                    );
                }
            }
        });
    });
}

fn finish_restore_after_error(
    status: &gtk::Label,
    ui_handles: &Rc<UiHandles>,
    dialog_closed: &Rc<Cell<bool>>,
    save_running: &Rc<Cell<bool>>,
    finish_management_dialog: &Rc<dyn Fn()>,
    config_actions: &ConfigBackupActions,
    message: &str,
) {
    status.set_text(message);
    show_status(ui_handles, message);
    save_running.set(false);
    if dialog_closed.get() {
        finish_management_dialog();
    } else {
        set_config_backup_actions_enabled(config_actions, true);
        set_restore_action_availability(&config_actions.restore_latest);
    }
}

fn set_config_backup_actions_enabled(actions: &ConfigBackupActions, enabled: bool) {
    actions.back_up.set_enabled(enabled);
    actions.restore_latest.set_enabled(enabled);
}

fn set_restore_action_availability(action: &gtk::gio::SimpleAction) {
    action.set_enabled(data::configuration_archive_exists().unwrap_or(false));
}
