use super::*;

pub(in crate::app) fn render_year_comparison(
    data: &AppData,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) -> Option<i32> {
    let years = data.available_years.clone();
    let selected_year = selected_year(data, ui_handles.as_ref())?;

    ui_handles.overview.append(&ui::section_title(
        "Year",
        "Choose a year and inspect how income and expenses are built up.",
    ));
    ui_handles
        .overview
        .append(&year_selector_row(&years, selected_year, ui_handles, state));

    let totals = analytics::totals_for_year(&data.transactions, &data.budgets, selected_year);
    let state_for_balance = Rc::clone(state);
    let ui_for_balance = Rc::clone(ui_handles);
    let state_for_income = Rc::clone(state);
    let ui_for_income = Rc::clone(ui_handles);
    let state_for_expenses = Rc::clone(state);
    let ui_for_expenses = Rc::clone(ui_handles);
    ui_handles.overview.append(&ui::metric_grid(
        vec![
            ui::activatable_metric_card(
                &trf("Balance {year}", &[("year", selected_year.to_string())]),
                &signed_money(totals.balance),
                &trf(
                    "{count} transactions",
                    &[("count", totals.count.to_string())],
                ),
                move || {
                    show_transactions_filter(
                        &state_for_balance,
                        &ui_for_balance,
                        TransactionFilter::year(selected_year),
                    );
                },
            ),
            ui::activatable_metric_card(
                "Income",
                &money(totals.income),
                &selected_year.to_string(),
                move || {
                    show_transactions_filter(
                        &state_for_income,
                        &ui_for_income,
                        TransactionFilter::income_for_year(selected_year),
                    );
                },
            ),
            ui::activatable_metric_card(
                "Expenses",
                &money(totals.expenses),
                &selected_year.to_string(),
                move || {
                    show_transactions_filter(
                        &state_for_expenses,
                        &ui_for_expenses,
                        TransactionFilter::expenses_for_year(selected_year),
                    );
                },
            ),
        ],
        3,
    ));

    let Some(cash_flow) = analytics::cash_flow_breakdown_for_year(
        &data.transactions,
        &data.budgets,
        selected_year,
        comparison_mode(ui_handles.as_ref()),
        10,
    ) else {
        return Some(selected_year);
    };

    let cash_flow_section = gtk::Box::new(gtk::Orientation::Vertical, 18);
    let subtitle = if ui_handles.compare_categories_previous_period.get() {
        tr("Colored bars are the selected year; faded bars compare the previous year. Planned bars come from budgets; real bars come from transactions.")
    } else {
        tr("Colored bars show planned and real income and expenses for the selected year.")
    };
    let edit_income_button = ui::plain_text_icon_button(
        "document-edit-symbolic",
        "Planned Income",
        "Edit planned income budget",
    );
    register_exclusive_config_widget(ui_handles, &edit_income_button);
    let income_budget = data
        .budgets
        .iter()
        .find(|budget| planned_income::is_budget_code(&budget.code))
        .map(|budget| (budget.code.clone(), budget.category.clone()))
        .unwrap_or_else(|| {
            (
                planned_income::BUDGET_CODE.to_string(),
                "Income".to_string(),
            )
        });
    let state_for_income_edit = Rc::clone(state);
    let ui_for_income_edit = Rc::clone(ui_handles);
    edit_income_button.connect_clicked(move |_| {
        if config_operation_is_active(
            &ui_for_income_edit,
            "Another edit or save is already running.",
        ) {
            return;
        }
        show_budget_edit_dialog(
            &income_budget.0,
            &income_budget.1,
            &state_for_income_edit,
            &ui_for_income_edit,
        );
    });
    cash_flow_section.append(&ui::section_title_with_action(
        "Cash Flow",
        &subtitle,
        &edit_income_button,
    ));
    let state_for_chart = Rc::clone(state);
    let ui_for_chart = Rc::clone(ui_handles);
    cash_flow_section.append(&ui::year_cash_flow_chart(
        &cash_flow,
        move |kind, category, budget_code| {
            let budget_code = budget_code.trim().to_string();
            if matches!(
                kind,
                analytics::CashFlowSegmentKind::PlannedIncome
                    | analytics::CashFlowSegmentKind::PlannedExpense
            ) && !budget_code.is_empty()
            {
                if config_operation_is_active(
                    &ui_for_chart,
                    "Another edit or save is already running.",
                ) {
                    return;
                }
                show_budget_edit_dialog(&budget_code, &category, &state_for_chart, &ui_for_chart);
                return;
            }

            let filter = if budget_code.is_empty() {
                TransactionFilter::category_for_year(category, selected_year)
            } else {
                TransactionFilter::budget_for_year(budget_code, selected_year)
            };
            show_transactions_filter(&state_for_chart, &ui_for_chart, filter);
        },
    ));
    ui_handles.overview.append(&cash_flow_section);

    Some(selected_year)
}
