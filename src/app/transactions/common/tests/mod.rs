use crate::model::{BudgetCode, BudgetDirection, BudgetIncomeBasis, Transaction};
use chrono::NaiveDate;
use rust_decimal::Decimal;

mod actions;
mod budget_direction;
mod budget_targets;
mod text;

pub(super) fn tx(amount: i64, budget_code: &str, category: &str) -> Transaction {
    Transaction {
        date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        amount: Decimal::new(amount, 0),
        description: "Test transaction".to_string(),
        counterparty: String::new(),
        tags: String::new(),
        account: String::new(),
        transaction_id: String::new(),
        currency: "EUR".to_string(),
        source_file: "test.csv".to_string(),
        source_row: 2,
        category: category.to_string(),
        budget_code: budget_code.to_string(),
        notes: String::new(),
        strict_key: String::new(),
        loose_key: String::new(),
        rule_match: None,
    }
}

pub(super) fn budget(code: &str, direction: BudgetDirection) -> BudgetCode {
    BudgetCode {
        code: code.to_string(),
        category: code.to_string(),
        monthly_budget: None,
        yearly_budget: None,
        direction,
        income_basis: BudgetIncomeBasis::RealIncome,
        notes: String::new(),
    }
}
