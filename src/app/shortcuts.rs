use super::*;

type ActionAccelerators = (&'static str, &'static [&'static str]);

const COMMON_ACTION_ACCELERATORS: &[ActionAccelerators] = &[
    ("app.go-back", &["<alt>Left"]),
    ("app.import-csv", &["<primary>O"]),
    ("app.find", &["<primary>F"]),
    ("app.reload", &["<primary>R"]),
    ("app.reload-all", &["<primary><shift>R"]),
    ("app.clear-cache-and-reload", &["<primary><shift>Delete"]),
    ("app.copy-page", &["<primary><shift>C"]),
    ("app.print-page", &["<primary>P"]),
    ("app.export-csv", &["<primary>E"]),
    ("app.view-overview", &["<primary>1"]),
    ("app.view-budget", &["<primary>2"]),
    ("app.view-transactions", &["<primary>3"]),
    ("app.view-diagnostics", &["<primary>4"]),
    ("app.manage-rules", &["<primary><alt>R"]),
    ("app.manage-budgets", &["<primary><alt>B"]),
    ("app.manage-aliases", &["<primary><alt>N"]),
    ("app.configuration", &["<primary><shift>comma"]),
    ("app.preferences", &["<primary>comma"]),
    ("app.dedupe-enabled", &["<primary>D"]),
    ("app.advanced-features", &["<primary><alt>A"]),
    ("app.show-all", &["<primary><alt>L"]),
    (
        "app.compare-categories-previous-period",
        &["<primary><alt>P"],
    ),
    ("app.advanced-autofill", &["<primary><alt>F"]),
    ("app.autohide-status", &["<primary><alt>M"]),
    ("app.about", &["<primary>I"]),
    ("app.shortcuts", &["F1", "<primary>question"]),
    ("app.quit", &["<primary>Q"]),
];

const CHECK_FOR_UPDATES_ACCELERATORS: ActionAccelerators =
    ("app.check-for-updates", &["<primary>U"]);
const SHORTCUTS_DIALOG_RESOURCE: &str = "shortcuts-dialog.ui";

#[cfg(all(target_os = "linux", feature = "setup", not(feature = "flatpak")))]
const INSTALL_LOCALLY_ACCELERATORS: ActionAccelerators =
    ("app.install-locally", &["<primary><shift>I"]);

pub(in crate::app) fn install_action_accelerators(app: &adw::Application) {
    for (action_name, accelerators) in COMMON_ACTION_ACCELERATORS {
        app.set_accels_for_action(action_name, accelerators);
    }
    if updater::supports_update_checks() {
        app.set_accels_for_action(
            CHECK_FOR_UPDATES_ACCELERATORS.0,
            CHECK_FOR_UPDATES_ACCELERATORS.1,
        );
    }
    #[cfg(all(target_os = "linux", feature = "setup", not(feature = "flatpak")))]
    app.set_accels_for_action(
        INSTALL_LOCALLY_ACCELERATORS.0,
        INSTALL_LOCALLY_ACCELERATORS.1,
    );
}

pub(in crate::app) fn build_shortcuts_dialog() -> adw::ShortcutsDialog {
    let builder = ui::builder_from_resource(SHORTCUTS_DIALOG_RESOURCE);
    let dialog = ui::builder_object::<adw::ShortcutsDialog>(
        &builder,
        "shortcuts_dialog",
        SHORTCUTS_DIALOG_RESOURCE,
    );
    let app = ui::builder_object::<adw::ShortcutsSection>(
        &builder,
        "shortcuts_app_section",
        SHORTCUTS_DIALOG_RESOURCE,
    );

    add_optional_app_shortcuts(&app);
    dialog
}

fn add_optional_app_shortcuts(section: &adw::ShortcutsSection) {
    if updater::supports_update_checks() {
        add_action_shortcut(section, "Check for Updates", "app.check-for-updates");
    }
    #[cfg(all(target_os = "linux", feature = "setup", not(feature = "flatpak")))]
    if setup::can_install_locally() {
        add_action_shortcut(section, "Install Locally", "app.install-locally");
    }
}

fn add_action_shortcut(section: &adw::ShortcutsSection, title: &str, action_name: &str) {
    section.add(adw::ShortcutsItem::from_action(&tr(title), action_name));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn common_action_accelerators_are_unique_and_populated() {
        let mut action_names = HashSet::new();
        let mut accelerators = HashSet::new();

        for (action_name, action_accelerators) in COMMON_ACTION_ACCELERATORS {
            assert!(!action_name.is_empty());
            assert!(!action_accelerators.is_empty());
            assert!(action_names.insert(*action_name));
            for accelerator in *action_accelerators {
                assert!(!accelerator.is_empty());
                assert!(accelerators.insert(*accelerator));
            }
        }
    }
}
