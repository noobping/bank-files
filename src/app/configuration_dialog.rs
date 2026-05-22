use super::*;

pub(in crate::app) fn show_configuration_dialog(
    parent: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let (header, search_button) = build_settings_header("Configuration");
    root.append(&header);

    let search_entry = gtk::SearchEntry::builder()
        .placeholder_text(tr("Search configuration"))
        .hexpand(true)
        .build();
    let search_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    search_box.set_margin_top(8);
    search_box.set_margin_bottom(8);
    search_box.set_margin_start(12);
    search_box.set_margin_end(12);
    search_box.append(&search_entry);
    let search_bar = gtk::SearchBar::builder()
        .child(&search_box)
        .show_close_button(true)
        .search_mode_enabled(false)
        .build();
    search_bar.connect_entry(&search_entry);
    root.append(&search_bar);

    let page = adw::PreferencesPage::builder()
        .title(tr("Configuration"))
        .icon_name("document-properties-symbolic")
        .build();
    let mut search_groups = Vec::new();

    let status_bar = build_status_bar();
    connect_embedded_status_bar(parent, &status_bar, Rc::clone(&ui_handles.status_autohide));
    status_bar
        .label
        .set_text(&tr("Configuration actions report progress here."));
    let status = status_bar.label.clone();

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

    let search_bar_for_button = search_bar.clone();
    let search_entry_for_button = search_entry.clone();
    search_button.connect_clicked(move |_| {
        let enabled = !search_bar_for_button.is_search_mode();
        search_bar_for_button.set_search_mode(enabled);
        if enabled {
            search_entry_for_button.grab_focus();
        }
    });
    search_bar.set_key_capture_widget(Some(&dialog));
    connect_preference_search(&search_entry, search_groups);

    dialog.present(Some(parent));
}

fn archive_group(
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    status: &gtk::Label,
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
    archive_row.connect_activated(move |_| {
        archive_configuration(
            Rc::clone(&ui_for_archive),
            status_for_archive.clone(),
            restore_row_for_archive.clone(),
        );
    });

    let state_for_restore = Rc::clone(state);
    let ui_for_restore = Rc::clone(ui_handles);
    let status_for_restore = status.clone();
    restore_row.connect_activated(move |_| {
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
    status: gtk::Label,
) -> (adw::PreferencesGroup, SearchablePreferencesGroup) {
    let title = "Automatic Configuration";
    let description =
        "Generate budgeting from imported transactions, use defaults, or start clean.";
    let group = adw::PreferencesGroup::builder()
        .title(tr(title))
        .description(tr(description))
        .build();
    let mut search_group = SearchablePreferencesGroup::new(&group, title, description);

    let generate_title = "Generate Budgets from Transactions";
    let generate_subtitle = "Replace the budget list with budgets detected from imported spending.";
    let generate_row = action_row("view-refresh-symbolic", generate_title, generate_subtitle);
    search_group.add_row(&generate_row, generate_title, generate_subtitle);
    group.add(&generate_row);

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
    register_config_widget(ui_handles, &generate_row);
    register_config_widget(ui_handles, &defaults_row);
    register_config_widget(ui_handles, &empty_row);

    let state_for_generate = Rc::clone(state);
    let ui_for_generate = Rc::clone(ui_handles);
    let status_for_generate = status.clone();
    generate_row.connect_activated(move |_| {
        generate_budgets_from_transactions_with_status(
            &state_for_generate,
            &ui_for_generate,
            Some(status_for_generate.clone()),
        );
    });

    let state_for_defaults = Rc::clone(state);
    let ui_for_defaults = Rc::clone(ui_handles);
    let status_for_defaults = status.clone();
    defaults_row.connect_activated(move |_| {
        restore_default_configuration(
            Rc::clone(&state_for_defaults),
            Rc::clone(&ui_for_defaults),
            status_for_defaults.clone(),
        );
    });

    let state_for_empty = Rc::clone(state);
    let ui_for_empty = Rc::clone(ui_handles);
    let status_for_empty = status.clone();
    empty_row.connect_activated(move |_| {
        restore_empty_configuration(
            Rc::clone(&state_for_empty),
            Rc::clone(&ui_for_empty),
            status_for_empty.clone(),
        );
    });

    (group, search_group)
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
    status: gtk::Label,
    restore_row: adw::ActionRow,
) {
    let busy_message = "Another edit or save is already running.";
    if !try_begin_config_operation(&ui_handles, busy_message) {
        status.set_text(&tr(busy_message));
        return;
    }
    show_dialog_status(
        ui_handles.as_ref(),
        &status,
        "Backing up current configuration...",
    );
    begin_background_operation(ui_handles.as_ref());

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
            }
            Ok(Err(error)) => {
                let message = trf(
                    "Could not back up configuration: {error}",
                    &[("error", format!("{error:#}"))],
                );
                show_dialog_status_text(ui_handles.as_ref(), &status, &message);
            }
            Err(_) => show_dialog_status(
                ui_handles.as_ref(),
                &status,
                "Configuration backup canceled: the background task stopped unexpectedly.",
            ),
        }
        finish_background_operation(ui_handles.as_ref());
        finish_config_operation(&ui_handles);
    });
}

fn restore_configuration_archive(
    state: Rc<RefCell<AppData>>,
    ui_handles: Rc<UiHandles>,
    status: gtk::Label,
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
    status: gtk::Label,
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
    status: gtk::Label,
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
    status: gtk::Label,
    messages: ConfigurationTaskMessages,
    operation: F,
) where
    F: FnOnce() -> anyhow::Result<()> + Send + 'static,
{
    let busy_message = "Another edit or save is already running.";
    if !try_begin_config_operation(&ui_handles, busy_message) {
        status.set_text(&tr(busy_message));
        return;
    }

    let mode = state.borrow().dedupe_mode;
    let auto_clean_config = ui_handles.preferences.auto_clean_config();
    let scope = current_transaction_load_scope(&state.borrow(), ui_handles.as_ref());
    show_dialog_status(ui_handles.as_ref(), &status, messages.progress);
    begin_background_operation(ui_handles.as_ref());

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            operation()?;
            data::load_app_data_with_config_cleanup(mode, auto_clean_config, scope)
        });

        match task.await {
            Ok(Ok(data)) => {
                *state.borrow_mut() = data;
                render_views(&state.borrow(), &ui_handles, &state);
                show_dialog_status(ui_handles.as_ref(), &status, messages.success);
            }
            Ok(Err(error)) => {
                let message = trf(messages.failure, &[("error", format!("{error:#}"))]);
                show_dialog_status_text(ui_handles.as_ref(), &status, &message);
            }
            Err(_) => show_dialog_status(ui_handles.as_ref(), &status, messages.canceled),
        }
        finish_background_operation(ui_handles.as_ref());
        finish_config_operation(&ui_handles);
    });
}

fn show_dialog_status(ui_handles: &UiHandles, status: &gtk::Label, message: &str) {
    let message = tr(message);
    show_dialog_status_text(ui_handles, status, &message);
}

fn show_dialog_status_text(ui_handles: &UiHandles, status: &gtk::Label, message: &str) {
    status.set_text(message);
    show_status(ui_handles, message);
}
