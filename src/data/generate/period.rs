use super::super::*;
use super::GENERATED_NOTE;
use chrono::Datelike;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn uncategorized_transactions(transactions: &[Transaction]) -> Vec<Transaction> {
    transactions
        .iter()
        .cloned()
        .map(|mut transaction| {
            transaction.category = "Uncategorized".to_string();
            transaction.budget_code.clear();
            transaction.notes.clear();
            transaction
        })
        .collect()
}

#[derive(Debug, Default, Eq, PartialEq)]
pub(super) struct GenerationPeriod {
    complete_years: BTreeSet<i32>,
    months: BTreeSet<String>,
}

impl GenerationPeriod {
    pub(super) fn contains(&self, transaction: &Transaction) -> bool {
        self.months.contains(&transaction.month_key().to_string())
    }

    pub(super) fn month_count(&self) -> usize {
        self.months.len()
    }

    pub(super) fn year_count(&self) -> usize {
        self.complete_years.len()
    }
}

pub(super) fn generation_period(transactions: &[Transaction]) -> GenerationPeriod {
    let mut months_by_year = BTreeMap::<i32, BTreeSet<u32>>::new();
    for transaction in transactions {
        months_by_year
            .entry(transaction.date.year())
            .or_default()
            .insert(transaction.date.month());
    }

    let complete_years = months_by_year
        .into_iter()
        .filter_map(|(year, months)| (months.len() == 12).then_some(year))
        .collect::<BTreeSet<_>>();
    let months = transactions
        .iter()
        .filter(|transaction| complete_years.contains(&transaction.date.year()))
        .map(|transaction| transaction.month_key().to_string())
        .collect::<BTreeSet<_>>();

    GenerationPeriod {
        complete_years,
        months,
    }
}

pub(super) fn complete_period_transactions(
    transactions: &[Transaction],
    period: &GenerationPeriod,
) -> Vec<Transaction> {
    transactions
        .iter()
        .filter(|transaction| period.contains(transaction))
        .cloned()
        .collect()
}

pub(super) fn generation_note(year_count: usize, month_count: usize) -> String {
    crate::i18n::format(
        GENERATED_NOTE,
        &[
            ("count", year_count.to_string()),
            ("months", month_count.to_string()),
            ("date", chrono::Local::now().date_naive().to_string()),
        ],
    )
}
