use super::*;
use chrono::Datelike;

pub(in crate::app) fn render_overview(
    data: &AppData,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    ui::clear_box(&ui_handles.overview);
    let search = active_search(ui_handles.as_ref());
    let subtitle = search
        .as_ref()
        .map(|filter| {
            trf(
                "Filter “{query}” searches annual budgets and annual spending.",
                &[("query", filter.raw.clone())],
            )
        })
        .unwrap_or_else(|| {
            "Latest month, yearly comparison, trend, and budget room based on your imported CSV files."
                .to_string()
        });
    append_page_header(
        &ui_handles.overview,
        ui_handles.as_ref(),
        "Overview",
        &subtitle,
        summary::render_overview(data),
        &data.transactions,
    );

    if let Some(filter) = search {
        append_partial_load_notice(&ui_handles.overview, ui_handles, data);
        render_overview_search(data, ui_handles, state, &filter);
        return;
    }

    if data.transactions.is_empty() {
        append_partial_load_notice(&ui_handles.overview, ui_handles, data);
        let empty = adw::StatusPage::builder()
            .icon_name("document-open-symbolic")
            .title(tr("No transactions yet"))
            .description(tr(
                "Choose CSV files or drop bank files onto this window. Your data stays local.",
            ))
            .build();
        ui_handles.overview.append(&empty);
        return;
    }

    let selected_year = overview_selected_year(data, ui_handles.as_ref());
    if let Some(year) = selected_year {
        let warnings = annual_budget_attention_warnings(data, year);
        append_attention_warning_card(&ui_handles.overview, &warnings);
    }
    append_partial_load_notice(&ui_handles.overview, ui_handles, data);

    let dashboard = analytics::dashboard(data);
    let latest = dashboard
        .latest_month
        .map(ui::month_label)
        .unwrap_or_else(|| "-".to_string());

    let state_for_transactions = Rc::clone(state);
    let ui_for_transactions = Rc::clone(ui_handles);
    let latest_month_filter = dashboard.latest_month;
    let state_for_latest = Rc::clone(state);
    let ui_for_latest = Rc::clone(ui_handles);
    let state_for_rules = Rc::clone(state);
    let ui_for_rules = Rc::clone(ui_handles);
    let mut metric_cards = vec![
        ui::activatable_metric_card(
            "Transactions",
            &data.transactions.len().to_string(),
            if data.dedupe_mode.is_enabled() {
                "Imported after duplicate filtering"
            } else {
                "Imported without duplicate filtering"
            },
            move || {
                show_transactions_filter(
                    &state_for_transactions,
                    &ui_for_transactions,
                    TransactionFilter::all(),
                );
            },
        ),
        ui::activatable_metric_card(
            "Latest month",
            &signed_money(dashboard.latest_totals.balance),
            &trf("{month} balance", &[("month", latest)]),
            move || {
                if let Some(month) = latest_month_filter {
                    show_transactions_filter(
                        &state_for_latest,
                        &ui_for_latest,
                        TransactionFilter::month(month),
                    );
                }
            },
        ),
    ];
    let rules_value = data.rules_count.to_string();
    if ui_handles.advanced_features.get() {
        metric_cards.push(ui::activatable_metric_card(
            "Active rules",
            &rules_value,
            "Used for categorization",
            move || {
                show_management_dialog(
                    &ui_for_rules.window,
                    &state_for_rules,
                    &ui_for_rules,
                    "active-rules",
                );
            },
        ));
    } else {
        metric_cards.push(ui::metric_card(
            "Active rules",
            &rules_value,
            "Used for categorization",
        ));
    }
    ui_handles
        .overview
        .append(&ui::metric_grid(metric_cards, 3));

    if ui_handles.show_predictions.get()
        && prediction_scope_allows_forecast(
            data,
            selected_year,
            chrono::Local::now().date_naive().year(),
        )
    {
        append_survival_forecast(data, ui_handles, state);
    }

    let selected_year = render_year_comparison(data, ui_handles, state);
    if let Some(year) = selected_year {
        append_annual_pie_charts(data, year, ui_handles, state);
    }

    ui_handles.overview.append(&ui::section_title(
        "Trend",
        "Latest 24 months. Green is income, red is expenses, blue is balance; red points show a loss.",
    ));
    let state_for_trend = Rc::clone(state);
    let ui_for_trend = Rc::clone(ui_handles);
    ui_handles
        .overview
        .append(&ui::monthly_graph(&dashboard.monthly, move |month| {
            show_transactions_filter(
                &state_for_trend,
                &ui_for_trend,
                TransactionFilter::month(month),
            );
        }));

    if let Some(year) = selected_year {
        let mut sections = Vec::new();
        if let Some(section) = annual_budgets_section(data, year, ui_handles, state) {
            sections.push(section);
        }
        append_overview_sections(&ui_handles.overview, sections);
    }
}

