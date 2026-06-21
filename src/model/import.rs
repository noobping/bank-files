use super::{MonthKey, Transaction, TransactionLoadScope};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportReport {
    pub source: PathBuf,
    pub delimiter: char,
    pub headers: Vec<String>,
    #[serde(default)]
    pub records_total: usize,
    pub rows_seen: usize,
    pub rows_imported: usize,
    pub rows_skipped: usize,
    pub errors: Vec<String>,
    pub guessed_fields: FieldMap,
}

impl ImportReport {
    pub fn loaded_records(&self) -> usize {
        self.rows_imported + self.rows_skipped
    }

    pub fn total_records(&self) -> usize {
        self.records_total
            .max(self.loaded_records())
            .max(self.rows_seen)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldMap {
    pub date: Option<String>,
    pub amount: Option<String>,
    pub debit: Option<String>,
    pub credit: Option<String>,
    pub description: Option<String>,
    pub counterparty: Option<String>,
    pub tags: Option<String>,
    pub account: Option<String>,
    pub transaction_id: Option<String>,
    pub currency: Option<String>,
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportOutcome {
    pub transactions: Vec<Transaction>,
    pub reports: Vec<ImportReport>,
    pub warnings: Vec<String>,
    pub available_months: Vec<MonthKey>,
    pub loaded_scope: TransactionLoadScope,
}
