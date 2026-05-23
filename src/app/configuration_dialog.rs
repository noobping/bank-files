use super::*;

const CONFIGURATION_BUSY_MESSAGE: &str = "Another edit or save is already running.";

pub(in crate::app) fn show_configuration_dialog(
    parent: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let shell = build_settings_dialog_shell("Configuration", "Search configuration");
    let root = shell.root;
    let search_button = shell.search_button;
    let search_bar = shell.search_bar;
    let search_entry = shell.search_entry;

    let page = adw::PreferencesPage::builder()
        .title(tr("Configuration"))
        .icon_name("document-properties-symbolic")
        .build();
    let mut search_groups = Vec::new();

    let status_bar = build_status_bar();
    connect_embedded_status_bar(parent, &status_bar, Rc::clone(&ui_handles.status_autohide));
    connect_static_page_actions(
        &status_bar.page_actions_button,
        "configuration",
        &status_bar.label,
        ui_handles,
        configuration_page_snapshot(
            ui_handles.advanced_features.get(),
            ui_handles.show_predictions.get(),
        ),
    );
    let status = StatusHandle::from_status_bar(&status_bar);
    status.set_text(&tr("Configuration actions report progress here."));

    let (group, search_group) = archive_group(state, ui_handles, &status);
    page.add(&group);
    search_groups.push(search_group);

    let (group, search_group) = automatic_configuration_group(state, ui_handles, status.clone());
    page.add(&group);
    search_groups.push(search_group);

    root.append(&ui::scroll(&page));
    root.append(&status_bar.container);

    let dialog = adw::Dialog::builder()
        .title(tr("Configuration"))
        .content_width(720)
        .content_height(620)
        .child(&root)
        .build();

    ui::connect_search_button(&search_button, &search_bar, &search_entry);
    ui::connect_search_shortcut(&dialog, &search_bar, &search_entry);
    search_bar.set_key_capture_widget(Some(&dialog));
    connect_preference_search(&search_entry, search_groups);

    dialog.present(Some(parent));
}

fn configuration_page_snapshot(
    advanced_features: bool,
    smart_insights_enabled: bool,
) -> StaticPageSnapshot {
    StaticPageSnapshot::new(
        "configuration",
        "Configuration",
        "Configuration actions report progress here.",
        &["Group", "Action", "Description"],
        configuration_snapshot_rows(advanced_features, smart_insights_enabled),
    )
}

fn configuration_snapshot_rows(
    advanced_features: bool,
    smart_insights_enabled: bool,
) -> Vec<Vec<String>> {
    let mut rows = vec![
        vec![
            tr("Configuration Backup"),
            tr("Back Up Current Configuration"),
            tr("Replace the existing backup in the config folder."),
        ],
        vec![
            tr("Configuration Backup"),
            tr("Restore Configuration Backup"),
            tr("Restore rules, budgets, and field names from the backup."),
        ],
    ];

    if automatic_configuration_generation_visible(advanced_features, smart_insights_enabled) {
        rows.push(vec![
            tr("Automatic Configuration"),
            tr("Generate Configuration from Transactions"),
            tr(automatic_configuration_generation_subtitle(
                advanced_features,
                smart_insights_enabled,
            )),
        ]);
    }

    rows.extend([
        vec![
            tr("Automatic Configuration"),
            tr("Use Default Configuration"),
            tr("Replace rules, budgets, and field names with the built-in defaults."),
        ],
        vec![
            tr("Automatic Configuration"),
            tr("Use Empty Configuration"),
            tr("Remove all rules and budget codes while keeping CSV field names for imports."),
        ],
    ]);
    rows
}

fn archive_group(
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    status: &StatusHandle,
) -> (adw::PreferencesGroup, SearchablePreferencesGroup) {
    let title = "Configuration Backup";
    let description = "Back up or restore the current rules, budgets, and CSV field names.";
    let group = adw::PreferencesGroup::builder()
        .title(tr(title))
        .description(tr(description))
        .build();
    let mut search_group = SearchablePreferencesGroup::new(&group, title, description);

    let archive_title = "Back Up Current Configuration";
    let archive_subtitle = "Replace the existing backup in the config folder.";
    let archive_row = action_row("document-save-symbolic", archive_title, archive_subtitle);
    search_group.add_row(&archive_row, archive_title, archive_subtitle);
    group.add(&archive_row);

    let restore_title = "Restore Configuration Backup";
    let restore_subtitle = "Restore rules, budgets, and field names from the backup.";
    let restore_row = action_row("document-revert-symbolic", restore_title, restore_subtitle);
    restore_row.set_sensitive(data::configuration_archive_exists().unwrap_or(false));
    search_group.add_row(&restore_row, restore_title, restore_subtitle);
    group.add(&restore_row);
    register_config_widget(ui_handles, &archive_row);
    register_config_widget(ui_handles, &restore_row);

    let ui_for_archive = Rc::clone(ui_handles);
    let status_for_archive = status.clone();
    let restore_row_for_archive = restore_row.clone();
    archive_row.connect_activated(move |row| {
        if !row.is_sensitive() {
            return;
        }
        archive_configuration(
            Rc::clone(&ui_for_archive),
            status_for_archive.clone(),
            restore_row_for_archive.clone(),
        );
    });

    let state_for_restore = Rc::clone(state);
    let ui_for_restore = Rc::clone(ui_handles);
    let status_for_restore = status.clone();
    restore_row.connect_activated(move |row| {
        if !row.is_sensitive() {
            return;
        }
        restore_configuration_archive(
            Rc::clone(&state_for_restore),
            Rc::clone(&ui_for_restore),
            status_for_restore.clone(),
        );
    });

    (group, search_group)
}

