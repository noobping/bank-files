use super::core::{tr, trf};
use crate::analytics;
use crate::model::{AppData, BudgetCode, ComparisonMode, MonthKey, Transaction};
use crate::util::money;
use rust_decimal::Decimal;
use std::collections::HashMap;

pub(in crate::app) struct AttentionWarning {
    title: &'static str,
    message: String,
}

impl AttentionWarning {
    fn new(title: &'static str, message: String) -> Self {
        Self { title, message }
    }

    fn titled_message(&self) -> String {
        format!("{}: {}", tr(self.title), self.message)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::app) struct BudgetWarningTotals {
    pub(in crate::app) real_expenses: Decimal,
    pub(in crate::app) real_income: Decimal,
    pub(in crate::app) planned_expenses: Decimal,
    pub(in crate::app) planned_income: Decimal,
    pub(in crate::app) annual_budget_room_used: Decimal,
}

pub(in crate::app) fn monthly_budget_attention_warnings(
    data: &AppData,
    month: MonthKey,
) -> Vec<AttentionWarning> {
    let real_totals = analytics::totals_for_month(&data.transactions, &data.budgets, month);
    let planned_income = analytics::planned_month_income_total(&data.budgets, real_totals.income);

    budget_attention_warnings(BudgetWarningTotals {
        real_expenses: real_totals.expenses,
        real_income: real_totals.income,
        planned_expenses: monthly_planned_expense_total(
            &data.budgets,
            real_totals.income,
            planned_income,
        ),
        planned_income,
        annual_budget_room_used: yearly_only_budget_room_used(
            &data.transactions,
            &data.budgets,
            month,
        ),
    })
}

pub(in crate::app) fn annual_budget_attention_warnings(
    data: &AppData,
    year: i32,
) -> Vec<AttentionWarning> {
    let real_totals = analytics::totals_for_year(&data.transactions, &data.budgets, year);
    let planned_income = analytics::planned_year_income_total(&data.budgets, real_totals.income);
    let budget_rows = analytics::annual_budget_usage(
        &data.transactions,
        &data.budgets,
        year,
        ComparisonMode::CurrentOnly,
    );

    budget_attention_warnings(BudgetWarningTotals {
        real_expenses: real_totals.expenses,
        real_income: real_totals.income,
        planned_expenses: positive_budget_total(budget_rows.iter().map(|budget| budget.budget)),
        planned_income,
        annual_budget_room_used: Decimal::ZERO,
    })
}

pub(in crate::app) fn attention_warning_messages(warnings: &[AttentionWarning]) -> Vec<String> {
    warnings
        .iter()
        .map(AttentionWarning::titled_message)
        .collect()
}

pub(in crate::app) fn attention_warning_card_message(
    warnings: &[AttentionWarning],
) -> Option<String> {
    match warnings {
        [] => None,
        [warning] => Some(warning.message.clone()),
        warnings => Some(attention_warning_messages(warnings).join(
            "
",
        )),
    }
}

fn budget_attention_warnings(totals: BudgetWarningTotals) -> Vec<AttentionWarning> {
    [
        actual_spending_warning(totals)
            .map(|message| AttentionWarning::new("Spending is above income", message)),
        planned_expenses_warning(totals)
            .map(|message| AttentionWarning::new("Check your budget", message)),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn planned_expenses_warning(totals: BudgetWarningTotals) -> Option<String> {
    if totals.planned_expenses <= Decimal::ZERO || totals.planned_expenses <= totals.planned_income
    {
        return None;
    }

    if totals.planned_income <= Decimal::ZERO {
        return Some(trf(
            "Planned expenses total {expenses}, but this period has no planned income.",
            &[("expenses", money(totals.planned_expenses))],
        ));
    }

    Some(trf(
        "Planned expenses total {expenses}, above planned income of {income} by {overage}.",
        &[
            ("expenses", money(totals.planned_expenses)),
            ("income", money(totals.planned_income)),
            (
                "overage",
                money(totals.planned_expenses - totals.planned_income),
            ),
        ],
    ))
}

fn actual_spending_warning(totals: BudgetWarningTotals) -> Option<String> {
    let threshold = totals.real_income.max(totals.planned_income);
    let annual_budget_room = totals
        .annual_budget_room_used
        .max(Decimal::ZERO)
        .min(totals.real_expenses.max(Decimal::ZERO));
    let spending_limit = threshold + annual_budget_room;
    if totals.real_expenses <= spending_limit {
        return None;
    }

    let loss = totals.real_expenses - spending_limit;
    if annual_budget_room > Decimal::ZERO {
        return Some(actual_spending_above_income_and_annual_room_warning(
            totals,
            threshold,
            annual_budget_room,
            loss,
        ));
    }

    if threshold <= Decimal::ZERO {
        return Some(trf(
            "Expenses are {expenses}, with no real or planned income in this period.",
            &[("expenses", money(totals.real_expenses))],
        ));
    }

    if totals.real_income >= totals.planned_income {
        return Some(trf(
            "Expenses are {expenses}, above real income of {income} by {loss}.",
            &[
                ("expenses", money(totals.real_expenses)),
                ("income", money(totals.real_income)),
                ("loss", money(totals.real_expenses - totals.real_income)),
            ],
        ));
    }

    Some(trf(
        "Expenses are {expenses}, above planned income of {income} by {loss}.",
        &[
            ("expenses", money(totals.real_expenses)),
            ("income", money(totals.planned_income)),
            ("loss", money(totals.real_expenses - totals.planned_income)),
        ],
    ))
}

fn actual_spending_above_income_and_annual_room_warning(
    totals: BudgetWarningTotals,
    threshold: Decimal,
    annual_budget_room: Decimal,
    loss: Decimal,
) -> String {
    if threshold <= Decimal::ZERO {
        return trf(
            "Expenses are {expenses}, above annual budget room of {room} by {loss}.",
            &[
                ("expenses", money(totals.real_expenses)),
                ("room", money(annual_budget_room)),
                ("loss", money(loss)),
            ],
        );
    }

    if totals.real_income >= totals.planned_income {
        return trf(
            "Expenses are {expenses}, above real income of {income} plus annual budget room of {room} by {loss}.",
            &[
                ("expenses", money(totals.real_expenses)),
                ("income", money(totals.real_income)),
                ("room", money(annual_budget_room)),
                ("loss", money(loss)),
            ],
        );
    }

    trf(
        "Expenses are {expenses}, above planned income of {income} plus annual budget room of {room} by {loss}.",
        &[
            ("expenses", money(totals.real_expenses)),
            ("income", money(totals.planned_income)),
            ("room", money(annual_budget_room)),
            ("loss", money(loss)),
        ],
    )
}

fn positive_budget_total<I>(budget_amounts: I) -> Decimal
where
    I: IntoIterator<Item = Decimal>,
{
    budget_amounts
        .into_iter()
        .fold(Decimal::ZERO, |total, budget| {
            total + budget.max(Decimal::ZERO)
        })
}

fn monthly_planned_expense_total(
    budgets: &[BudgetCode],
    real_month_income: Decimal,
    planned_month_income: Decimal,
) -> Decimal {
    positive_budget_total(
        budgets
            .iter()
            .filter(|budget| budget.direction.is_expense())
            .filter(|budget| budget.monthly_budget.is_some())
            .map(|budget| {
                budget.monthly_amount_with_basis(real_month_income, planned_month_income)
            }),
    )
}

fn yearly_only_budget_room_used(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    month: MonthKey,
) -> Decimal {
    let real_year_income = analytics::totals_for_year(transactions, budgets, month.year).income;
    let planned_year_income = analytics::planned_year_income_total(budgets, real_year_income);
    let mut current_actual_by_code: HashMap<String, Decimal> = HashMap::new();
    let mut earlier_actual_by_code: HashMap<String, Decimal> = HashMap::new();

    for tx in transactions
        .iter()
        .filter(|tx| tx.year() == month.year && tx.amount < Decimal::ZERO)
        .filter(|tx| !analytics::transaction_is_transfer(tx, budgets))
    {
        if tx.budget_code.trim().is_empty() {
            continue;
        }
        let tx_month = tx.month_key();
        if tx_month == month {
            *current_actual_by_code
                .entry(tx.budget_code.clone())
                .or_default() += -tx.amount;
        } else if tx_month < month {
            *earlier_actual_by_code
                .entry(tx.budget_code.clone())
                .or_default() += -tx.amount;
        }
    }

    budgets
        .iter()
        .filter(|budget| yearly_only_expense_budget(budget))
        .map(|budget| {
            let annual_budget = budget
                .annual_amount_with_basis(real_year_income, planned_year_income)
                .max(Decimal::ZERO);
            let earlier_actual = earlier_actual_by_code
                .get(&budget.code)
                .copied()
                .unwrap_or(Decimal::ZERO);
            let current_actual = current_actual_by_code
                .get(&budget.code)
                .copied()
                .unwrap_or(Decimal::ZERO);
            current_actual.min((annual_budget - earlier_actual).max(Decimal::ZERO))
        })
        .sum()
}

fn yearly_only_expense_budget(budget: &BudgetCode) -> bool {
    budget.direction.is_expense()
        && budget.monthly_budget.is_none()
        && budget.yearly_budget.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        AppData, BudgetAmount, BudgetCode, BudgetDirection, BudgetIncomeBasis, Transaction,
    };
    use chrono::NaiveDate;

    fn dec(value: i64) -> Decimal {
        Decimal::new(value, 0)
    }

    fn warning_titles(totals: BudgetWarningTotals) -> Vec<&'static str> {
        budget_attention_warnings(totals)
            .into_iter()
            .map(|warning| warning.title)
            .collect()
    }

    fn warning_totals(
        real_expenses: i64,
        real_income: i64,
        planned_expenses: i64,
        planned_income: i64,
    ) -> BudgetWarningTotals {
        BudgetWarningTotals {
            real_expenses: dec(real_expenses),
            real_income: dec(real_income),
            planned_expenses: dec(planned_expenses),
            planned_income: dec(planned_income),
            annual_budget_room_used: Decimal::ZERO,
        }
    }

    fn tx(date: &str, amount: i64, category: &str, budget_code: &str) -> Transaction {
        Transaction {
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            amount: dec(amount),
            description: "Test".to_string(),
            counterparty: "Counterparty".to_string(),
            tags: String::new(),
            account: "NL00TEST".to_string(),
            transaction_id: format!("{date}-{amount}"),
            currency: "EUR".to_string(),
            source_file: "test.csv".to_string(),
            source_row: 1,
            category: category.to_string(),
            budget_code: budget_code.to_string(),
            notes: String::new(),
            strict_key: format!("{date}-{amount}-strict"),
            loose_key: format!("{date}-{amount}-loose"),
        }
    }

    fn budget(
        code: &str,
        category: &str,
        direction: BudgetDirection,
        monthly_budget: i64,
    ) -> BudgetCode {
        BudgetCode {
            code: code.to_string(),
            category: category.to_string(),
            monthly_budget: Some(BudgetAmount::Fixed(dec(monthly_budget))),
            yearly_budget: None,
            direction,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        }
    }

    fn yearly_budget(code: &str, category: &str, yearly_budget: i64) -> BudgetCode {
        BudgetCode {
            code: code.to_string(),
            category: category.to_string(),
            monthly_budget: None,
            yearly_budget: Some(BudgetAmount::Fixed(dec(yearly_budget))),
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        }
    }

    fn app_data(transactions: Vec<Transaction>, budgets: Vec<BudgetCode>) -> AppData {
        AppData {
            transactions,
            budgets,
            ..AppData::default()
        }
    }

    #[test]
    fn planned_expenses_above_planned_income_warns() {
        let titles = warning_titles(warning_totals(100, 1_000, 1_200, 1_000));

        assert_eq!(titles, vec!["Check your budget"]);
    }

    #[test]
    fn planned_expenses_at_or_below_planned_income_do_not_warn() {
        let titles = warning_titles(warning_totals(100, 1_000, 1_000, 1_000));

        assert!(titles.is_empty());
    }

    #[test]
    fn real_expenses_above_real_and_planned_income_warn() {
        let titles = warning_titles(warning_totals(1_200, 800, 0, 1_000));

        assert_eq!(titles, vec!["Spending is above income"]);
    }

    #[test]
    fn real_expenses_above_real_but_below_planned_income_do_not_warn() {
        let titles = warning_titles(warning_totals(900, 800, 0, 1_000));

        assert!(titles.is_empty());
    }

    #[test]
    fn real_expenses_above_planned_but_below_real_income_do_not_warn() {
        let titles = warning_titles(warning_totals(900, 1_000, 0, 800));

        assert!(titles.is_empty());
    }

    #[test]
    fn planned_expenses_without_planned_income_warns_clearly() {
        let warnings = budget_attention_warnings(warning_totals(0, 0, 50, 0));

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].title, "Check your budget");
        assert!(!warnings[0].message.trim().is_empty());
    }

    #[test]
    fn monthly_warnings_use_income_budgets_as_planned_income() {
        let data = app_data(
            vec![tx("2025-03-01", 500, "Income", "INC")],
            vec![
                budget("INC", "Income", BudgetDirection::Income, 1_000),
                budget("FOOD", "Food", BudgetDirection::Expense, 900),
            ],
        );

        let warnings = monthly_budget_attention_warnings(&data, MonthKey::new(2025, 3));

        assert!(warnings.is_empty());
    }

    #[test]
    fn monthly_warnings_ignore_yearly_only_budgets_as_planned_monthly_expenses() {
        let data = app_data(
            Vec::new(),
            vec![
                budget("INC", "Income", BudgetDirection::Income, 1_000),
                yearly_budget("TRAVEL", "Travel", 1_200),
            ],
        );

        let warnings = monthly_budget_attention_warnings(&data, MonthKey::new(2025, 3));

        assert!(warnings.is_empty());
    }

    #[test]
    fn monthly_warnings_allow_spending_covered_by_yearly_budget_room() {
        let data = app_data(
            vec![
                tx("2025-03-01", 1_000, "Income", "INC"),
                tx("2025-03-02", -1_200, "Travel", "TRAVEL"),
            ],
            vec![
                budget("INC", "Income", BudgetDirection::Income, 1_000),
                yearly_budget("TRAVEL", "Travel", 1_200),
            ],
        );

        let warnings = monthly_budget_attention_warnings(&data, MonthKey::new(2025, 3));

        assert!(warnings.is_empty());
    }

    #[test]
    fn monthly_warnings_warn_for_spending_above_yearly_budget_room() {
        let data = app_data(
            vec![
                tx("2025-01-02", -200, "Travel", "TRAVEL"),
                tx("2025-03-02", -900, "Travel", "TRAVEL"),
            ],
            vec![yearly_budget("TRAVEL", "Travel", 1_000)],
        );

        let warnings = monthly_budget_attention_warnings(&data, MonthKey::new(2025, 3));

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].title, "Spending is above income");
        assert!(!warnings[0].message.trim().is_empty());
    }

    #[test]
    fn annual_warnings_use_expense_budgets_as_planned_expenses() {
        let data = app_data(
            Vec::new(),
            vec![
                budget("INC", "Income", BudgetDirection::Income, 100),
                budget("FOOD", "Food", BudgetDirection::Expense, 110),
            ],
        );

        let warnings = annual_budget_attention_warnings(&data, 2025);

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].title, "Check your budget");
    }

    #[test]
    fn transfer_transactions_do_not_create_false_warnings() {
        let data = app_data(
            vec![tx("2025-03-01", -2_000, "Transfer", "TRANSFER")],
            vec![budget("TRANSFER", "Transfer", BudgetDirection::Transfer, 0)],
        );

        let warnings = monthly_budget_attention_warnings(&data, MonthKey::new(2025, 3));

        assert!(warnings.is_empty());
    }
}
