use super::*;

pub(super) fn preference_group(
    title: &str,
    description: &str,
    rows: &[PreferenceSpec<'_>],
    advanced_features: bool,
    smart_insights_enabled: bool,
    preferences: &Preferences,
) -> Option<(adw::PreferencesGroup, SearchablePreferencesGroup)> {
    let group = adw::PreferencesGroup::builder()
        .title(tr(title))
        .description(tr(description))
        .build();
    let mut search_group = SearchablePreferencesGroup::new(&group, title, description);
    let mut added = false;
    let mut enabled_controllers: Vec<(Rc<Cell<bool>>, adw::SwitchRow)> = Vec::new();
    let mut enabled_targets: Vec<(Rc<Cell<bool>>, gtk::Widget, bool)> = Vec::new();

    for spec in rows {
        let writable = Preferences::key_for_action(spec.action_name)
            .map(|key| preferences.is_writable(key))
            .unwrap_or(true);
        if !spec.visible(writable, advanced_features, smart_insights_enabled) {
            continue;
        }
        let row = preference_row(spec, writable, smart_insights_enabled);
        let row_widget = row.clone().upcast::<gtk::Widget>();
        if let Some(gate) = &spec.enabled_controller_gate {
            enabled_controllers.push((Rc::clone(gate), row.clone()));
        }
        if let Some(gate) = &spec.enabled_by_gate {
            row_widget.set_sensitive(writable && gate.get());
            enabled_targets.push((Rc::clone(gate), row_widget.clone(), writable));
        }
        search_group.add_row(&row, spec.title, spec.subtitle);
        group.add(&row);
        added = true;
    }

    for (controller_gate, controller) in enabled_controllers {
        let targets = enabled_targets
            .iter()
            .filter(|(target_gate, _, _)| Rc::ptr_eq(target_gate, &controller_gate))
            .map(|(_, target, target_writable)| (target.clone(), *target_writable))
            .collect::<Vec<_>>();
        controller.connect_active_notify(move |row| {
            let active = row.is_active();
            controller_gate.set(active);
            targets
                .iter()
                .for_each(|(target, writable)| target.set_sensitive(active && *writable));
        });
    }

    added.then_some((group, search_group))
}

pub(super) fn preference_row_visible(
    writable: bool,
    advanced_features: bool,
    smart_insights_enabled: bool,
    requires_smart_insights: bool,
) -> bool {
    (writable || advanced_features)
        && (!requires_smart_insights || smart_insights_enabled || advanced_features)
}

pub(super) fn preference_row_enabled(
    writable: bool,
    smart_insights_enabled: bool,
    requires_smart_insights: bool,
) -> bool {
    writable && (!requires_smart_insights || smart_insights_enabled)
}
fn preference_row(
    spec: &PreferenceSpec<'_>,
    writable: bool,
    smart_insights_enabled: bool,
) -> adw::SwitchRow {
    let row = adw::SwitchRow::builder()
        .title(tr(spec.title))
        .subtitle(tr(spec.subtitle))
        .build();
    row.set_active(spec.active);
    if writable {
        row.set_action_name(Some(spec.action_name));
        row.set_sensitive(spec.sensitive(writable, smart_insights_enabled));
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
    if writable && spec.requires_smart_insights && !smart_insights_enabled {
        row.set_tooltip_text(Some(&tr("Enable Smart Insights to use this preference.")));
    }
    row
}
