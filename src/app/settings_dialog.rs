use super::*;

pub(in crate::app) fn build_settings_header(title: &str) -> (adw::HeaderBar, gtk::Button) {
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&adw::WindowTitle::new(&tr(title), "")));

    let search_button = ui::icon_button("edit-find-symbolic", "Search");
    search_button.add_css_class("flat");
    header.pack_start(&search_button);

    (header, search_button)
}

pub(in crate::app) struct SearchablePreferencesGroup {
    pub(in crate::app) group: adw::PreferencesGroup,
    rows: Vec<SearchablePreferenceRow>,
    keywords: String,
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
        }
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
            let group_matches = !show_all && group.keywords.contains(&query);
            let mut group_visible = show_all || group_matches;

            for row in &group.rows {
                let row_visible = show_all || group_matches || row.keywords.contains(&query);
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
