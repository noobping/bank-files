use super::super::*;
use super::budget_code::{budget_code_key, generated_budget_code_for_category, human_category};
use super::patterns::{generated_transfer_hint, transaction_key};
use super::period::{generation_note, GenerationPeriod};
use super::rule_fields::{best_pattern_labels, rule_candidate, rule_search_from_labels, RuleField};
use super::{GENERATED_RULE_NOTE, TRANSFER_CODE};
use crate::analytics;
use crate::model::BudgetDirection;
use chrono::Datelike;
use rust_decimal::Decimal;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

pub(super) fn append_transfer_configuration(
    transactions: &[Transaction],
    patterns: &[analytics::TransactionPattern],
    reserved_codes: &mut BTreeSet<String>,
    budgets: &mut Vec<EditableBudget>,
    rules: &mut Vec<EditableRule>,
) {
    let transfer_patterns = patterns
        .iter()
        .filter(|pattern| matches!(pattern.kind, analytics::TransactionPatternKind::Transfer))
        .collect::<Vec<_>>();
    if transfer_patterns.is_empty() {
        return;
    }

    reserved_codes.insert(budget_code_key(TRANSFER_CODE));
    budgets.push(EditableBudget {
        code: TRANSFER_CODE.to_string(),
        category: crate::i18n::gettext("Transfers"),
        monthly_budget: "0".to_string(),
        yearly_budget: String::new(),
        direction: "transfer".to_string(),
        income_basis: "real".to_string(),
        notes: crate::i18n::gettext("Generated from detected transfers."),
    });

    let mut labels = transfer_patterns
        .iter()
        .flat_map(|pattern| best_pattern_labels(pattern, transactions))
        .collect::<Vec<_>>();
    labels.sort_by_key(|label| normalize_key(label));
    labels.dedup_by(|left, right| normalize_key(left) == normalize_key(right));
    if labels.is_empty() {
        labels.push("transfer".to_string());
    }

    rules.push(EditableRule {
        priority: 200,
        active: true,
        field: "any".to_string(),
        search: rule_search_from_labels(&labels),
        is_regex: labels.len() > 1,
        category: crate::i18n::gettext("Transfers"),
        budget_code: TRANSFER_CODE.to_string(),
        direction: "transfer".to_string(),
        amount_min: String::new(),
        amount_max: String::new(),
        notes: crate::i18n::gettext(GENERATED_RULE_NOTE),
    });
}

pub(super) fn append_grouped_transaction_configuration(
    transactions: &[Transaction],
    ignored_keys: &HashSet<String>,
    period: &GenerationPeriod,
    reserved_codes: &mut BTreeSet<String>,
    budgets: &mut Vec<EditableBudget>,
    rules: &mut Vec<EditableRule>,
) {
    let mut groups = BTreeMap::<RuleGroupKey, RuleGroupStats>::new();
    for transaction in transactions {
        if transaction.amount == Decimal::ZERO
            || ignored_keys.contains(&transaction_key(transaction))
            || generated_transfer_hint(transaction)
        {
            continue;
        }
        let Some(candidate) = rule_candidate(transaction) else {
            continue;
        };
        groups
            .entry(RuleGroupKey {
                field: candidate.field,
                direction: transaction_direction(transaction.amount),
                normalized_label: normalize_key(&candidate.label),
            })
            .or_default()
            .push(transaction, candidate.label, period);
    }

    for (key, group) in groups {
        if !group.is_balanced_candidate(&key) {
            continue;
        }
        let category = group.category();
        let existing_codes = reserved_codes.iter().cloned().collect::<Vec<_>>();
        let code = generated_budget_code_for_category(&category, &existing_codes);
        reserved_codes.insert(budget_code_key(&code));
        let direction = key.direction.as_str();

        budgets.push(EditableBudget {
            code: code.clone(),
            category: category.clone(),
            monthly_budget: group.monthly_budget(period).to_string(),
            yearly_budget: group.yearly_budget(),
            direction: direction.to_string(),
            income_basis: "real".to_string(),
            notes: generation_note(period.year_count(), period.month_count()),
        });

        let labels = group.rule_labels();
        rules.push(EditableRule {
            priority: 120,
            active: true,
            field: key.field.as_str().to_string(),
            search: rule_search_from_labels(&labels),
            is_regex: labels.len() > 1,
            category,
            budget_code: code,
            direction: direction.to_string(),
            amount_min: String::new(),
            amount_max: String::new(),
            notes: crate::i18n::gettext(GENERATED_RULE_NOTE),
        });
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum GeneratedDirection {
    Expense,
    Income,
}

impl GeneratedDirection {
    fn as_str(self) -> &'static str {
        match self {
            Self::Expense => "expense",
            Self::Income => "income",
        }
    }
}

fn transaction_direction(amount: Decimal) -> GeneratedDirection {
    if amount > Decimal::ZERO {
        GeneratedDirection::Income
    } else {
        GeneratedDirection::Expense
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct RuleGroupKey {
    field: RuleField,
    direction: GeneratedDirection,
    normalized_label: String,
}

#[derive(Default)]
struct RuleGroupStats {
    period_count: usize,
    period_total: Decimal,
    year_totals: BTreeMap<i32, Decimal>,
    labels: HashMap<String, usize>,
}

impl RuleGroupStats {
    fn push(&mut self, transaction: &Transaction, label: String, period: &GenerationPeriod) {
        if period.contains(transaction) {
            let amount = transaction.amount.abs();
            self.period_count += 1;
            self.period_total += amount;
            *self.year_totals.entry(transaction.date.year()).or_default() += amount;
        }
        *self.labels.entry(label).or_default() += 1;
    }

    fn is_balanced_candidate(&self, key: &RuleGroupKey) -> bool {
        let minimum_count = match key.field {
            RuleField::Counterparty | RuleField::Tags => 2,
            RuleField::Description => 3,
        };
        self.period_count >= minimum_count && !self.year_totals.is_empty()
    }

    fn category(&self) -> String {
        self.labels
            .iter()
            .max_by(|left, right| left.1.cmp(right.1).then_with(|| right.0.cmp(left.0)))
            .map(|(label, _)| human_category(label))
            .unwrap_or_else(|| crate::i18n::gettext("Generated budget"))
    }

    fn monthly_budget(&self, period: &GenerationPeriod) -> Decimal {
        if period.month_count() == 0 {
            return Decimal::ZERO;
        }
        (self.period_total / Decimal::from(period.month_count() as u64)).round_dp(2)
    }

    fn yearly_budget(&self) -> String {
        if self.year_totals.is_empty() {
            return String::new();
        }
        let total = self
            .year_totals
            .values()
            .copied()
            .fold(Decimal::ZERO, |sum, amount| sum + amount);
        (total / Decimal::from(self.year_totals.len() as u64))
            .round_dp(2)
            .to_string()
    }

    fn rule_labels(&self) -> Vec<String> {
        let mut labels = self.labels.keys().cloned().collect::<Vec<_>>();
        labels.sort_by_key(|label| normalize_key(label));
        labels
    }
}

pub(super) fn generated_rule_order(left: &EditableRule, right: &EditableRule) -> Ordering {
    direction_rank(&left.direction)
        .cmp(&direction_rank(&right.direction))
        .then_with(|| left.category.cmp(&right.category))
        .then_with(|| left.budget_code.cmp(&right.budget_code))
}

fn direction_rank(direction: &str) -> usize {
    match BudgetDirection::from_config(direction).unwrap_or(BudgetDirection::Expense) {
        BudgetDirection::Transfer => 0,
        BudgetDirection::Income => 1,
        BudgetDirection::Expense => 2,
    }
}
