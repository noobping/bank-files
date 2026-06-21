use super::*;
use crate::model::{BudgetAmount, BudgetDirection, BudgetIncomeBasis};
use chrono::NaiveDate;

fn tx(date: &str, amount: i64, category: &str, budget_code: &str) -> Transaction {
    Transaction {
        date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
        amount: Decimal::new(amount, 0),
        description: "Test".to_string(),
        tags: String::new(),
        counterparty: "Counterparty".to_string(),
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

mod budget_usage;
mod cash_flow;
mod dashboard;
mod diagnostics;
mod yearly;
