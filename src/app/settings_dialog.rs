use super::*;

pub(in crate::app) struct SettingsDialogShell {
    pub(in crate::app) root: gtk::Box,
    pub(in crate::app) search_button: gtk::Button,
    pub(in crate::app) search_bar: gtk::SearchBar,
    pub(in crate::app) search_entry: gtk::SearchEntry,
}

pub(in crate::app) fn build_settings_dialog_shell(
    title: &str,
    search_placeholder: &str,
) -> SettingsDialogShell {
    let builder = ui::builder_from_resource("settings-dialog.ui");
    let root = builder
        .object::<gtk::Box>("settings_root")
        .expect("settings-dialog.ui should define settings_root");
    let header = builder
        .object::<adw::HeaderBar>("settings_header")
        .expect("settings-dialog.ui should define settings_header");
    header.set_title_widget(Some(&adw::WindowTitle::new(&tr(title), "")));

    let search_button = builder
        .object::<gtk::Button>("settings_search_button")
        .expect("settings-dialog.ui should define settings_search_button");
    search_button.set_tooltip_text(Some(&tr("Search")));

    let search_bar = builder
        .object::<gtk::SearchBar>("settings_search_bar")
        .expect("settings-dialog.ui should define settings_search_bar");
    let search_entry = builder
        .object::<gtk::SearchEntry>("settings_search_entry")
        .expect("settings-dialog.ui should define settings_search_entry");
    search_entry.set_placeholder_text(Some(&tr(search_placeholder)));
    search_bar.connect_entry(&search_entry);

    SettingsDialogShell {
        root,
        search_button,
        search_bar,
        search_entry,
    }
}

pub(in crate::app) struct SearchablePreferencesGroup {
    pub(in crate::app) group: adw::PreferencesGroup,
    rows: Vec<SearchablePreferenceRow>,
    keywords: String,
    visibility_gate: Option<Rc<Cell<bool>>>,
}

impl SearchablePreferencesGroup {
    pub(in crate::app) fn new(
        group: &adw::PreferencesGroup,
        title: &str,
        description: &str,
    ) -> Self {
        Self {
            group: group.clone(),
            rows: Vec::new(),
            keywords: search_keywords(title, description),
            visibility_gate: None,
        }
    }

    pub(in crate::app) fn set_visibility_gate(&mut self, gate: Rc<Cell<bool>>) {
        self.visibility_gate = Some(gate);
    }

    pub(in crate::app) fn add_row(
        &mut self,
        row: &impl IsA<gtk::Widget>,
        title: &str,
        subtitle: &str,
    ) {
        self.rows.push(SearchablePreferenceRow {
            widget: row.clone().upcast::<gtk::Widget>(),
            keywords: search_keywords(title, subtitle),
        });
    }
}

struct SearchablePreferenceRow {
    widget: gtk::Widget,
    keywords: String,
}

pub(in crate::app) fn connect_preference_search(
    search_entry: &gtk::SearchEntry,
    groups: Vec<SearchablePreferencesGroup>,
) {
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().trim().to_lowercase();
        let show_all = query.is_empty();

        for group in &groups {
            let gate_visible = group
                .visibility_gate
                .as_ref()
                .map(|gate| gate.get())
                .unwrap_or(true);
            let group_matches = !show_all && group.keywords.contains(&query);
            let mut group_visible = gate_visible && (show_all || group_matches);

            for row in &group.rows {
                let row_visible =
                    gate_visible && (show_all || group_matches || row.keywords.contains(&query));
                row.widget.set_visible(row_visible);
                group_visible |= row_visible;
            }

            group.group.set_visible(group_visible);
        }
    });
}

fn search_keywords(title: &str, subtitle: &str) -> String {
    format!("{title} {subtitle} {} {}", tr(title), tr(subtitle)).to_lowercase()
}
