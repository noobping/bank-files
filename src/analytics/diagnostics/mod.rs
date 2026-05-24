use crate::analytics::{
    transaction_is_transfer, RepeatingCadence, TransactionPattern, TransactionPatternKind,
};
use crate::model::{BudgetCode, Transaction};

use rust_decimal::Decimal;
use std::collections::HashSet;

mod amounts;
mod grouping;
mod labels;
mod matching;
mod offsetting;
mod repeating;
mod transfers;

pub use matching::{transaction_matches_pattern, transactions_without_canceled_patterns};

#[derive(Default)]
pub struct TransactionPatternAnalysis {
    pub patterns: Vec<TransactionPattern>,
    hidden_transaction_keys: HashSet<String>,
}

impl TransactionPatternAnalysis {
    pub fn hidden_canceled_transaction_count(&self) -> usize {
        self.hidden_transaction_keys.len()
    }
}

pub fn unconfigured_expense_budget_count(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
) -> usize {
    transactions
        .iter()
        .filter(|tx| transaction_has_unconfigured_expense_budget(tx, budgets))
        .count()
}

pub fn transaction_has_unconfigured_expense_budget(
    tx: &Transaction,
    budgets: &[BudgetCode],
) -> bool {
    if tx.amount >= Decimal::ZERO || transaction_is_transfer(tx, budgets) {
        return false;
    }

    let code = tx.budget_code.trim();
    code.is_empty()
        || !budgets.iter().any(|budget| {
            budget.direction.is_expense() && budget.code.trim().eq_ignore_ascii_case(code)
        })
}

pub fn other_category_count(transactions: &[Transaction]) -> usize {
    transactions
        .iter()
        .filter(|tx| matches!(tx.budget_code.trim(), "OTHER" | "INC-OTHER"))
        .count()
}

pub fn transaction_pattern_analysis(
    transactions: &[Transaction],
    group_patterns: bool,
) -> TransactionPatternAnalysis {
    let mut patterns = repeating::repeating_transaction_patterns(transactions);
    let transfer_patterns = transfers::transfer_transaction_patterns(transactions);
    let transfer_keys = transfer_patterns
        .iter()
        .flat_map(|pattern| pattern.transaction_keys.iter().cloned())
        .collect::<HashSet<_>>();
    let offsetting_patterns = offsetting::offsetting_transaction_patterns(transactions)
        .into_iter()
        .filter(|pattern| {
            !pattern
                .transaction_keys
                .iter()
                .any(|key| transfer_keys.contains(key))
        })
        .collect::<Vec<_>>();
    let hidden_transaction_keys = offsetting_patterns
        .iter()
        .flat_map(|pattern| pattern.transaction_keys.iter().cloned())
        .collect::<HashSet<_>>();
    patterns.extend(transfer_patterns);
    patterns.extend(offsetting_patterns);
    let mut patterns = if group_patterns {
        grouping::group_transaction_patterns(patterns)
    } else {
        patterns
    };
    patterns.sort_by(|left, right| {
        right
            .last_date
            .cmp(&left.last_date)
            .then(right.count.cmp(&left.count))
            .then(left.label.cmp(&right.label))
    });
    patterns.truncate(12);
    TransactionPatternAnalysis {
        patterns,
        hidden_transaction_keys,
    }
}

pub fn transaction_pattern_key(pattern: &TransactionPattern) -> String {
    format!(
        "{}:{}",
        transaction_pattern_kind_key(pattern.kind),
        labels::normalized_text(&pattern.label)
    )
}

pub(super) fn transaction_pattern_kind_key(kind: TransactionPatternKind) -> &'static str {
    match kind {
        TransactionPatternKind::Repeating(RepeatingCadence::Weekly) => "repeating-weekly",
        TransactionPatternKind::Repeating(RepeatingCadence::Biweekly) => "repeating-biweekly",
        TransactionPatternKind::Repeating(RepeatingCadence::Monthly) => "repeating-monthly",
        TransactionPatternKind::Repeating(RepeatingCadence::Quarterly) => "repeating-quarterly",
        TransactionPatternKind::Repeating(RepeatingCadence::Yearly) => "repeating-yearly",
        TransactionPatternKind::Repeating(RepeatingCadence::Recurring) => "repeating-recurring",
        TransactionPatternKind::FullRefund => "refund",
        TransactionPatternKind::BillSplit => "bill-split",
        TransactionPatternKind::Transfer => "transfer",
    }
}

pub(super) fn transaction_key(transaction: &Transaction) -> String {
    format!(
        "{}\u{1f}{}",
        transaction.source_file, transaction.source_row
    )
}

pub(super) fn days_between(left: chrono::NaiveDate, right: chrono::NaiveDate) -> i64 {
    left.signed_duration_since(right).num_days().abs()
}