fn append_overview_sections(container: &gtk::Box, mut sections: Vec<gtk::Box>) {
    if sections.iter().any(ui::is_card_list_section) {
        for section in sections {
            container.append(&section);
        }
        return;
    }

    match sections.len() {
        0 => {}
        1 => container.append(&sections.remove(0)),
        _ => container.append(&ui::responsive_columns(sections)),
    }
}

fn append_annual_pie_charts(
    data: &AppData,
    year: i32,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    let mut charts = Vec::new();
    let advanced_features = ui_handles.advanced_features.get();

    let real_year_income =
        analytics::totals_for_year(&data.transactions, &data.budgets, year).income;
    let planned_year_income = analytics::planned_year_income_total(&data.budgets, real_year_income);
    let mut planned = data
        .budgets
        .iter()
        .filter(|budget| budget.direction.is_expense())
        .map(|budget| {
            let amount = budget.annual_amount_with_basis(real_year_income, planned_year_income);
            (budget, amount)
        })
        .filter(|(_, amount)| *amount > Decimal::ZERO)
        .map(|(budget, amount)| {
            (
                budget.code.clone(),
                ui::PieSlice::new(
                    budget_display_title(&budget.code, &budget.category, advanced_features),
                    amount,
                    trf(
                        "{amount} planned",
                        &[(
                            "amount",
                            planned_budget_label(amount, &budget.annual_budget_description()),
                        )],
                    ),
                ),
            )
        })
        .collect::<Vec<_>>();
    ui::sort_pie_slices_largest_first(&mut planned);
    if !planned.is_empty() {
        let codes = planned
            .iter()
            .map(|(code, _)| code.clone())
            .collect::<Vec<_>>();
        let planned = planned
            .into_iter()
            .map(|(_, slice)| slice)
            .collect::<Vec<_>>();
        let total = planned.iter().map(|slice| slice.value).sum::<Decimal>();
        let state_for_chart = Rc::clone(state);
        let ui_for_chart = Rc::clone(ui_handles);
        charts.push(ui::pie_chart_with_capacity(
            &tr("Annual budget distribution"),
            &trf(
                "Planned annual budgets for {year}.",
                &[("year", year.to_string())],
            ),
            &planned,
            &money(total),
            Some(planned_year_income),
            move |index| {
                if let Some(code) = codes.get(index) {
                    show_transactions_filter(
                        &state_for_chart,
                        &ui_for_chart,
                        TransactionFilter::budget_for_year(code.clone(), year),
                    );
                }
            },
        ));
    }

    let categories =
        analytics::category_totals_for_year(&data.transactions, &data.budgets, year, usize::MAX);
    let actual = categories
        .iter()
        .filter(|category| category.totals.expenses > Decimal::ZERO)
        .map(|category| {
            ui::PieSlice::new(
                category.category.clone(),
                category.totals.expenses,
                category_transaction_detail(
                    category.totals.count,
                    &category.budget_code,
                    advanced_features,
                ),
            )
        })
        .collect::<Vec<_>>();
    if !actual.is_empty() {
        let category_names = categories
            .iter()
            .filter(|category| category.totals.expenses > Decimal::ZERO)
            .map(|category| category.category.clone())
            .collect::<Vec<_>>();
        let total = actual.iter().map(|slice| slice.value).sum::<Decimal>();
        let state_for_chart = Rc::clone(state);
        let ui_for_chart = Rc::clone(ui_handles);
        charts.push(ui::pie_chart_with_capacity(
            &tr("Annual spending distribution"),
            &trf(
                "Actual expenses by category for {year}.",
                &[("year", year.to_string())],
            ),
            &actual,
            &money(total),
            Some(real_year_income),
            move |index| {
                if let Some(category) = category_names.get(index) {
                    show_transactions_filter(
                        &state_for_chart,
                        &ui_for_chart,
                        TransactionFilter::category_for_year(category.clone(), year),
                    );
                }
            },
        ));
    }

    if !charts.is_empty() {
        ui_handles
            .overview
            .append(&ui::responsive_chart_columns(charts));
    }
}

