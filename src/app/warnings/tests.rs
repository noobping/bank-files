use super::evaluator::budget_attention_warnings;
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
        rule_match: None,
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
        special: crate::model::BudgetSpecialKind::None,
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
        special: crate::model::BudgetSpecialKind::None,
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
