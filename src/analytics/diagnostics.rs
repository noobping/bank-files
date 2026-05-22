use super::*;
use rust_decimal::prelude::ToPrimitive;
use std::collections::{HashMap, HashSet};

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

pub struct TransactionPatternAnalysis {
    pub patterns: Vec<TransactionPattern>,
    hidden_transaction_keys: HashSet<String>,
}

impl TransactionPatternAnalysis {
    pub fn hidden_canceled_transaction_count(&self) -> usize {
        self.hidden_transaction_keys.len()
    }
}

pub fn transaction_pattern_analysis(
    transactions: &[Transaction],
    group_patterns: bool,
) -> TransactionPatternAnalysis {
    let mut patterns = repeating_transaction_patterns(transactions);
    let transfer_patterns = transfer_transaction_patterns(transactions);
    let transfer_keys = transfer_patterns
        .iter()
        .flat_map(|pattern| pattern.transaction_keys.iter().cloned())
        .collect::<HashSet<_>>();
    let offsetting_patterns = offsetting_transaction_patterns(transactions)
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
        group_transaction_patterns(patterns)
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

fn group_transaction_patterns(patterns: Vec<TransactionPattern>) -> Vec<TransactionPattern> {
    let mut grouped: HashMap<(String, String), TransactionPattern> = HashMap::new();
    for pattern in patterns {
        let key = (
            transaction_pattern_kind_key(pattern.kind).to_string(),
            normalized_text(&pattern.label),
        );
        grouped
            .entry(key)
            .and_modify(|existing| {
                existing.count += pattern.count;
                existing.net += pattern.net;
                existing.first_date = existing.first_date.min(pattern.first_date);
                existing.last_date = existing.last_date.max(pattern.last_date);
                merge_amount_stats(&mut existing.amount_stats, &pattern.amount_stats);
                merge_match_labels(&mut existing.match_labels, &pattern.match_labels);
                merge_match_labels(&mut existing.transaction_keys, &pattern.transaction_keys);
                if let Some(amount) = dominant_amount(&existing.amount_stats) {
                    existing.amount = amount;
                }
            })
            .or_insert(pattern);
    }
    let mut patterns = grouped.into_values().collect::<Vec<_>>();
    for pattern in &mut patterns {
        pattern
            .amount_stats
            .sort_by_key(|stat| std::cmp::Reverse(stat.count));
    }
    patterns
}

fn merge_amount_stats(
    existing: &mut Vec<TransactionPatternAmountStat>,
    incoming: &[TransactionPatternAmountStat],
) {
    for stat in incoming {
        if let Some(existing_stat) = existing.iter_mut().find(|item| item.amount == stat.amount) {
            existing_stat.count += stat.count;
        } else {
            existing.push(stat.clone());
        }
    }
}

fn dominant_amount(stats: &[TransactionPatternAmountStat]) -> Option<Decimal> {
    stats
        .iter()
        .max_by_key(|stat| stat.count)
        .map(|stat| stat.amount)
}

fn merge_match_labels(existing: &mut Vec<String>, incoming: &[String]) {
    for label in incoming {
        if !existing.iter().any(|item| item == label) {
            existing.push(label.clone());
        }
    }
}

pub fn transaction_pattern_key(pattern: &TransactionPattern) -> String {
    format!(
        "{}:{}",
        transaction_pattern_kind_key(pattern.kind),
        normalized_text(&pattern.label)
    )
}

fn transaction_pattern_kind_key(kind: TransactionPatternKind) -> &'static str {
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

pub fn transactions_without_canceled_patterns(transactions: &[Transaction]) -> Vec<Transaction> {
    let hidden_transaction_keys = hidden_canceled_transaction_keys(transactions);
    if hidden_transaction_keys.is_empty() {
        return transactions.to_vec();
    }

    transactions
        .iter()
        .filter(|transaction| !hidden_transaction_keys.contains(&transaction_key(transaction)))
        .cloned()
        .collect()
}

fn hidden_canceled_transaction_keys(transactions: &[Transaction]) -> HashSet<String> {
    offsetting_transaction_patterns(transactions)
        .into_iter()
        .flat_map(|pattern| pattern.transaction_keys)
        .collect()
}

pub fn transaction_matches_pattern(
    transaction: &Transaction,
    pattern: &TransactionPattern,
) -> bool {
    if !pattern.transaction_keys.is_empty() {
        return pattern
            .transaction_keys
            .iter()
            .any(|key| key == &transaction_key(transaction));
    }
    if transaction.date < pattern.first_date || transaction.date > pattern.last_date {
        return false;
    }
    let amount_matches = pattern
        .amount_stats
        .iter()
        .any(|stat| stat.amount == transaction.amount || stat.amount == -transaction.amount);
    if !amount_matches {
        return false;
    }
    let labels = transaction_pattern_match_labels(transaction);
    pattern
        .match_labels
        .iter()
        .any(|label| labels.iter().any(|candidate| candidate == label))
}

fn transaction_key(transaction: &Transaction) -> String {
    format!(
        "{}\u{1f}{}",
        transaction.source_file, transaction.source_row
    )
}

fn repeating_transaction_patterns(transactions: &[Transaction]) -> Vec<TransactionPattern> {
    let mut groups: HashMap<String, (String, Vec<usize>)> = HashMap::new();
    for (index, transaction) in transactions.iter().enumerate() {
        let mut seen = HashSet::new();
        for label in transaction_pattern_labels(transaction) {
            let key = normalized_text(&label);
            if key.is_empty() || !seen.insert(key.clone()) {
                continue;
            }
            groups
                .entry(key)
                .or_insert_with(|| (label.trim().to_string(), Vec::new()))
                .1
                .push(index);
        }
    }

    let mut patterns = Vec::new();
    for (_, (label, mut indexes)) in groups {
        if indexes.len() < 2 {
            continue;
        }
        indexes.sort_by_key(|index| transactions[*index].date);
        let gaps = indexes
            .windows(2)
            .map(|window| {
                transactions[window[1]]
                    .date
                    .signed_duration_since(transactions[window[0]].date)
                    .num_days()
            })
            .collect::<Vec<_>>();
        let cadence = if indexes.len() == 2 {
            repeating_cadence(&gaps).filter(|cadence| *cadence == RepeatingCadence::Yearly)
        } else {
            repeating_cadence(&gaps).or_else(|| recurring_cadence(&indexes, transactions))
        };
        let Some(cadence) = cadence else {
            continue;
        };
        let first = indexes[0];
        let last = indexes[indexes.len() - 1];
        let net = indexes.iter().fold(Decimal::ZERO, |total, index| {
            total + transactions[*index].amount
        });
        let amount_stats = amount_stats(&indexes, transactions);
        let match_labels = indexes
            .iter()
            .flat_map(|index| transaction_pattern_match_labels(&transactions[*index]))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let amount = dominant_amount(&amount_stats).unwrap_or(transactions[first].amount);
        patterns.push(TransactionPattern {
            kind: TransactionPatternKind::Repeating(cadence),
            label,
            match_labels,
            transaction_keys: indexes
                .iter()
                .map(|index| transaction_key(&transactions[*index]))
                .collect(),
            count: indexes.len(),
            amount,
            amount_stats,
            net,
            first_date: transactions[first].date,
            last_date: transactions[last].date,
        });
    }

    patterns.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(right.last_date.cmp(&left.last_date))
            .then(left.label.cmp(&right.label))
    });
    patterns.truncate(8);
    patterns
}

