use super::group::preference_row_visible;
use super::*;

pub(super) fn remember_preference_group(
    advanced_features: bool,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) -> Option<(adw::PreferencesGroup, SearchablePreferencesGroup)> {
    let title = "Remember";
    let description = "Choose whether opened bank files are forgotten after this session, remembered as data, or remembered with reusable analysis cache.";
    let writable = Preferences::key_for_action("app.remember-mode")
        .map(|key| ui.preferences.is_writable(key))
        .unwrap_or(true);
    if !preference_row_visible(writable, advanced_features) {
        return None;
    }

    let group = adw::PreferencesGroup::builder()
        .title(tr(title))
        .description(tr(description))
        .build();
    let mut search_group = SearchablePreferencesGroup::new(&group, title, description);
    let row = remember_preference_row(state, ui, writable);
    search_group.add_row(&row, title, description);
    group.add(&row);
    Some((group, search_group))
}

fn remember_preference_row(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    writable: bool,
) -> adw::ComboRow {
    let labels = RememberMode::SETTINGS_VALUES
        .iter()
        .map(|mode| tr(mode.label()))
        .collect::<Vec<_>>();
    let label_refs = labels.iter().map(String::as_str).collect::<Vec<_>>();
    let model = gtk::StringList::new(&label_refs);
    let selected = remember_mode_index(ui.remember_mode.get());
    let row = adw::ComboRow::builder()
        .title(tr("Remember"))
        .subtitle(tr("Forget opens bank files live for this session. Data only remembers copied bank files. Data and analytics also keeps a reusable processed cache."))
        .model(&model)
        .selected(selected)
        .build();

    if writable {
        let state_for_row = Rc::clone(state);
        let ui_for_row = Rc::clone(ui);
        row.connect_selected_notify(move |row| {
            let Some(mode) = RememberMode::SETTINGS_VALUES
                .get(row.selected() as usize)
                .copied()
            else {
                return;
            };
            set_remember_mode(mode, &state_for_row, &ui_for_row);
        });
    } else {
        row.set_sensitive(false);
        row.set_tooltip_text(Some(&tr("This preference is managed by the system.")));
    }

    row
}

fn remember_mode_index(mode: RememberMode) -> u32 {
    RememberMode::SETTINGS_VALUES
        .iter()
        .position(|candidate| *candidate == mode)
        .unwrap_or(2) as u32
}
