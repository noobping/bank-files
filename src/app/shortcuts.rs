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
    ("app.auto-clean-config", &["<primary><alt>C"]),
    ("app.autohide-status", &["<primary><alt>M"]),
    ("app.about", &["<primary>I"]),
    ("app.shortcuts", &["F1", "<primary>question"]),
    ("app.quit", &["<primary>Q"]),
];

const CHECK_FOR_UPDATES_ACCELERATORS: ActionAccelerators =
    ("app.check-for-updates", &["<primary>U"]);
#[cfg(feature = "smart-insights")]
const SMART_INSIGHTS_ACCELERATORS: &[ActionAccelerators] = &[
    ("app.show-predictions", &["<primary><alt>S"]),
    ("app.hide-canceled-transactions", &["<primary><alt>H"]),
];
#[cfg(all(feature = "smart-insights", not(feature = "flatpak")))]
const ONLINE_SMART_INSIGHTS_ACCELERATORS: ActionAccelerators =
    ("app.online-smart-insights", &["<primary><alt>O"]);

#[cfg(all(target_os = "linux", feature = "setup", not(feature = "flatpak")))]
const INSTALL_LOCALLY_ACCELERATORS: ActionAccelerators =
    ("app.install-locally", &["<primary><shift>I"]);

pub(in crate::app) fn install_action_accelerators(app: &adw::Application) {
    for (action_name, accelerators) in COMMON_ACTION_ACCELERATORS {
        app.set_accels_for_action(action_name, accelerators);
    }
    #[cfg(feature = "smart-insights")]
    for (action_name, accelerators) in SMART_INSIGHTS_ACCELERATORS {
        app.set_accels_for_action(action_name, accelerators);
    }
    #[cfg(all(feature = "smart-insights", not(feature = "flatpak")))]
    app.set_accels_for_action(
        ONLINE_SMART_INSIGHTS_ACCELERATORS.0,
        ONLINE_SMART_INSIGHTS_ACCELERATORS.1,
    );
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

pub(in crate::app) fn build_shortcuts_dialog(
    advanced_features: bool,
    _smart_patterns_enabled: bool,
) -> adw::ShortcutsDialog {
    let dialog = ui::shortcuts_dialog(tr("Keyboard Shortcuts"), 640, -1);

    dialog.add(shortcuts_section(
        "Navigation",
        &[
            ShortcutSpec::action("Go Back", "app.go-back"),
            ShortcutSpec::action("Open Overview", "app.view-overview"),
            ShortcutSpec::action("Open Budget", "app.view-budget"),
            ShortcutSpec::action("Open Transactions", "app.view-transactions"),
            ShortcutSpec::action("Open Diagnostics", "app.view-diagnostics"),
        ],
    ));
    let file_shortcuts = [
        ShortcutSpec::action("Choose CSV Files", "app.import-csv"),
        ShortcutSpec::action("Export CSV", "app.export-csv"),
        ShortcutSpec::action("Quick Reload", "app.reload"),
        ShortcutSpec::action("Reload All", "app.reload-all"),
        ShortcutSpec::action("Clear Cache and Reload", "app.clear-cache-and-reload"),
    ];
    dialog.add(shortcuts_section("Files", &file_shortcuts));
    dialog.add(shortcuts_section(
        "Page",
        &[
            ShortcutSpec::action("Search or Filter", "app.find"),
            ShortcutSpec::action("Copy Page", "app.copy-page"),
            ShortcutSpec::action("Print Page", "app.print-page"),
        ],
    ));
    let mut manage_shortcuts = Vec::new();
    if advanced_features {
        manage_shortcuts.push(ShortcutSpec::action(
            "Manage Categorization Rules",
            "app.manage-rules",
        ));
    }
    manage_shortcuts.extend([
        ShortcutSpec::action("Manage Budgets", "app.manage-budgets"),
        ShortcutSpec::action("Normalize CSV Fields", "app.manage-aliases"),
        ShortcutSpec::accelerator("Filter the Manage Window", "<primary>F"),
    ]);
    dialog.add(shortcuts_section("Manage", &manage_shortcuts));

    let mut settings_shortcuts = vec![
        ShortcutSpec::action("Open Preferences", "app.preferences"),
        ShortcutSpec::action("Open Configuration", "app.configuration"),
        ShortcutSpec::action("Toggle Advanced Features", "app.advanced-features"),
    ];
    #[cfg(feature = "smart-insights")]
    if advanced_features {
        settings_shortcuts.push(ShortcutSpec::action(
            "Toggle Smart Insights",
            "app.show-predictions",
        ));
        #[cfg(not(feature = "flatpak"))]
        settings_shortcuts.push(ShortcutSpec::action(
            "Toggle Online Smart Insights",
            "app.online-smart-insights",
        ));
    }
    settings_shortcuts.push(ShortcutSpec::action(
        "Toggle Whole Form Autofill",
        "app.advanced-autofill",
    ));
    settings_shortcuts.extend([
        ShortcutSpec::action("Toggle Duplicate Filtering", "app.dedupe-enabled"),
        ShortcutSpec::action("Toggle Full Lists", "app.show-all"),
        ShortcutSpec::action(
            "Toggle Spending Comparison",
            "app.compare-categories-previous-period",
        ),
        ShortcutSpec::action("Toggle Auto Clean Config", "app.auto-clean-config"),
        ShortcutSpec::action("Toggle Status Autohide", "app.autohide-status"),
    ]);
    #[cfg(feature = "smart-insights")]
    if advanced_features {
        settings_shortcuts.push(ShortcutSpec::action(
            "Toggle Hide Refunded Transactions",
            "app.hide-canceled-transactions",
        ));
    }
    dialog.add(shortcuts_section("Settings", &settings_shortcuts));

    let mut app_shortcuts = Vec::new();
    if updater::supports_update_checks() {
        app_shortcuts.push(ShortcutSpec::action(
            "Check for Updates",
            "app.check-for-updates",
        ));
    }
    #[cfg(all(target_os = "linux", feature = "setup", not(feature = "flatpak")))]
    if setup::can_install_locally() {
        app_shortcuts.push(ShortcutSpec::action(
            "Install Locally",
            "app.install-locally",
        ));
    }
    app_shortcuts.extend([
        ShortcutSpec::action("About", "app.about"),
        ShortcutSpec::action("Show Keyboard Shortcuts", "app.shortcuts"),
        ShortcutSpec::action("Quit", "app.quit"),
    ]);
    dialog.add(shortcuts_section("App", &app_shortcuts));

    dialog
}

enum ShortcutSpec<'a> {
    Action {
        title: &'a str,
        action_name: &'a str,
    },
    Accelerator {
        title: &'a str,
        accelerator: &'a str,
    },
}

impl<'a> ShortcutSpec<'a> {
    fn action(title: &'a str, action_name: &'a str) -> Self {
        Self::Action { title, action_name }
    }

    fn accelerator(title: &'a str, accelerator: &'a str) -> Self {
        Self::Accelerator { title, accelerator }
    }
}

fn shortcuts_section(title: &str, shortcuts: &[ShortcutSpec<'_>]) -> adw::ShortcutsSection {
    let section = adw::ShortcutsSection::new(Some(&tr(title)));
    for shortcut in shortcuts {
        let item = match shortcut {
            ShortcutSpec::Action { title, action_name } => {
                adw::ShortcutsItem::from_action(&tr(title), action_name)
            }
            ShortcutSpec::Accelerator { title, accelerator } => {
                adw::ShortcutsItem::new(&tr(title), accelerator)
            }
        };
        section.add(item);
    }
    section
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
