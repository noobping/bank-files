use super::MonthKey;
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub date: NaiveDate,
    pub amount: Decimal,
    pub description: String,
    pub counterparty: String,
    #[serde(default)]
    pub tags: String,
    pub account: String,
    pub transaction_id: String,
    pub currency: String,
    pub source_file: String,
    pub source_row: usize,
    pub category: String,
    pub budget_code: String,
    pub notes: String,
    pub strict_key: String,
    pub loose_key: String,
}

impl Transaction {
    pub fn year(&self) -> i32 {
        self.date.year()
    }

    pub fn month_key(&self) -> MonthKey {
        MonthKey::new(self.date.year(), self.date.month())
    }
}
