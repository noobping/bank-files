use super::*;

mod groups;
mod snapshot;
mod tasks;

use groups::{archive_group, automatic_configuration_group};
use snapshot::configuration_page_snapshot;

const CONFIGURATION_BUSY_MESSAGE: &str = "Another edit or save is already running.";

pub(in crate::app) fn show_configuration_dialog(
    parent: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let shell = build_settings_dialog_shell("Configuration", "Search configuration");
    let root = shell.root;
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
        configuration_page_snapshot(),
    );
    let status = StatusHandle::from_status_bar(&status_bar);
    status.set_text(&tr("Configuration actions report progress here."));

    let (group, search_group) = archive_group(state, ui_handles, &status);
    page.add(&group);
    search_groups.push(search_group);

    let (group, search_group) = automatic_configuration_group(state, ui_handles, status.clone());
    page.add(&group);
    search_groups.push(search_group);

    root.append(&settings_dialog_scroll(&page));
    root.append(&status_bar.container);

    let dialog = settings_content_dialog("Configuration", &root, 720);

    ui::bind_search_bar(&dialog, &dialog, &search_bar, &search_entry);
    connect_preference_search(&search_entry, search_groups);

    dialog.present(Some(parent));
}

#[cfg(test)]
mod tests;
