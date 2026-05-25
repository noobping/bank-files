use super::*;

#[derive(Clone)]
pub(in crate::app) struct SearchableActionRow {
    pub(in crate::app) widget: gtk::Widget,
    keywords: String,
}

pub(in crate::app) fn searchable_action_row(
    row: &impl IsA<gtk::Widget>,
    title: &str,
    subtitle: &str,
    extra_keywords: &[String],
) -> SearchableActionRow {
    SearchableActionRow {
        widget: row.clone().upcast::<gtk::Widget>(),
        keywords: action_search_keywords(title, subtitle, extra_keywords),
    }
}

pub(in crate::app) fn connect_action_search(
    search_entry: &gtk::SearchEntry,
    rows: Vec<SearchableActionRow>,
    empty_widget: Option<gtk::Widget>,
) {
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().trim().to_lowercase();
        let show_all = query.is_empty();
        let mut visible_count = 0usize;

        for row in &rows {
            let visible = show_all || row.keywords.contains(&query);
            row.widget.set_visible(visible);
            if visible {
                visible_count += 1;
            }
        }

        if let Some(empty_widget) = &empty_widget {
            empty_widget.set_visible(visible_count == 0);
        }
    });
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

fn action_search_keywords(title: &str, subtitle: &str, extra_keywords: &[String]) -> String {
    let mut values = vec![
        title.to_string(),
        subtitle.to_string(),
        tr(title),
        tr(subtitle),
    ];
    values.extend(extra_keywords.iter().cloned());
    values.join(" ").to_lowercase()
}

fn search_keywords(title: &str, subtitle: &str) -> String {
    format!("{title} {subtitle} {} {}", tr(title), tr(subtitle)).to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_search_keywords_include_extra_values() {
        let keywords = action_search_keywords(
            "Groceries",
            "Expenses",
            &["FOOD".to_string(), "Rule match".to_string()],
        );
        assert!(keywords.contains("groceries"));
        assert!(keywords.contains("expenses"));
        assert!(keywords.contains("food"));
        assert!(keywords.contains("rule match"));
    }
}
