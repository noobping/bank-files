use super::tasks::{
    archive_configuration, remove_configuration_archive, restore_configuration_archive,
    restore_configuration_archive_by_id, restore_default_configuration,
    restore_empty_configuration,
};
use super::*;

type BackupRefresh = Rc<dyn Fn()>;
type BackupRefreshSlot = Rc<RefCell<Option<BackupRefresh>>>;

pub(super) fn archive_group(
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    status: &StatusHandle,
) -> (adw::PreferencesGroup, SearchablePreferencesGroup) {
    let title = "Configuration Backup";
    let description =
        "Back up, restore latest, or choose saved backups. New backups keep the latest five.";
    let group = adw::PreferencesGroup::builder()
        .title(tr(title))
        .description(tr(description))
        .build();
    let mut search_group = SearchablePreferencesGroup::new(&group, title, description);

    let archive_title = "Back Up Current Configuration";
    let archive_subtitle = "Create a new backup in the config folder.";
    let archive_row = action_row("document-save-symbolic", archive_title, archive_subtitle);
    search_group.add_row(&archive_row, archive_title, archive_subtitle);
    group.add(&archive_row);

    let restore_title = "Restore Latest Configuration Backup";
    let restore_subtitle = "Restore rules, budgets, and field names from the latest backup.";
    let restore_row = action_row("document-revert-symbolic", restore_title, restore_subtitle);
    search_group.add_row(&restore_row, restore_title, restore_subtitle);
    group.add(&restore_row);
    register_config_widget(ui_handles, &archive_row);
    register_config_widget(ui_handles, &restore_row);

    let backup_rows = Rc::new(RefCell::new(Vec::new()));
    let refresh_holder: BackupRefreshSlot = Rc::new(RefCell::new(None));
    let refresh_for_closure = Rc::clone(&refresh_holder);
    let backups_group_for_refresh = group.clone();
    let backup_rows_for_refresh = Rc::clone(&backup_rows);
    let state_for_refresh = Rc::clone(state);
    let ui_for_refresh = Rc::clone(ui_handles);
    let status_for_refresh = status.clone();
    let restore_row_for_refresh = restore_row.clone();
    let refresh_backups: BackupRefresh = Rc::new(move || {
        let Some(refresh_backups) = refresh_for_closure.borrow().as_ref().cloned() else {
            return;
        };
        refresh_backup_rows(
            &backups_group_for_refresh,
            &backup_rows_for_refresh,
            &state_for_refresh,
            &ui_for_refresh,
            &status_for_refresh,
            &restore_row_for_refresh,
            refresh_backups,
        );
    });
    *refresh_holder.borrow_mut() = Some(Rc::clone(&refresh_backups));
    refresh_backups();

    let ui_for_archive = Rc::clone(ui_handles);
    let status_for_archive = status.clone();
    let restore_row_for_archive = restore_row.clone();
    let refresh_for_archive = Rc::clone(&refresh_backups);
    archive_row.connect_activated(move |row| {
        if !row.is_sensitive() {
            return;
        }
        archive_configuration(
            Rc::clone(&ui_for_archive),
            status_for_archive.clone(),
            restore_row_for_archive.clone(),
            Rc::clone(&refresh_for_archive),
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

pub(super) fn automatic_configuration_group(
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    status: StatusHandle,
) -> (adw::PreferencesGroup, SearchablePreferencesGroup) {
    let title = "Configuration Templates";
    let description = "Use the built-in defaults or start with an empty configuration.";
    let group = adw::PreferencesGroup::builder()
        .title(tr(title))
        .description(tr(description))
        .build();
    let mut search_group = SearchablePreferencesGroup::new(&group, title, description);

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

fn refresh_backup_rows(
    group: &adw::PreferencesGroup,
    rows: &Rc<RefCell<Vec<gtk::Widget>>>,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    status: &StatusHandle,
    restore_row: &adw::ActionRow,
    refresh_backups: BackupRefresh,
) {
    for row in rows.borrow_mut().drain(..) {
        group.remove(&row);
    }

    let archives = match data::configuration_archives() {
        Ok(archives) => archives,
        Err(error) => {
            set_config_widget_base_sensitive(ui_handles, restore_row, false);
            let row = adw::ActionRow::builder()
                .title(tr("Could not read configuration backups"))
                .subtitle(format!("{error:#}"))
                .build();
            group.add(&row);
            rows.borrow_mut().push(row.upcast::<gtk::Widget>());
            return;
        }
    };

    set_config_widget_base_sensitive(ui_handles, restore_row, !archives.is_empty());
    if archives.is_empty() {
        let row = adw::ActionRow::builder()
            .title(tr("No configuration backups yet"))
            .subtitle(tr("Create a backup to make it available here."))
            .build();
        group.add(&row);
        rows.borrow_mut().push(row.upcast::<gtk::Widget>());
        return;
    }

    for archive in archives {
        let row = backup_row(
            &archive,
            state,
            ui_handles,
            status,
            Rc::clone(&refresh_backups),
        );
        group.add(&row);
        rows.borrow_mut().push(row.upcast::<gtk::Widget>());
    }
}

fn backup_row(
    archive: &data::ConfigurationArchive,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    status: &StatusHandle,
    refresh_backups: BackupRefresh,
) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(&archive.label)
        .subtitle(archive.path.display().to_string())
        .build();
    let restore_button = backup_icon_button("document-revert-symbolic", "Restore this backup");
    let remove_button = backup_icon_button("user-trash-symbolic", "Remove this backup");
    remove_button.add_css_class("destructive-action");
    row.add_suffix(&restore_button);
    row.add_suffix(&remove_button);
    register_config_widget(ui_handles, &restore_button);
    register_config_widget(ui_handles, &remove_button);

    let state_for_restore = Rc::clone(state);
    let ui_for_restore = Rc::clone(ui_handles);
    let status_for_restore = status.clone();
    let archive_id_for_restore = archive.id.clone();
    restore_button.connect_clicked(move |_| {
        restore_configuration_archive_by_id(
            Rc::clone(&state_for_restore),
            Rc::clone(&ui_for_restore),
            status_for_restore.clone(),
            archive_id_for_restore.clone(),
        );
    });

    let ui_for_remove = Rc::clone(ui_handles);
    let status_for_remove = status.clone();
    let archive_id_for_remove = archive.id.clone();
    remove_button.connect_clicked(move |_| {
        remove_configuration_archive(
            Rc::clone(&ui_for_remove),
            status_for_remove.clone(),
            archive_id_for_remove.clone(),
            Rc::clone(&refresh_backups),
        );
    });

    row
}

fn backup_icon_button(icon_name: &str, tooltip: &str) -> gtk::Button {
    let button = gtk::Button::builder()
        .icon_name(icon_name)
        .tooltip_text(tr(tooltip))
        .build();
    button.add_css_class("flat");
    button.add_css_class("image-button");
    button
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
