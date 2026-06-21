use super::*;

pub(super) fn preference_group(
    title: &str,
    description: &str,
    rows: &[PreferenceSpec<'_>],
    advanced_features: bool,
    preferences: &Preferences,
) -> Option<(adw::PreferencesGroup, SearchablePreferencesGroup)> {
    let group = adw::PreferencesGroup::builder()
        .title(tr(title))
        .description(tr(description))
        .build();
    let mut search_group = SearchablePreferencesGroup::new(&group, title, description);
    let mut added = false;

    for spec in rows {
        let writable = Preferences::key_for_action(spec.action_name)
            .map(|key| preferences.is_writable(key))
            .unwrap_or(true);
        if !preference_row_visible(writable, advanced_features) {
            continue;
        }
        let row = preference_row(spec, writable);
        search_group.add_row(&row, spec.title, spec.subtitle);
        group.add(&row);
        added = true;
    }

    added.then_some((group, search_group))
}

pub(super) fn preference_row_visible(writable: bool, advanced_features: bool) -> bool {
    writable || advanced_features
}

fn preference_row(spec: &PreferenceSpec<'_>, writable: bool) -> adw::SwitchRow {
    let row = adw::SwitchRow::builder()
        .title(tr(spec.title))
        .subtitle(tr(spec.subtitle))
        .build();
    row.set_active(spec.active);
    if writable {
        row.set_action_name(Some(spec.action_name));
        if let Some(target) = spec.visibility_target.clone() {
            let visibility_gate = spec.visibility_gate.clone();
            row.connect_active_notify(move |row| {
                let visible = row.is_active();
                if let Some(gate) = &visibility_gate {
                    gate.set(visible);
                }
                target.set_visible(visible);
            });
        }
    } else {
        row.set_sensitive(false);
        row.set_tooltip_text(Some(&tr("This preference is managed by the system.")));
    }
    row
}
