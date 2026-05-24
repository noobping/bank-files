use super::*;

pub(super) fn append_overview_sections(container: &gtk::Box, mut sections: Vec<gtk::Box>) {
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

pub(super) fn append_annual_pie_charts(
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
