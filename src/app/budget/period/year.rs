use super::controls::{period_controls_are_loading, period_controls_box};
use super::*;

pub(in crate::app) fn year_selector_row(
    years: &[i32],
    selected_year: i32,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) -> gtk::ListBox {
    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    list.set_selection_mode(gtk::SelectionMode::None);

    let subtitle = if ui_handles.compare_categories_previous_period.get() {
        trf(
            "Gray graph bars show {year}",
            &[("year", (selected_year - 1).to_string())],
        )
    } else {
        tr("Only the selected year is loaded")
    };
    let row = adw::ActionRow::builder()
        .title(tr("Year"))
        .subtitle(subtitle)
        .use_markup(false)
        .activatable(false)
        .selectable(false)
        .build();
    row.add_prefix(&gtk::Image::from_icon_name("view-grid-symbolic"));

    let previous_year = years
        .iter()
        .rev()
        .copied()
        .find(|year| *year < selected_year);
    let next_year = years.iter().copied().find(|year| *year > selected_year);

    let labels = years.iter().map(i32::to_string).collect::<Vec<_>>();
    let label_refs = labels.iter().map(String::as_str).collect::<Vec<_>>();
    let dropdown = gtk::DropDown::from_strings(&label_refs);
    if let Some(index) = years.iter().position(|year| *year == selected_year) {
        dropdown.set_selected(index as u32);
    }
    dropdown.set_valign(gtk::Align::Center);

    let years_for_dropdown = years.to_vec();
    let ui_for_dropdown = Rc::clone(ui_handles);
    let state_for_dropdown = Rc::clone(state);
    dropdown.connect_selected_notify(move |dropdown| {
        let index = dropdown.selected() as usize;
        let Some(year) = years_for_dropdown.get(index).copied() else {
            return;
        };
        set_shared_year(year, &ui_for_dropdown, &state_for_dropdown);
    });

    let controls = period_controls_box(ui_handles.as_ref());
    if let Some(year) = previous_year {
        let button = ui::icon_button("go-previous-symbolic", "Previous year");
        button.add_css_class("flat");
        button.set_valign(gtk::Align::Center);
        let ui_for_previous = Rc::clone(ui_handles);
        let state_for_previous = Rc::clone(state);
        button.connect_clicked(move |_| {
            set_shared_year(year, &ui_for_previous, &state_for_previous);
        });
        controls.append(&button);
    }
    controls.append(&dropdown);
    if let Some(year) = next_year {
        let button = ui::icon_button("go-next-symbolic", "Next year");
        button.add_css_class("flat");
        button.set_valign(gtk::Align::Center);
        let ui_for_next = Rc::clone(ui_handles);
        let state_for_next = Rc::clone(state);
        button.connect_clicked(move |_| {
            set_shared_year(year, &ui_for_next, &state_for_next);
        });
        controls.append(&button);
    }

    row.add_suffix(&controls);
    list.append(&row);
    list
}

pub(in crate::app) fn selected_year(data: &AppData, ui_handles: &UiHandles) -> Option<i32> {
    let years = available_years(data);
    let default_year = data
        .default_month
        .map(|month| month.year)
        .or_else(|| years.last().copied())?;
    let selected = ui_handles
        .selected_year
        .get()
        .filter(|year| ui_handles.period_user_selected.get() && years.contains(year))
        .unwrap_or(default_year);
    ui_handles.selected_year.set(Some(selected));
    ui_handles.preferences.set_selected_year(selected);
    sync_budget_period_year(ui_handles, selected);
    Some(selected)
}

fn available_years(data: &AppData) -> Vec<i32> {
    if data.available_years.is_empty() {
        available_budget_months(data)
            .into_iter()
            .map(|month| month.year)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    } else {
        data.available_years.clone()
    }
}

fn set_shared_year(year: i32, ui_handles: &Rc<UiHandles>, state: &Rc<RefCell<AppData>>) {
    if period_controls_are_loading(ui_handles.as_ref()) {
        return;
    }
    if ui_handles.selected_year.get() == Some(year)
        && ui_handles
            .selected_budget_month
            .get()
            .map(|month| month.year == year)
            .unwrap_or(true)
    {
        return;
    }
    ui_handles.period_user_selected.set(true);
    ui_handles.selected_year.set(Some(year));
    ui_handles.preferences.set_selected_year(year);
    sync_budget_period_year(ui_handles.as_ref(), year);
    sync_transaction_filter_year(ui_handles.as_ref(), year);
    render_views(&state.borrow(), ui_handles, state);
    show_status(
        ui_handles,
        &trf(
            "Year: {year}. The budget period uses the same year.",
            &[("year", year.to_string())],
        ),
    );
}

fn sync_transaction_filter_year(ui_handles: &UiHandles, year: i32) {
    if ui_handles.stack.visible_child_name().as_deref() != Some("transactions") {
        return;
    }

    let Some(filter) = ui_handles
        .active_transaction_filter
        .borrow()
        .as_ref()
        .and_then(|filter| filter.with_year(year))
    else {
        return;
    };

    let query = filter.query();
    *ui_handles.active_transaction_filter.borrow_mut() = Some(filter);
    *ui_handles.search_query.borrow_mut() = query.clone();
    ui_handles.search_bar.set_search_mode(!query.is_empty());
    if ui_handles.search_entry.text().as_str() != query {
        ui_handles.search_entry.set_text(&query);
    }
}

fn sync_budget_period_year(ui_handles: &UiHandles, year: i32) {
    if let Some(month) = ui_handles.selected_budget_month.get() {
        if month.year != year {
            let month = MonthKey::new(year, month.month);
            ui_handles.selected_budget_month.set(Some(month));
            ui_handles.preferences.set_selected_budget_month(month);
        }
    }
}
