use super::*;

fn assert_object<T: IsA<gtk::glib::Object>>(builder: &gtk::Builder, id: &str, resource: &str) {
    let _: T = builder_object(builder, id, resource);
}

#[test]
fn ui_resources_build_expected_objects() {
    gtk::init().expect("initialize GTK");
    crate::resources::register().expect("register embedded resources");
    for resource in [
        "action-dialog.ui",
        "fake-transactions-dialog.ui",
        "loading-placeholder.ui",
        "management-dialog.ui",
        "main-window.ui",
        "operation-queue-dialog.ui",
        "partial-load-notice.ui",
        "rule-search-chips.ui",
        "settings-dialog.ui",
        "shortcuts-dialog.ui",
        "status-bar.ui",
        "status-history-dialog.ui",
    ] {
        let _ = builder_from_resource(resource);
    }

    let action = builder_from_resource("action-dialog.ui");
    assert_object::<adw::WindowTitle>(&action, "action_title", "action-dialog.ui");
    assert_object::<gtk::Button>(&action, "action_search_button", "action-dialog.ui");

    let loading = builder_from_resource("loading-placeholder.ui");
    assert_object::<gtk::Box>(&loading, "loading_placeholder", "loading-placeholder.ui");
    assert_object::<adw::Spinner>(
        &loading,
        "loading_placeholder_spinner",
        "loading-placeholder.ui",
    );
    let loading_title = builder_object::<gtk::Label>(
        &loading,
        "loading_placeholder_title",
        "loading-placeholder.ui",
    );
    assert_eq!(loading_title.label(), gettext("Loading"));
    let loading_description = builder_object::<gtk::Label>(
        &loading,
        "loading_placeholder_description",
        "loading-placeholder.ui",
    );
    assert_eq!(
        loading_description.label(),
        gettext("Preparing this page. Large CSV files may take a moment.")
    );

    let settings = builder_from_resource("settings-dialog.ui");
    assert_object::<adw::WindowTitle>(&settings, "settings_title", "settings-dialog.ui");

    let operation_queue = builder_from_resource("operation-queue-dialog.ui");
    assert_object::<gtk::ListBox>(
        &operation_queue,
        "operation_queue_list",
        "operation-queue-dialog.ui",
    );

    let status_history = builder_from_resource("status-history-dialog.ui");
    assert_object::<gtk::ListBox>(
        &status_history,
        "status_history_list",
        "status-history-dialog.ui",
    );

    let partial_load = builder_from_resource("partial-load-notice.ui");
    assert_object::<gtk::Button>(
        &partial_load,
        "partial_load_notice_reload_button",
        "partial-load-notice.ui",
    );

    let rule_search_chips = builder_from_resource("rule-search-chips.ui");
    assert_object::<adw::WrapBox>(
        &rule_search_chips,
        "rule_search_chips_wrap",
        "rule-search-chips.ui",
    );
    let chips_entry = builder_object::<adw::EntryRow>(
        &rule_search_chips,
        "rule_search_chips_entry",
        "rule-search-chips.ui",
    );
    assert_eq!(chips_entry.title(), gettext("Add search text"));

    let shortcuts = builder_from_resource("shortcuts-dialog.ui");
    let shortcuts_dialog = builder_object::<adw::ShortcutsDialog>(
        &shortcuts,
        "shortcuts_dialog",
        "shortcuts-dialog.ui",
    );
    assert_eq!(shortcuts_dialog.title(), gettext("Keyboard Shortcuts"));
    assert_object::<adw::ShortcutsSection>(
        &shortcuts,
        "shortcuts_settings_section",
        "shortcuts-dialog.ui",
    );
}

#[test]
fn translated_menu_keeps_actions_and_translates_labels() {
    let menu = gtk::gio::Menu::new();
    menu.append(Some("Save"), Some("app.save"));

    translate_menu(&menu);

    let label = menu
        .item_attribute_value(0, "label", None)
        .and_then(|value| value.get::<String>());
    let action = menu
        .item_attribute_value(0, "action", None)
        .and_then(|value| value.get::<String>());

    let expected_label = gettext("Save");
    assert_eq!(label.as_deref(), Some(expected_label.as_str()));
    assert_eq!(action.as_deref(), Some("app.save"));
}

#[test]
fn strings_without_translation_are_left_as_is() {
    assert_eq!(translate_text("not in the catalog"), "not in the catalog");
}