fn automatic_configuration_group(
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    status: StatusHandle,
) -> (adw::PreferencesGroup, SearchablePreferencesGroup) {
    let title = "Automatic Configuration";
    let description =
        "Generate configuration from imported transactions, use defaults, or start clean.";
    let group = adw::PreferencesGroup::builder()
        .title(tr(title))
        .description(tr(description))
        .build();
    let mut search_group = SearchablePreferencesGroup::new(&group, title, description);
    let advanced_features = ui_handles.advanced_features.get();
    let smart_insights_enabled = ui_handles.show_predictions.get();

    if automatic_configuration_generation_visible(advanced_features, smart_insights_enabled) {
        let generate_title = "Generate Configuration from Transactions";
        let generate_subtitle =
            automatic_configuration_generation_subtitle(advanced_features, smart_insights_enabled);
        let generate_row = action_row("view-refresh-symbolic", generate_title, generate_subtitle);
        if !automatic_configuration_generation_enabled(smart_insights_enabled) {
            generate_row.set_sensitive(false);
            generate_row.set_tooltip_text(Some(&tr(
                "Enable Smart Insights to generate configuration from transactions.",
            )));
        }
        search_group.add_row(&generate_row, generate_title, generate_subtitle);
        group.add(&generate_row);
        register_config_widget(ui_handles, &generate_row);

        let state_for_generate = Rc::clone(state);
        let ui_for_generate = Rc::clone(ui_handles);
        let status_for_generate = status.clone();
        generate_row.connect_activated(move |row| {
            if !row.is_sensitive() {
                return;
            }
            generate_configuration_from_transactions_with_status(
                &state_for_generate,
                &ui_for_generate,
                Some(status_for_generate.clone()),
            );
        });
    }

    let defaults_title = "Use Default Configuration";
    let defaults_subtitle = "Replace rules, budgets, and field names with the built-in defaults.";
    let defaults_row = action_row(
        "document-revert-symbolic",
        defaults_title,
        defaults_subtitle,
    );
    search_group.add_row(&defaults_row, defaults_title, defaults_subtitle);
    group.add(&defaults_row);

    let empty_title = "Use Empty Configuration";
    let empty_subtitle =
        "Remove all rules and budget codes while keeping CSV field names for imports.";
    let empty_row = action_row("edit-clear-symbolic", empty_title, empty_subtitle);
    search_group.add_row(&empty_row, empty_title, empty_subtitle);
    group.add(&empty_row);
    register_config_widget(ui_handles, &defaults_row);
    register_config_widget(ui_handles, &empty_row);

    let state_for_defaults = Rc::clone(state);
    let ui_for_defaults = Rc::clone(ui_handles);
    let status_for_defaults = status.clone();
    defaults_row.connect_activated(move |row| {
        if !row.is_sensitive() {
            return;
        }
        restore_default_configuration(
            Rc::clone(&state_for_defaults),
            Rc::clone(&ui_for_defaults),
            status_for_defaults.clone(),
        );
    });

    let state_for_empty = Rc::clone(state);
    let ui_for_empty = Rc::clone(ui_handles);
    let status_for_empty = status.clone();
    empty_row.connect_activated(move |row| {
        if !row.is_sensitive() {
            return;
        }
        restore_empty_configuration(
            Rc::clone(&state_for_empty),
            Rc::clone(&ui_for_empty),
            status_for_empty.clone(),
        );
    });

    (group, search_group)
}

fn automatic_configuration_generation_visible(
    advanced_features: bool,
    smart_insights_enabled: bool,
) -> bool {
    smart_insights_enabled || advanced_features
}

fn automatic_configuration_generation_enabled(smart_insights_enabled: bool) -> bool {
    smart_pattern_detection_enabled(smart_insights_enabled)
}

fn automatic_configuration_generation_subtitle(
    advanced_features: bool,
    smart_insights_enabled: bool,
) -> &'static str {
    if !smart_insights_enabled {
        "Requires Smart Insights. Enable Smart Insights to generate configuration from transactions."
    } else if advanced_features {
        "Create rules, budget codes, field mappings, and hidden refund/split patterns from imported transactions."
    } else {
        "Create a working setup from imported transactions and hide refund/split patterns."
    }
}

fn action_row(icon_name: &str, title: &str, subtitle: &str) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(tr(title))
        .subtitle(tr(subtitle))
        .build();
    row.set_activatable(true);
    row.add_prefix(&gtk::Image::from_icon_name(icon_name));
    row
}

fn archive_configuration(
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

fn restore_configuration_archive(
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

fn restore_default_configuration(
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

fn restore_empty_configuration(
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
    let auto_clean_config = ui_handles.preferences.auto_clean_config();
    let smart_insights_enabled = ui_handles.show_predictions.get();
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
            data::load_app_data_with_sources(
                mode,
                auto_clean_config,
                scope,
                remember_mode,
                &sources,
                smart_insights_enabled,
            )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_configuration_generation_hides_in_simple_mode_without_smart_insights() {
        assert!(!automatic_configuration_generation_visible(false, false));
    }

    #[test]
    fn automatic_configuration_generation_shows_disabled_in_advanced_mode_without_smart_insights() {
        assert!(automatic_configuration_generation_visible(true, false));
        assert!(!automatic_configuration_generation_enabled(false));
    }

    #[test]
    fn configuration_snapshot_follows_generation_visibility() {
        let simple_rows = configuration_snapshot_rows(false, false);
        let advanced_rows = configuration_snapshot_rows(true, false);

        assert_eq!(simple_rows.len(), 4);
        assert_eq!(advanced_rows.len(), 5);
    }
}
