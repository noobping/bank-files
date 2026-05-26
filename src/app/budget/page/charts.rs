use super::super::*;

pub(super) struct BudgetPieChartData<'a> {
    pub(super) budget_rows: &'a [analytics::BudgetUsage],
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
    let budget_rows = data.budget_rows;
    let configured_budgets = data.configured_budgets;
    let real_month_income = data.real_month_income;
    let planned_month_income = data.planned_month_income;
    let categories = data.categories;
    let month = data.month;
    let month_label = data.month_label;
    let mut charts = Vec::new();
    let advanced_features = ui_handles.advanced_features.get();

    let mut planned = budget_rows
        .iter()
        .filter_map(|budget| {
            let (pie_budget, pie_basis) = budget_distribution_pie_budget(
                budget,
                configured_budgets,
                real_month_income,
                planned_month_income,
            );
            (pie_budget > Decimal::ZERO).then(|| {
                (
                    budget.code.clone(),
                    ui::PieSlice::new(
                        budget_display_title(&budget.code, &budget.category, advanced_features),
                        pie_budget,
                        trf(
                            "{amount} planned",
                            &[("amount", planned_budget_label(pie_budget, &pie_basis))],
                        ),
                    ),
                )
            })
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

fn budget_distribution_pie_budget(
    budget: &analytics::BudgetUsage,
    configured_budgets: &[crate::model::BudgetCode],
    real_month_income: Decimal,
    planned_month_income: Decimal,
) -> (Decimal, String) {
    if budget.budget_basis == "remaining yearly budget" {
        if let Some(configured) = configured_budgets.iter().find(|configured| {
            configured.direction.is_expense() && configured.code.eq_ignore_ascii_case(&budget.code)
        }) {
            return (
                configured.monthly_amount_with_basis(real_month_income, planned_month_income),
                configured.monthly_budget_description(),
            );
        }
    }

    (budget.budget, budget.budget_basis.clone())
}

#[cfg(test)]
mod tests {
    use super::budget_distribution_pie_budget;
    use crate::analytics::BudgetUsage;
    use crate::model::{BudgetAmount, BudgetCode, BudgetDirection, BudgetIncomeBasis};
    use rust_decimal::Decimal;

    fn budget_usage(code: &str, budget: i64, basis: &str) -> BudgetUsage {
        BudgetUsage {
            code: code.to_string(),
            category: "Travel".to_string(),
            budget: Decimal::new(budget, 0),
            actual: Decimal::ZERO,
            remaining: Decimal::new(budget, 0),
            budget_basis: basis.to_string(),
            notes: String::new(),
        }
    }

    fn configured_yearly_budget(code: &str, yearly: i64) -> BudgetCode {
        BudgetCode {
            code: code.to_string(),
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
    fn budget_distribution_pie_uses_monthly_share_for_yearly_budget() {
        let row = budget_usage("TRAVEL", 900, "remaining yearly budget");
        let budgets = vec![configured_yearly_budget("TRAVEL", 1_200)];

        let (amount, basis) =
            budget_distribution_pie_budget(&row, &budgets, Decimal::ZERO, Decimal::ZERO);

        assert_eq!(amount, Decimal::new(100, 0));
        assert_eq!(basis, "yearly budget / 12");
        assert_eq!(row.budget, Decimal::new(900, 0));
    }

    #[test]
    fn budget_distribution_pie_keeps_monthly_budget_row_value() {
        let row = budget_usage("GROCERIES", 250, "fixed budget");
        let budgets = vec![configured_yearly_budget("GROCERIES", 1_200)];

        let (amount, basis) =
            budget_distribution_pie_budget(&row, &budgets, Decimal::ZERO, Decimal::ZERO);

        assert_eq!(amount, Decimal::new(250, 0));
        assert_eq!(basis, "fixed budget");
    }
}
