use crate::analytics::TransactionPatternAmountStat;
use crate::model::Transaction;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::collections::HashMap;

pub(super) fn amount_stats(
    indexes: &[usize],
    transactions: &[Transaction],
) -> Vec<TransactionPatternAmountStat> {
    let mut stats = HashMap::<i64, TransactionPatternAmountStat>::new();
    for index in indexes {
        let amount = transactions[*index].amount;
        let Some(cents) = amount_cents(amount) else {
            continue;
        };
        stats
            .entry(cents)
            .and_modify(|stat| stat.count += 1)
            .or_insert(TransactionPatternAmountStat { amount, count: 1 });
    }
    let mut stats = stats.into_values().collect::<Vec<_>>();
    stats.sort_by_key(|stat| std::cmp::Reverse(stat.count));
    stats
}

pub(super) fn dominant_amount(stats: &[TransactionPatternAmountStat]) -> Option<Decimal> {
    stats
        .iter()
        .max_by_key(|stat| stat.count)
        .map(|stat| stat.amount)
}

pub(super) fn amount_cents(amount: Decimal) -> Option<i64> {
    (amount * Decimal::new(100, 0)).round_dp(0).to_i64()
}
