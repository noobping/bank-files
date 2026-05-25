use super::*;
mod search;
mod sections;

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
        search::render_overview_search(data, ui_handles, state, &filter);
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
    ui_handles
        .overview
        .append(&ui::metric_grid(metric_cards, 3));

    let selected_year = render_year_comparison(data, ui_handles, state);
    if let Some(year) = selected_year {
        sections::append_annual_pie_charts(data, year, ui_handles, state);
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
        sections::append_overview_sections(&ui_handles.overview, sections);
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
