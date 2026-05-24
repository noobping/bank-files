use super::*;

pub(super) fn prediction_scope_allows_forecast(
    data: &AppData,
    selected_year: Option<i32>,
    current_year: i32,
) -> bool {
    let Some(selected_year) = selected_year else {
        return false;
    };
    let years = forecast_years(data);
    let Some(anchor_year) = forecast_anchor_year(&years, current_year) else {
        return false;
    };

    !has_year_after(&years, selected_year) && !has_year_after(&years, anchor_year)
}

fn forecast_years(data: &AppData) -> Vec<i32> {
    if !data.available_years.is_empty() {
        return data.available_years.clone();
    }

    data.transactions
        .iter()
        .map(Transaction::year)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn forecast_anchor_year(years: &[i32], current_year: i32) -> Option<i32> {
    let latest = years.last().copied()?;
    if years.contains(&current_year) || latest > current_year {
        Some(current_year)
    } else {
        Some(latest)
    }
}

fn has_year_after(years: &[i32], year: i32) -> bool {
    years.iter().any(|candidate| *candidate > year)
}

pub(super) fn append_survival_forecast(
    data: &AppData,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    let Some(forecast) = analytics::survival_forecast(data) else {
        return;
    };

    ui_handles.overview.append(&ui::section_title(
        "Forecast",
        "Projected with imported transactions, budgeted income, and planned spending.",
    ));

    let state_for_current = Rc::clone(state);
    let ui_for_current = Rc::clone(ui_handles);
    let current_month = forecast.anchor_month;
    let current_period = forecast.current_month.clone();
    let state_for_next = Rc::clone(state);
    let ui_for_next = Rc::clone(ui_handles);
    let next_month = forecast.next_month;
    let next_period = forecast.next_month_period.clone();
    let state_for_year = Rc::clone(state);
    let ui_for_year = Rc::clone(ui_handles);
    let year = forecast.anchor_month.year;
    let year_period = forecast.rest_of_year.clone();

    ui_handles.overview.append(&ui::metric_grid(
        vec![
            ui::activatable_metric_card(
                "Current month",
                &signed_money(forecast.current_month.projected_balance),
                &forecast_subtitle(&forecast.current_month),
                move || {
                    show_forecast_details(
                        "Current month",
                        &ui::month_label(current_month),
                        current_period.clone(),
                        TransactionFilter::month(current_month),
                        &state_for_current,
                        &ui_for_current,
                    );
                },
            ),
            ui::activatable_metric_card(
                "Next month",
                &signed_money(forecast.next_month_period.projected_balance),
                &forecast_subtitle(&forecast.next_month_period),
                move || {
                    show_forecast_details(
                        "Next month",
                        &ui::month_label(next_month),
                        next_period.clone(),
                        TransactionFilter::month(next_month),
                        &state_for_next,
                        &ui_for_next,
                    );
                },
            ),
            ui::activatable_metric_card(
                "Rest of year",
                &signed_money(forecast.rest_of_year.projected_balance),
                &forecast_subtitle(&forecast.rest_of_year),
                move || {
                    show_forecast_details(
                        "Rest of year",
                        &year.to_string(),
                        year_period.clone(),
                        TransactionFilter::year(year),
                        &state_for_year,
                        &ui_for_year,
                    );
                },
            ),
        ],
        3,
    ));
}

fn forecast_subtitle(period: &analytics::ForecastPeriod) -> String {
    trf(
        "{status}: {income} in, {expenses} out",
        &[
            ("status", tr(forecast_status_label(period.status))),
            ("income", money(period.income)),
            ("expenses", money(period.expenses)),
        ],
    )
}

fn forecast_status_label(status: analytics::ForecastStatus) -> &'static str {
    match status {
        analytics::ForecastStatus::Safe => "Looks safe",
        analytics::ForecastStatus::Tight => "Tight",
        analytics::ForecastStatus::Short => "Short",
    }
}

fn show_forecast_details(
    title: &str,
    period_label: &str,
    period: analytics::ForecastPeriod,
    filter: TransactionFilter,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let header = ui::cancelable_dialog_header(title, period_label);

    let transactions_button = ui::icon_button(
        "view-list-symbolic",
        "Open the transactions used for this forecast period",
    );
    transactions_button.add_css_class("flat");
    header.pack_end(&transactions_button);

    let page = ui::page_box();
    page.append(&ui::section_title(
        "Prediction details",
        "This estimate uses imported transactions, budgeted income, planned spending, and recent history.",
    ));

    let grid = ui::form_grid();
    add_forecast_detail(&grid, 0, "Status", tr(forecast_status_label(period.status)));
    add_forecast_detail(&grid, 1, "Imported income", money(period.imported_income));
    add_forecast_detail(&grid, 2, "Expected income", money(period.expected_income));
    add_forecast_detail(&grid, 3, "Projected income", money(period.income));
    add_forecast_detail(
        &grid,
        4,
        "Imported expenses",
        money(period.imported_expenses),
    );
    add_forecast_detail(&grid, 5, "Planned expenses", money(period.planned_expenses));
    add_forecast_detail(&grid, 6, "Projected expenses", money(period.expenses));
    add_forecast_detail(
        &grid,
        7,
        "Projected balance",
        signed_money(period.projected_balance),
    );
    page.append(&grid);
    let content = ui::action_dialog_scroll(&page);
    let view = ui::dialog_toolbar_view(&header, &content);

    let dialog = ui::content_dialog(tr(title), &view)
        .content_width(620)
        .default_widget(&transactions_button)
        .build();

    let state_for_transactions = Rc::clone(state);
    let ui_for_transactions = Rc::clone(ui_handles);
    let dialog_for_transactions = dialog.clone();
    transactions_button.connect_clicked(move |_| {
        dialog_for_transactions.close();
        show_transactions_filter(
            &state_for_transactions,
            &ui_for_transactions,
            filter.clone(),
        );
    });

    dialog.present(Some(&ui_handles.window));
}

fn add_forecast_detail(grid: &gtk::Grid, row: i32, label: &str, value: String) {
    ui::add_labeled(grid, row, label, &ui::wrapped_label(&value));
}
