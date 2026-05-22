use super::month_label;
use crate::model::{AppData, MonthKey, Transaction};
use crate::util::{money, pad_table, signed_money};
use rust_decimal::Decimal;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub(super) struct Totals {
    pub(super) income: Decimal,
    pub(super) expenses: Decimal,
    pub(super) balance: Decimal,
    pub(super) count: usize,
}

impl Totals {
    pub(super) fn add(&mut self, tx: &Transaction) {
        self.count += 1;
        self.balance += tx.amount;
        if tx.amount >= Decimal::ZERO {
            self.income += tx.amount;
        } else {
            self.expenses += -tx.amount;
        }
    }
}

pub fn render_overview(data: &AppData) -> String {
    let mut out = String::new();
    out.push_str("Monthly and yearly overview\n");
    out.push_str("===========================\n\n");
    out.push_str(&format!(
        "Transactions: {} · duplicates removed: {} · duplicate filtering: {} · active rules: {}\n\n",
        data.transactions.len(),
        data.duplicate_count,
        data.dedupe_mode.label(),
        data.rules_count
    ));

    if data.transactions.is_empty() {
        out.push_str("No transactions yet. Drop CSV files onto the window or choose files with the button at the top.\n");
        return out;
    }

    let mut by_month: BTreeMap<MonthKey, Totals> = BTreeMap::new();
    let mut by_year: BTreeMap<i32, Totals> = BTreeMap::new();
    for tx in &data.transactions {
        by_month.entry(tx.month_key()).or_default().add(tx);
        by_year.entry(tx.year()).or_default().add(tx);
    }

    out.push_str("By month\n--------\n");
    let mut rows = Vec::new();
    let mut previous: Option<Totals> = None;
    for (month, totals) in by_month.iter() {
        let delta = previous
            .as_ref()
            .map(|p| signed_money(totals.balance - p.balance))
            .unwrap_or_else(|| "-".to_string());
        rows.push(vec![
            month_label(*month),
            money(totals.income),
            money(totals.expenses),
            signed_money(totals.balance),
            delta,
            totals.count.to_string(),
        ]);
        previous = Some(totals.clone());
    }
    out.push_str(&pad_table(
        &[
            "Period",
            "Income",
            "Expenses",
            "Balance",
            "Δ previous month",
            "Tx",
        ],
        &rows,
    ));

    out.push_str("\nBy year\n-------\n");
    let mut rows = Vec::new();
    let mut previous_year: Option<Totals> = None;
    for (year, totals) in by_year.iter() {
        let delta = previous_year
            .as_ref()
            .map(|p| signed_money(totals.balance - p.balance))
            .unwrap_or_else(|| "-".to_string());
        rows.push(vec![
            year.to_string(),
            money(totals.income),
            money(totals.expenses),
            signed_money(totals.balance),
            delta,
            totals.count.to_string(),
        ]);
        previous_year = Some(totals.clone());
    }
    out.push_str(&pad_table(
        &[
            "Year",
            "Income",
            "Expenses",
            "Balance",
            "Δ previous year",
            "Tx",
        ],
        &rows,
    ));

    if let Some(latest) = by_month.keys().next_back() {
        let previous = latest.previous();
        let cur = by_month.get(latest).cloned().unwrap_or_default();
        let prev = by_month.get(&previous).cloned().unwrap_or_default();
        out.push_str("\nLatest period\n-------------\n");
        out.push_str(&format!(
            "Latest month: {}. Balance {}. Difference from {}: {}.\n",
            month_label(*latest),
            signed_money(cur.balance),
            month_label(previous),
            signed_money(cur.balance - prev.balance)
        ));
    }

    out
}
