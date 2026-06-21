use super::super::*;

pub(super) struct BudgetPieChartData<'a> {
    pub(super) configured_budgets: &'a [crate::model::BudgetCode],
    pub(super) real_month_income: Decimal,
    pub(super) planned_month_income: Decimal,
    pub(super) categories: &'a [analytics::CategorySummary],
    pub(super) month: MonthKey,
    pub(super) month_label: &'a str,
}

pub(super) fn append_budget_pie_charts(
    container: &gtk::Box,
    data: BudgetPieChartData<'_>,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) -> bool {
    let configured_budgets = data.configured_budgets;
    let real_month_income = data.real_month_income;
    let planned_month_income = data.planned_month_income;
    let categories = data.categories;
    let month = data.month;
    let month_label = data.month_label;
    let mut charts = Vec::new();
    let advanced_features = ui_handles.advanced_features.get();

    let planned = planned_budget_pie_entries(
        configured_budgets,
        real_month_income,
        planned_month_income,
        advanced_features,
    );
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
            "Budget distribution",
            &trf(
                "Planned monthly budgets for {month}.",
                &[("month", month_label.to_string())],
            ),
            &planned,
            &money(total),
            Some(planned_month_income),
            move |index| {
                if let Some(code) = codes.get(index) {
                    show_transactions_filter(
                        &state_for_chart,
                        &ui_for_chart,
                        TransactionFilter::budget_for_month(code.clone(), month),
                    );
                }
            },
        ));
    }

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
        let codes = categories
            .iter()
            .filter(|category| category.totals.expenses > Decimal::ZERO)
            .map(|category| category.budget_code.clone())
            .collect::<Vec<_>>();
        let total = actual.iter().map(|slice| slice.value).sum::<Decimal>();
        let state_for_chart = Rc::clone(state);
        let ui_for_chart = Rc::clone(ui_handles);
        charts.push(ui::pie_chart_with_capacity(
            "Spending distribution",
            &trf(
                "Actual expenses by category for {month}.",
                &[("month", month_label.to_string())],
            ),
            &actual,
            &money(total),
            Some(real_month_income),
            move |index| {
                if let Some(code) = codes.get(index) {
                    show_transactions_filter(
                        &state_for_chart,
                        &ui_for_chart,
                        TransactionFilter::budget_for_month(code.clone(), month),
                    );
                }
            },
        ));
    }

    if charts.is_empty() {
        return false;
    }

    container.append(&ui::responsive_chart_columns(charts));
    true
}

fn planned_budget_pie_entries(
    configured_budgets: &[crate::model::BudgetCode],
    real_month_income: Decimal,
    planned_month_income: Decimal,
    advanced_features: bool,
) -> Vec<(String, ui::PieSlice)> {
    let mut planned = configured_budgets
        .iter()
        .filter(|budget| budget.direction.is_expense())
        .filter_map(|budget| {
            let pie_budget =
                budget.monthly_amount_with_basis(real_month_income, planned_month_income);
            (pie_budget > Decimal::ZERO).then(|| {
                (
                    budget.code.clone(),
                    ui::PieSlice::new(
                        budget_display_title(&budget.code, &budget.category, advanced_features),
                        pie_budget,
                        trf(
                            "{amount} planned",
                            &[(
                                "amount",
                                planned_budget_label(
                                    pie_budget,
                                    &budget.monthly_budget_description(),
                                ),
                            )],
                        ),
                    ),
                )
            })
        })
        .collect::<Vec<_>>();
    ui::sort_pie_slices_largest_first(&mut planned);
    planned
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BudgetAmount, BudgetCode, BudgetDirection, BudgetIncomeBasis};
    use rust_decimal::Decimal;

    fn configured_yearly_budget(code: &str, yearly: i64) -> BudgetCode {
        BudgetCode {
            code: code.to_string(),
            parent_code: String::new(),
            special: crate::model::BudgetSpecialKind::None,
            category: "Travel".to_string(),
            monthly_budget: None,
            yearly_budget: Some(BudgetAmount::Fixed(Decimal::new(yearly, 0))),
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        }
    }

    #[test]
    fn configured_yearly_budget_uses_monthly_share_for_chart_amount() {
        let budget = configured_yearly_budget("TRAVEL", 1_200);

        assert_eq!(
            budget.monthly_amount_with_basis(Decimal::ZERO, Decimal::ZERO),
            Decimal::new(100, 0)
        );
        assert_eq!(budget.monthly_budget_description(), "yearly budget / 12");
    }

    #[test]
    fn configured_monthly_budget_keeps_monthly_value_for_chart_amount() {
        let mut budget = configured_yearly_budget("GROCERIES", 1_200);
        budget.monthly_budget = Some(BudgetAmount::Fixed(Decimal::new(250, 0)));

        assert_eq!(
            budget.monthly_amount_with_basis(Decimal::ZERO, Decimal::ZERO),
            Decimal::new(250, 0)
        );
        assert_eq!(budget.monthly_budget_description(), "fixed budget");
    }

    #[test]
    fn planned_budget_pie_entries_use_direct_budgets_without_parent_rollup() {
        let mut home = configured_yearly_budget("HOME", 1_200);
        home.category = "Home".to_string();
        let mut rent = configured_yearly_budget("RENT", 12_000);
        rent.parent_code = "HOME".to_string();
        rent.category = "Rent".to_string();
        let mut utilities = configured_yearly_budget("UTIL", 2_400);
        utilities.parent_code = "HOME".to_string();
        utilities.category = "Utilities".to_string();

        let entries = planned_budget_pie_entries(
            &[home, rent, utilities],
            Decimal::ZERO,
            Decimal::ZERO,
            false,
        );

        assert_eq!(
            entries
                .iter()
                .map(|(code, slice)| (code.as_str(), slice.value))
                .collect::<Vec<_>>(),
            vec![
                ("RENT", Decimal::new(1000, 0)),
                ("UTIL", Decimal::new(200, 0)),
                ("HOME", Decimal::new(100, 0)),
            ]
        );
    }
}
