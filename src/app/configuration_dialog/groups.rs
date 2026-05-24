use super::tasks::{
    archive_configuration, restore_configuration_archive, restore_default_configuration,
    restore_empty_configuration,
};
use super::*;

pub(super) fn archive_group(
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

pub(super) fn experimental_configuration_group(
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    status: StatusHandle,
) -> Option<(adw::PreferencesGroup, SearchablePreferencesGroup)> {
    let advanced_features = ui_handles.advanced_features.get();
    let smart_insights_enabled = ui_handles.show_predictions.get();
    if !automatic_configuration_generation_visible(advanced_features) {
        return None;
    }

    let title = "Experimental";
    let description = "Try Smart Insights and local AI assisted configuration actions.";
    let group = adw::PreferencesGroup::builder()
        .title(tr(title))
        .description(tr(description))
        .build();
    let mut search_group = SearchablePreferencesGroup::new(&group, title, description);

    let generate_title = "Generate Configuration from Transactions";
    let generate_subtitle = automatic_configuration_generation_subtitle(smart_insights_enabled);
    let generate_row = action_row("view-refresh-symbolic", generate_title, generate_subtitle);
    if !automatic_configuration_generation_enabled(advanced_features, smart_insights_enabled) {
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

    Some((group, search_group))
}

pub(super) fn automatic_configuration_generation_visible(advanced_features: bool) -> bool {
    cfg!(feature = "smart-insights") && advanced_features
}

pub(super) fn automatic_configuration_generation_enabled(
    advanced_features: bool,
    smart_insights_enabled: bool,
) -> bool {
    smart_pattern_detection_enabled(advanced_features, smart_insights_enabled)
}

pub(super) fn automatic_configuration_generation_subtitle(
    smart_insights_enabled: bool,
) -> &'static str {
    if !smart_insights_enabled {
        "Requires Smart Insights. Enable Smart Insights to generate configuration from transactions."
    } else {
        "Create rules, budget codes, field mappings, and hidden refund/split patterns from imported transactions."
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