fn amount_stats(
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

fn transfer_transaction_patterns(transactions: &[Transaction]) -> Vec<TransactionPattern> {
    let mut positives_by_amount: HashMap<i64, Vec<usize>> = HashMap::new();
    let mut expenses = Vec::new();
    for (index, transaction) in transactions.iter().enumerate() {
        let Some(cents) = amount_cents(transaction.amount) else {
            continue;
        };
        if cents > 0 {
            positives_by_amount.entry(cents).or_default().push(index);
        } else if cents < 0 {
            expenses.push(index);
        }
    }

    let mut used = HashSet::new();
    let mut patterns = Vec::new();
    for expense_index in expenses {
        if used.contains(&expense_index) {
            continue;
        }
        let expense = &transactions[expense_index];
        let Some(target) = amount_cents(-expense.amount) else {
            continue;
        };
        let Some(income_index) = positives_by_amount.get(&target).and_then(|candidates| {
            candidates
                .iter()
                .copied()
                .filter(|index| !used.contains(index))
                .filter(|index| days_between(expense.date, transactions[*index].date) <= 5)
                .filter(|index| transfer_pair_score(expense, &transactions[*index]) >= 2)
                .min_by_key(|index| days_between(expense.date, transactions[*index].date))
        }) else {
            continue;
        };

        let income = &transactions[income_index];
        let first_date = expense.date.min(income.date);
        let last_date = expense.date.max(income.date);
        patterns.push(TransactionPattern {
            kind: TransactionPatternKind::Transfer,
            label: transfer_label(expense, income),
            match_labels: transaction_pattern_pair_match_labels(expense, income),
            transaction_keys: vec![transaction_key(expense), transaction_key(income)],
            count: 2,
            amount: -expense.amount,
            amount_stats: amount_stats(&[expense_index, income_index], transactions),
            net: expense.amount + income.amount,
            first_date,
            last_date,
        });
        used.insert(expense_index);
        used.insert(income_index);
    }

    patterns.truncate(8);
    patterns
}

fn offsetting_transaction_patterns(transactions: &[Transaction]) -> Vec<TransactionPattern> {
    let mut positives_by_amount: HashMap<i64, Vec<usize>> = HashMap::new();
    let mut expenses = Vec::new();
    for (index, transaction) in transactions.iter().enumerate() {
        let Some(cents) = amount_cents(transaction.amount) else {
            continue;
        };
        if cents > 0 {
            positives_by_amount.entry(cents).or_default().push(index);
        } else if cents < 0 {
            expenses.push(index);
        }
    }
    expenses.sort_by_key(|index| transactions[*index].date);

    let mut used = HashSet::new();
    let mut patterns = Vec::new();
    for expense_index in expenses {
        if used.contains(&expense_index) {
            continue;
        }
        let expense = &transactions[expense_index];
        let Some(target) = amount_cents(-expense.amount) else {
            continue;
        };

        if let Some(refund_index) = refund_match(
            transactions,
            &positives_by_amount,
            &used,
            expense_index,
            target,
        ) {
            let refund = &transactions[refund_index];
            let first_date = expense.date.min(refund.date);
            let last_date = expense.date.max(refund.date);
            patterns.push(TransactionPattern {
                kind: TransactionPatternKind::FullRefund,
                label: display_label(expense),
                match_labels: transaction_pattern_pair_match_labels(expense, refund),
                transaction_keys: vec![transaction_key(expense), transaction_key(refund)],
                count: 2,
                amount: -expense.amount,
                amount_stats: amount_stats(&[expense_index, refund_index], transactions),
                net: expense.amount + refund.amount,
                first_date,
                last_date,
            });
            used.insert(expense_index);
            used.insert(refund_index);
            continue;
        }

        let candidates = split_candidates(transactions, &used, expense_index, target);
        if let Some(split_indexes) = subset_sum(&candidates, target) {
            let mut all_indexes = split_indexes.clone();
            all_indexes.push(expense_index);
            let first_date = all_indexes
                .iter()
                .map(|index| transactions[*index].date)
                .min()
                .unwrap_or(expense.date);
            let last_date = all_indexes
                .iter()
                .map(|index| transactions[*index].date)
                .max()
                .unwrap_or(expense.date);
            let net = all_indexes.iter().fold(Decimal::ZERO, |total, index| {
                total + transactions[*index].amount
            });
            patterns.push(TransactionPattern {
                kind: TransactionPatternKind::BillSplit,
                label: display_label(expense),
                match_labels: all_indexes
                    .iter()
                    .flat_map(|index| transaction_pattern_match_labels(&transactions[*index]))
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
                transaction_keys: all_indexes
                    .iter()
                    .map(|index| transaction_key(&transactions[*index]))
                    .collect(),
                count: all_indexes.len(),
                amount: -expense.amount,
                amount_stats: amount_stats(&all_indexes, transactions),
                net,
                first_date,
                last_date,
            });
            used.insert(expense_index);
            for index in split_indexes {
                used.insert(index);
            }
        }
    }

    patterns.truncate(8);
    patterns
}

fn refund_match(
    transactions: &[Transaction],
    positives_by_amount: &HashMap<i64, Vec<usize>>,
    used: &HashSet<usize>,
    expense_index: usize,
    target: i64,
) -> Option<usize> {
    let expense = &transactions[expense_index];
    let expense_label = normalized_label(expense);
    positives_by_amount.get(&target).and_then(|candidates| {
        candidates
            .iter()
            .copied()
            .filter(|index| !used.contains(index))
            .filter(|index| days_between(expense.date, transactions[*index].date) <= 45)
            .filter(|index| {
                labels_related(&expense_label, &normalized_label(&transactions[*index]))
                    || looks_like_refund(&transactions[*index])
            })
            .min_by_key(|index| days_between(expense.date, transactions[*index].date))
    })
}

fn split_candidates(
    transactions: &[Transaction],
    used: &HashSet<usize>,
    expense_index: usize,
    target: i64,
) -> Vec<(usize, i64)> {
    let expense = &transactions[expense_index];
    let mut candidates = transactions
        .iter()
        .enumerate()
        .filter(|(index, transaction)| {
            if *index == expense_index
                || used.contains(index)
                || transaction.amount <= Decimal::ZERO
            {
                return false;
            }
            let days_after = transaction
                .date
                .signed_duration_since(expense.date)
                .num_days();
            (-2..=21).contains(&days_after)
        })
        .filter_map(|(index, transaction)| {
            amount_cents(transaction.amount)
                .filter(|amount| *amount > 0 && *amount < target)
                .map(|amount| (index, amount))
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|candidate| std::cmp::Reverse(candidate.1));
    candidates.truncate(12);
    candidates
}

fn subset_sum(candidates: &[(usize, i64)], target: i64) -> Option<Vec<usize>> {
    fn search(
        candidates: &[(usize, i64)],
        target: i64,
        start: usize,
        total: i64,
        chosen: &mut Vec<usize>,
    ) -> Option<Vec<usize>> {
        if chosen.len() >= 2 && (total - target).abs() <= 1 {
            return Some(chosen.clone());
        }
        if total >= target || chosen.len() >= 6 {
            return None;
        }
        for offset in start..candidates.len() {
            chosen.push(candidates[offset].0);
            if let Some(result) = search(
                candidates,
                target,
                offset + 1,
                total + candidates[offset].1,
                chosen,
            ) {
                return Some(result);
            }
            chosen.pop();
        }
        None
    }

    search(candidates, target, 0, 0, &mut Vec::new())
}

fn repeating_cadence(gaps: &[i64]) -> Option<RepeatingCadence> {
    if gaps.is_empty() {
        return None;
    }
    let required = (gaps.len() * 2).div_ceil(3);
    [
        (RepeatingCadence::Weekly, 5_i64, 9_i64),
        (RepeatingCadence::Biweekly, 12_i64, 16_i64),
        (RepeatingCadence::Monthly, 24_i64, 38_i64),
        (RepeatingCadence::Quarterly, 70_i64, 110_i64),
        (RepeatingCadence::Yearly, 320_i64, 410_i64),
    ]
    .into_iter()
    .find(|(_, min, max)| {
        gaps.iter()
            .filter(|gap| (*min..=*max).contains(&**gap))
            .count()
            >= required.max(1)
    })
    .map(|(cadence, _, _)| cadence)
}

fn recurring_cadence(indexes: &[usize], transactions: &[Transaction]) -> Option<RepeatingCadence> {
    let months = indexes
        .iter()
        .map(|index| transactions[*index].month_key().to_string())
        .collect::<HashSet<_>>();
    (months.len() >= 3).then_some(RepeatingCadence::Recurring)
}

fn amount_cents(amount: Decimal) -> Option<i64> {
    (amount * Decimal::new(100, 0)).round_dp(0).to_i64()
}

fn days_between(left: chrono::NaiveDate, right: chrono::NaiveDate) -> i64 {
    left.signed_duration_since(right).num_days().abs()
}

fn labels_related(left: &str, right: &str) -> bool {
    !left.is_empty()
        && !right.is_empty()
        && (left == right
            || (left.len() >= 4 && right.contains(left))
            || (right.len() >= 4 && left.contains(right)))
}

fn looks_like_refund(transaction: &Transaction) -> bool {
    let text = format!(
        "{} {} {} {}",
        transaction.counterparty, transaction.description, transaction.tags, transaction.notes
    )
    .to_ascii_lowercase();
    [
        "refund",
        "reversal",
        "retour",
        "terug",
        "restitutie",
        "erstattung",
    ]
    .iter()
    .any(|word| text.contains(word))
}

fn normalized_label(transaction: &Transaction) -> String {
    transaction_pattern_labels(transaction)
        .into_iter()
        .map(|label| normalized_text(&label))
        .find(|label| !label.is_empty())
        .unwrap_or_default()
}

fn transaction_pattern_pair_match_labels(left: &Transaction, right: &Transaction) -> Vec<String> {
    transaction_pattern_match_labels(left)
        .into_iter()
        .chain(transaction_pattern_match_labels(right))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

fn transaction_pattern_match_labels(transaction: &Transaction) -> Vec<String> {
    transaction_pattern_labels(transaction)
        .into_iter()
        .map(|label| normalized_text(&label))
        .filter(|label| !label.is_empty())
        .collect()
}

fn transaction_pattern_labels(transaction: &Transaction) -> Vec<String> {
    let mut labels = Vec::new();
    push_label(&mut labels, &transaction.tags);
    push_label(&mut labels, &transaction.counterparty);
    push_label(&mut labels, &description_tag_text(&transaction.description));
    push_label(&mut labels, &transaction.description);
    push_label(&mut labels, &transaction.budget_code);
    push_label(&mut labels, &transaction.category);
    labels
}

fn push_label(labels: &mut Vec<String>, label: &str) {
    let label = label.trim();
    if label.is_empty() {
        return;
    }
    let normalized = normalized_text(label);
    if normalized.is_empty()
        || labels
            .iter()
            .any(|existing| normalized_text(existing) == normalized)
    {
        return;
    }
    labels.push(label.to_string());
}

fn normalized_text(raw: &str) -> String {
    raw.chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .filter(|token| meaningful_token(token))
        .take(8)
        .collect::<Vec<_>>()
        .join(" ")
}

fn meaningful_token(token: &str) -> bool {
    const NOISE: &[&str] = &[
        "afschrijving",
        "betaling",
        "beschrijving",
        "card",
        "description",
        "id",
        "ideal",
        "iban",
        "incasso",
        "kenmerk",
        "machtiging",
        "mandate",
        "message",
        "nummer",
        "omschrijving",
        "pas",
        "payment",
        "reference",
        "ref",
        "sepa",
        "transaction",
        "transactie",
    ];
    if token.len() <= 1 || NOISE.contains(&token) {
        return false;
    }
    let digits = token
        .chars()
        .filter(|character| character.is_ascii_digit())
        .count();
    if digits == 0 {
        return true;
    }
    let len = token.chars().count();
    digits * 2 < len && len <= 18
}

fn display_label(transaction: &Transaction) -> String {
    transaction_pattern_labels(transaction)
        .into_iter()
        .find(|label| !normalized_text(label).is_empty())
        .unwrap_or_else(|| "Transaction".to_string())
}

fn description_tag_text(description: &str) -> String {
    normalized_text(description)
}

fn transfer_pair_score(left: &Transaction, right: &Transaction) -> usize {
    let mut score = 0;
    let left_account = normalized_text(&left.account);
    let right_account = normalized_text(&right.account);
    if !left_account.is_empty() && !right_account.is_empty() && left_account != right_account {
        score += 2;
    }
    if looks_like_transfer(left) || looks_like_transfer(right) {
        score += 2;
    }
    if labels_related(&normalized_label(left), &normalized_label(right)) {
        score += 1;
    }
    score
}

fn looks_like_transfer(transaction: &Transaction) -> bool {
    let text = normalized_text(&format!(
        "{} {} {} {} {}",
        transaction.account,
        transaction.counterparty,
        transaction.description,
        transaction.tags,
        transaction.notes
    ));
    [
        "overboeking",
        "overboekingen",
        "transfer",
        "transfers",
        "sparen",
        "spaarrekening",
        "savings",
        "saving",
        "internal",
        "eigen rekening",
        "rekening",
        "wise",
        "revolut",
        "bunq",
        "paypal",
        "ueberweisung",
        "uberweisung",
        "umbuchung",
    ]
    .iter()
    .any(|word| text.contains(word))
}

fn transfer_label(left: &Transaction, right: &Transaction) -> String {
    [
        left.counterparty.trim(),
        right.counterparty.trim(),
        description_tag_text(&left.description).trim(),
        description_tag_text(&right.description).trim(),
        left.account.trim(),
        right.account.trim(),
    ]
    .into_iter()
    .find(|label| !label.is_empty())
    .unwrap_or("Transfer")
    .to_string()
}
