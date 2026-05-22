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

pub(in crate::app) fn selected_budget_month(
    data: &AppData,
    ui_handles: &UiHandles,
) -> Option<MonthKey> {
    let fallback = default_budget_month(data)?;
    let years = data.available_years.clone();
    let shared_year = ui_handles
        .selected_year
        .get()
        .filter(|year| ui_handles.period_user_selected.get() && years.contains(year));
    let current = ui_handles
        .selected_budget_month
        .get()
        .filter(|month| ui_handles.period_user_selected.get() && years.contains(&month.year));
    let selected = match (current, shared_year) {
        (Some(month), Some(year)) if month.year != year => MonthKey::new(year, month.month),
        (Some(month), _) => month,
        (None, Some(year)) => MonthKey::new(year, fallback.month),
        (None, None) => fallback,
    };
    ui_handles.selected_budget_month.set(Some(selected));
    ui_handles.preferences.set_selected_budget_month(selected);
    ui_handles.preferences.set_selected_year(selected.year);
    Some(selected)
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

pub(in crate::app) fn set_shared_year(
    year: i32,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
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

pub(in crate::app) fn sync_budget_period_year(ui_handles: &UiHandles, year: i32) {
    if let Some(month) = ui_handles.selected_budget_month.get() {
        if month.year != year {
            let month = MonthKey::new(year, month.month);
            ui_handles.selected_budget_month.set(Some(month));
            ui_handles.preferences.set_selected_budget_month(month);
        }
    }
}

pub(in crate::app) fn default_budget_month(data: &AppData) -> Option<MonthKey> {
    data.default_month
        .or_else(|| analytics::default_reporting_month(&data.transactions, &data.budgets))
}

pub(in crate::app) fn totals_for_month(data: &AppData, month: MonthKey) -> analytics::Totals {
    analytics::totals_for_month(&data.transactions, &data.budgets, month)
}

pub(in crate::app) fn budget_period_row(
    data: &AppData,
    selected_month: MonthKey,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) -> gtk::ListBox {
    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    list.set_selection_mode(gtk::SelectionMode::None);

    let row = adw::ActionRow::builder()
        .title(tr("Period"))
        .subtitle(tr("Choose the month and year for budgets and spending."))
        .use_markup(false)
        .activatable(false)
        .selectable(false)
        .build();
    row.add_prefix(&gtk::Image::from_icon_name("view-grid-symbolic"));

    let months = available_budget_months(data);
    let previous_month = months
        .iter()
        .rev()
        .copied()
        .find(|month| *month < selected_month);
    let next_month = months.iter().copied().find(|month| *month > selected_month);

    let years = data.available_years.clone();
    let year_labels = years.iter().map(i32::to_string).collect::<Vec<_>>();
    let year_refs = year_labels.iter().map(String::as_str).collect::<Vec<_>>();
    let year_dropdown = gtk::DropDown::from_strings(&year_refs);
    if let Some(index) = years.iter().position(|year| *year == selected_month.year) {
        year_dropdown.set_selected(index as u32);
    }
    year_dropdown.set_tooltip_text(Some(&tr("Year")));
    year_dropdown.set_valign(gtk::Align::Center);

    let month_names = month_names();
    let month_refs = month_names.iter().map(String::as_str).collect::<Vec<_>>();
    let month_dropdown = gtk::DropDown::from_strings(&month_refs);
    month_dropdown.set_selected(selected_month.month.saturating_sub(1).min(11));
    month_dropdown.set_tooltip_text(Some(&tr("Month")));
    month_dropdown.set_valign(gtk::Align::Center);

    let controls = period_controls_box(ui_handles.as_ref());
    if let Some(month) = previous_month {
        let button = ui::icon_button("go-previous-symbolic", "Previous period");
        button.add_css_class("flat");
        button.set_valign(gtk::Align::Center);
        let ui_for_previous = Rc::clone(ui_handles);
        let state_for_previous = Rc::clone(state);
        button.connect_clicked(move |_| {
            set_budget_month(month, &ui_for_previous, &state_for_previous);
        });
        controls.append(&button);
    }
    controls.append(&month_dropdown);
    controls.append(&year_dropdown);
    if let Some(month) = next_month {
        let button = ui::icon_button("go-next-symbolic", "Next period");
        button.add_css_class("flat");
        button.set_valign(gtk::Align::Center);
        let ui_for_next = Rc::clone(ui_handles);
        let state_for_next = Rc::clone(state);
        button.connect_clicked(move |_| {
            set_budget_month(month, &ui_for_next, &state_for_next);
        });
        controls.append(&button);
    }

    let years_for_month = years.clone();
    let ui_for_month = Rc::clone(ui_handles);
    let state_for_month = Rc::clone(state);
    let year_dropdown_for_month = year_dropdown.clone();
    month_dropdown.connect_selected_notify(move |dropdown| {
        let year_index = year_dropdown_for_month.selected() as usize;
        let Some(year) = years_for_month.get(year_index).copied() else {
            return;
        };
        let month = dropdown.selected().saturating_add(1);
        set_budget_month(MonthKey::new(year, month), &ui_for_month, &state_for_month);
    });

    let ui_for_year = Rc::clone(ui_handles);
    let state_for_year = Rc::clone(state);
    let years_for_year = years.clone();
    let month_dropdown_for_year = month_dropdown.clone();
    year_dropdown.connect_selected_notify(move |dropdown| {
        let Some(year) = years_for_year.get(dropdown.selected() as usize).copied() else {
            return;
        };
        let month = month_dropdown_for_year.selected().saturating_add(1);
        set_budget_month(MonthKey::new(year, month), &ui_for_year, &state_for_year);
    });

    row.add_suffix(&controls);
    list.append(&row);
    list
}

fn period_controls_box(ui_handles: &UiHandles) -> gtk::Box {
    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    controls.add_css_class("period-controls");
    controls.set_sensitive(!period_controls_are_loading(ui_handles));
    controls
}

fn period_controls_are_loading(ui_handles: &UiHandles) -> bool {
    ui_handles.loading_count.get() > 0
}

fn available_budget_months(data: &AppData) -> Vec<MonthKey> {
    if data.available_months.is_empty() {
        analytics::monthly_totals_without_transfers(&data.transactions, &data.budgets, usize::MAX)
            .into_iter()
            .map(|summary| summary.month)
            .collect()
    } else {
        data.available_months.clone()
    }
}

pub(in crate::app) fn set_budget_month(
    month: MonthKey,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    if period_controls_are_loading(ui_handles.as_ref()) {
        return;
    }
    if ui_handles.selected_budget_month.get() == Some(month)
        && ui_handles.selected_year.get() == Some(month.year)
    {
        return;
    }
    ui_handles.period_user_selected.set(true);
    ui_handles.selected_budget_month.set(Some(month));
    ui_handles.selected_year.set(Some(month.year));
    ui_handles.preferences.set_selected_budget_month(month);
    ui_handles.preferences.set_selected_year(month.year);
    render_views(&state.borrow(), ui_handles, state);
    show_status(
        ui_handles,
        &trf(
            "Budget period: {month}. The overview also uses {year}.",
            &[
                ("month", ui::month_label(month)),
                ("year", month.year.to_string()),
            ],
        ),
    );
}

pub(in crate::app) fn month_names() -> Vec<String> {
    (1..=12).map(ui::month_name).collect()
}