fn prediction_scope_allows_forecast(
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

fn append_survival_forecast(
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
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header = ui::cancelable_dialog_header(title, period_label);

    let transactions_button = ui::primary_text_icon_button(
        "view-list-symbolic",
        "Show transactions",
        "Open the transactions used for this forecast period",
    );
    header.pack_end(&transactions_button);
    root.append(&header);

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
    root.append(&ui::action_dialog_scroll(&page));

    let dialog = adw::Dialog::builder()
        .title(tr(title))
        .content_width(620)
        .default_widget(&transactions_button)
        .child(&root)
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

pub(in crate::app) fn render_overview_search(
    data: &AppData,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
    filter: &SearchFilter,
) {
    let mut has_results = false;

    let years = data.available_years.clone();
    if let Some(year) = years.last().copied() {
        let budgets = analytics::annual_budget_usage(
            &data.transactions,
            &data.budgets,
            year,
            comparison_mode(ui_handles.as_ref()),
        )
        .into_iter()
        .filter(|budget| annual_budget_matches(budget, filter))
        .take(6)
        .collect::<Vec<_>>();
        let mut annual_sections = Vec::new();
        if !budgets.is_empty() {
            has_results = true;
            let section = ui::card_list_section_group(
                "Annual Budgets",
                &trf("Filtered on {year}.", &[("year", year.to_string())]),
            );
            let box_ = ui::card_grid(Vec::new(), 2);
            for budget in &budgets {
                append_annual_budget_row(&box_, budget, year, ui_handles, state);
            }
            section.append(&box_);
            annual_sections.push(section);
        }

        let categories = analytics::category_totals_for_year(
            &data.transactions,
            &data.budgets,
            year,
            usize::MAX,
        )
        .into_iter()
        .filter(|category| annual_category_matches(category, filter))
        .collect::<Vec<_>>();
        if !categories.is_empty() {
            has_results = true;
            annual_sections.push(annual_spending_section_from_rows(
                categories,
                year,
                &trf("Filtered on {year}.", &[("year", year.to_string())]),
                SEARCH_CATEGORY_PREVIEW_LIMIT,
                ui_handles,
                state,
            ));
        }

        append_overview_sections(&ui_handles.overview, annual_sections);
    }

    if !has_results {
        ui_handles.overview.append(&search_empty_page(
            "No results",
            "Adjust your search term or clear the search bar to see the full overview.",
        ));
    }
}

fn overview_selected_year(data: &AppData, ui_handles: &UiHandles) -> Option<i32> {
    let years = data.available_years.clone();
    let default_year = years.last().copied()?;
    Some(
        ui_handles
            .selected_year
            .get()
            .filter(|year| years.contains(year))
            .unwrap_or(default_year),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_data_with_years(years: &[i32]) -> AppData {
        AppData {
            available_years: years.to_vec(),
            ..AppData::default()
        }
    }

    #[test]
    fn prediction_scope_blocks_selected_year_when_later_year_exists() {
        let data = app_data_with_years(&[2024, 2025]);

        assert!(!prediction_scope_allows_forecast(&data, Some(2024), 2025));
    }

    #[test]
    fn prediction_scope_blocks_when_future_year_is_loaded() {
        let data = app_data_with_years(&[2026, 2027]);

        assert!(!prediction_scope_allows_forecast(&data, Some(2027), 2026));
    }

    #[test]
    fn prediction_scope_allows_latest_loaded_year_without_next_year() {
        let data = app_data_with_years(&[2024, 2025]);

        assert!(prediction_scope_allows_forecast(&data, Some(2025), 2026));
    }
}
