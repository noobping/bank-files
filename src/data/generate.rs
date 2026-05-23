use super::*;
use crate::analytics;
use crate::model::{BudgetDirection, FieldMap, ImportReport};
use chrono::Datelike;
use rust_decimal::Decimal;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

const TRANSFER_CODE: &str = "TRANSFER";
const PLANNED_INCOME_CODE: &str = "INC";
const GENERATED_NOTE: &str =
    "Generated from {count} complete imported year(s), covering {months} month(s), on {date}.";
const GENERATED_RULE_NOTE: &str = "Generated from automatic configuration.";
const SMART_INSIGHTS_REQUIRED_MESSAGE: &str = "Automatic Configuration requires Smart Insights.";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeneratedConfigurationSummary {
    pub budgets: usize,
    pub rules: usize,
    pub field_mappings: usize,
    pub ignored_patterns: usize,
    pub complete_years: usize,
    pub budget_months: usize,
}

impl GeneratedConfigurationSummary {
    pub fn is_empty(&self) -> bool {
        self.budgets == 0
            && self.rules == 0
            && self.field_mappings == 0
            && self.ignored_patterns == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedConfiguration {
    pub rules: Vec<EditableRule>,
    pub budgets: Vec<EditableBudget>,
    pub aliases: Vec<EditableAlias>,
    pub ignored_patterns: Vec<IgnoredTransactionPattern>,
    pub summary: GeneratedConfigurationSummary,
}

pub fn generate_automatic_configuration(
    data: &AppData,
    smart_insights_enabled: bool,
) -> Result<GeneratedConfiguration> {
    ensure_smart_insights_for_automatic_configuration(smart_insights_enabled)?;

    let transactions = uncategorized_transactions(&data.transactions);
    let period = generation_period(&transactions);
    let analysis_transactions = complete_period_transactions(&transactions, &period);
    let patterns = analytics::transaction_pattern_analysis(
        &analysis_transactions,
        data.dedupe_mode.is_enabled(),
    );
    let ignored_patterns = generated_ignored_patterns(&patterns.patterns);
    let mut excluded_keys = cancellation_pattern_keys(&patterns.patterns);
    excluded_keys.extend(transfer_pattern_keys(&patterns.patterns));

    let mut reserved_codes = BTreeSet::<String>::new();
    let mut budgets = Vec::new();
    let mut rules = Vec::new();
    append_transfer_configuration(
        &analysis_transactions,
        &patterns.patterns,
        &mut reserved_codes,
        &mut budgets,
        &mut rules,
    );
    append_grouped_transaction_configuration(
        &analysis_transactions,
        &excluded_keys,
        &period,
        &mut reserved_codes,
        &mut budgets,
        &mut rules,
    );

    rules.sort_by(generated_rule_order);
    for (offset, rule) in rules.iter_mut().enumerate() {
        rule.priority = 200 - offset as i32;
    }

    let (aliases, generated_alias_count) = generated_field_aliases(&data.reports)?;
    let summary = GeneratedConfigurationSummary {
        budgets: budgets.len(),
        rules: rules.len(),
        field_mappings: generated_alias_count,
        ignored_patterns: ignored_patterns.len(),
        complete_years: period.year_count(),
        budget_months: period.month_count(),
    };

    Ok(GeneratedConfiguration {
        rules,
        budgets,
        aliases,
        ignored_patterns,
        summary,
    })
}

pub fn generated_budget_code_for_category(category: &str, existing_codes: &[String]) -> String {
    let reserved = existing_codes
        .iter()
        .map(|code| budget_code_key(code))
        .collect::<HashSet<_>>();
    generated_budget_code_with_reserved(category, &reserved)
}

fn ensure_smart_insights_for_automatic_configuration(smart_insights_enabled: bool) -> Result<()> {
    if smart_insights_enabled {
        Ok(())
    } else {
        anyhow::bail!(SMART_INSIGHTS_REQUIRED_MESSAGE)
    }
}

fn append_transfer_configuration(
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

fn append_grouped_transaction_configuration(
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

fn generated_ignored_patterns(
    patterns: &[analytics::TransactionPattern],
) -> Vec<IgnoredTransactionPattern> {
    let mut ignored = patterns
        .iter()
        .filter(|pattern| {
            matches!(
                pattern.kind,
                analytics::TransactionPatternKind::FullRefund
                    | analytics::TransactionPatternKind::BillSplit
            )
        })
        .map(|pattern| IgnoredTransactionPattern {
            key: analytics::transaction_pattern_key(pattern),
            label: pattern.label.trim().to_string(),
        })
        .collect::<Vec<_>>();
    ignored.sort_by(|left, right| left.label.cmp(&right.label).then(left.key.cmp(&right.key)));
    ignored
}

fn cancellation_pattern_keys(patterns: &[analytics::TransactionPattern]) -> HashSet<String> {
    patterns
        .iter()
        .filter(|pattern| {
            matches!(
                pattern.kind,
                analytics::TransactionPatternKind::FullRefund
                    | analytics::TransactionPatternKind::BillSplit
            )
        })
        .flat_map(|pattern| pattern.transaction_keys.iter().cloned())
        .collect()
}

fn transfer_pattern_keys(patterns: &[analytics::TransactionPattern]) -> HashSet<String> {
    patterns
        .iter()
        .filter(|pattern| matches!(pattern.kind, analytics::TransactionPatternKind::Transfer))
        .flat_map(|pattern| pattern.transaction_keys.iter().cloned())
        .collect()
}

fn generated_field_aliases(reports: &[ImportReport]) -> Result<(Vec<EditableAlias>, usize)> {
    let mut aliases = parse_editable_aliases(default_aliases())?;
    let mut keys = aliases
        .iter()
        .map(alias_key)
        .collect::<HashSet<(String, String)>>();
    let initial_count = aliases.len();

    for report in reports.iter().filter(|report| report.rows_imported > 0) {
        for (canonical, alias) in field_map_aliases(&report.guessed_fields) {
            let alias = alias.trim();
            if alias.is_empty() {
                continue;
            }
            let candidate = EditableAlias {
                canonical: canonical.to_string(),
                alias: alias.to_string(),
            };
            if keys.insert(alias_key(&candidate)) {
                aliases.push(candidate);
            }
        }
    }

    aliases.sort_by(|left, right| {
        left.canonical
            .cmp(&right.canonical)
            .then_with(|| left.alias.cmp(&right.alias))
    });
    let generated_count = aliases.len().saturating_sub(initial_count);
    Ok((aliases, generated_count))
}

fn field_map_aliases(fields: &FieldMap) -> Vec<(&'static str, &str)> {
    [
        ("date", fields.date.as_deref()),
        ("amount", fields.amount.as_deref()),
        ("debit", fields.debit.as_deref()),
        ("credit", fields.credit.as_deref()),
        ("description", fields.description.as_deref()),
        ("counterparty", fields.counterparty.as_deref()),
        ("tags", fields.tags.as_deref()),
        ("account", fields.account.as_deref()),
        ("transaction_id", fields.transaction_id.as_deref()),
        ("currency", fields.currency.as_deref()),
        ("direction", fields.direction.as_deref()),
    ]
    .into_iter()
    .filter_map(|(canonical, alias)| alias.map(|alias| (canonical, alias)))
    .collect()
}

fn alias_key(alias: &EditableAlias) -> (String, String) {
    (normalize_key(&alias.canonical), normalize_key(&alias.alias))
}

fn uncategorized_transactions(transactions: &[Transaction]) -> Vec<Transaction> {
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
struct GenerationPeriod {
    complete_years: BTreeSet<i32>,
    months: BTreeSet<String>,
}

impl GenerationPeriod {
    fn contains(&self, transaction: &Transaction) -> bool {
        self.months.contains(&transaction.month_key().to_string())
    }

    fn month_count(&self) -> usize {
        self.months.len()
    }

    fn year_count(&self) -> usize {
        self.complete_years.len()
    }
}

fn generation_period(transactions: &[Transaction]) -> GenerationPeriod {
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

fn complete_period_transactions(
    transactions: &[Transaction],
    period: &GenerationPeriod,
) -> Vec<Transaction> {
    transactions
        .iter()
        .filter(|transaction| period.contains(transaction))
        .cloned()
        .collect()
}

fn generation_note(year_count: usize, month_count: usize) -> String {
    crate::i18n::format(
        GENERATED_NOTE,
        &[
            ("count", year_count.to_string()),
            ("months", month_count.to_string()),
            ("date", chrono::Local::now().date_naive().to_string()),
        ],
    )
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

fn generated_transfer_hint(transaction: &Transaction) -> bool {
    let text = normalize_key(&format!(
        "{} {} {} {} {}",
        transaction.account,
        transaction.counterparty,
        transaction.description,
        transaction.tags,
        transaction.notes
    ));
    [
        "transfer",
        "transfers",
        "internal transfer",
        "overboeking",
        "overboekingen",
        "ueberweisung",
        "uberweisung",
        "umbuchung",
    ]
    .iter()
    .any(|hint| text.contains(hint))
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct RuleGroupKey {
    field: RuleField,
    direction: GeneratedDirection,
    normalized_label: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum RuleField {
    Counterparty,
    Tags,
    Description,
}

impl RuleField {
    fn as_str(self) -> &'static str {
        match self {
            Self::Counterparty => "counterparty",
            Self::Tags => "tags",
            Self::Description => "description",
        }
    }
}

struct RuleCandidate {
    field: RuleField,
    label: String,
}

fn rule_candidate(transaction: &Transaction) -> Option<RuleCandidate> {
    [
        (RuleField::Counterparty, transaction.counterparty.as_str()),
        (RuleField::Tags, transaction.tags.as_str()),
        (RuleField::Description, transaction.description.as_str()),
    ]
    .into_iter()
    .find_map(|(field, label)| {
        let label = clean_label(label);
        meaningful_label(&label).then_some(RuleCandidate { field, label })
    })
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

fn generated_rule_order(left: &EditableRule, right: &EditableRule) -> Ordering {
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

fn best_pattern_labels(
    pattern: &analytics::TransactionPattern,
    transactions: &[Transaction],
) -> Vec<String> {
    let matched = transactions
        .iter()
        .filter(|transaction| analytics::transaction_matches_pattern(transaction, pattern))
        .collect::<Vec<_>>();
    let mut labels = matched
        .iter()
        .filter_map(|transaction| rule_candidate(transaction).map(|candidate| candidate.label))
        .collect::<Vec<_>>();
    if labels.is_empty() {
        labels.extend(pattern.match_labels.iter().cloned());
    }
    labels
}

fn rule_search_from_labels(labels: &[String]) -> String {
    if labels.len() == 1 {
        return labels[0].trim().to_string();
    }
    let terms = labels
        .iter()
        .map(|label| regex_term(label))
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>();
    format!("(?:{})", terms.join("|"))
}

fn regex_term(label: &str) -> String {
    normalize_key(label)
        .split_whitespace()
        .map(regex::escape)
        .collect::<Vec<_>>()
        .join(r"\W+")
}

fn clean_label(label: &str) -> String {
    label.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn meaningful_label(label: &str) -> bool {
    let normalized = normalize_key(label);
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() {
        return false;
    }
    let meaningful = tokens
        .iter()
        .filter(|token| meaningful_token(token))
        .count();
    meaningful > 0
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
    digits == 0 || digits * 2 < token.chars().count()
}

fn human_category(label: &str) -> String {
    let label = clean_label(label);
    if label.chars().any(|character| character.is_lowercase()) {
        return label;
    }
    label
        .to_ascii_lowercase()
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn budget_code_key(code: &str) -> String {
    code.trim().to_ascii_lowercase()
}

fn generated_budget_code_with_reserved(category: &str, reserved: &HashSet<String>) -> String {
    let base = generated_budget_code_base(category);
    let mut candidate = base.clone();
    let mut suffix = 2;
    while budget_code_is_unavailable(&candidate, reserved) {
        candidate = format!("{base}-{suffix}");
        suffix += 1;
    }
    candidate
}

fn generated_budget_code_base(category: &str) -> String {
    let mut code = String::new();
    let mut last_was_separator = false;
    for ch in category.chars() {
        if ch.is_ascii_alphanumeric() {
            code.push(ch.to_ascii_uppercase());
            last_was_separator = false;
        } else if !code.is_empty() && !last_was_separator {
            code.push('-');
            last_was_separator = true;
        }
    }
    let code = code.trim_matches('-');
    if code.is_empty() {
        "BUDGET".to_string()
    } else {
        code.to_string()
    }
}

fn budget_code_is_unavailable(code: &str, reserved: &HashSet<String>) -> bool {
    code.trim().eq_ignore_ascii_case(PLANNED_INCOME_CODE)
        || reserved.contains(&budget_code_key(code))
}

fn transaction_key(transaction: &Transaction) -> String {
    format!(
        "{}\u{1f}{}",
        transaction.source_file, transaction.source_row
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn empty_input_generates_no_user_configuration() {
        let generated = generate_automatic_configuration(&AppData::default(), true).unwrap();

        assert!(generated.summary.is_empty());
        assert!(generated.rules.is_empty());
        assert!(generated.budgets.is_empty());
        assert!(generated.ignored_patterns.is_empty());
    }

    #[test]
    fn repeated_merchant_generates_budget_and_rule() {
        let data = app_data(complete_year_transactions(
            2025,
            -100,
            "Coffee",
            "Coffee Shop",
        ));

        let generated = generate_automatic_configuration(&data, true).unwrap();

        assert_eq!(generated.summary.complete_years, 1);
        assert_eq!(generated.summary.budget_months, 12);
        assert_eq!(generated.summary.budgets, 1);
        assert_eq!(generated.summary.rules, 1);
        assert_eq!(generated.budgets[0].category, "Coffee Shop");
        assert_eq!(generated.budgets[0].code, "COFFEE-SHOP");
        assert_eq!(generated.budgets[0].direction, "expense");
        assert_eq!(generated.budgets[0].monthly_budget, "1.00");
        assert_eq!(generated.budgets[0].yearly_budget, "12.00");
        assert_eq!(generated.rules[0].field, "counterparty");
        assert_eq!(generated.rules[0].search, "Coffee Shop");
        assert_eq!(generated.rules[0].budget_code, "COFFEE-SHOP");
    }

    #[test]
    fn incomplete_year_transactions_do_not_generate_budget_amounts() {
        let data = app_data(vec![
            tx("2026-01-03", -500, "Coffee", "Coffee Shop", 1),
            tx("2026-01-10", -700, "Coffee", "Coffee Shop", 2),
        ]);

        let generated = generate_automatic_configuration(&data, true).unwrap();

        assert_eq!(generated.summary.complete_years, 0);
        assert_eq!(generated.summary.budget_months, 0);
        assert!(generated.budgets.is_empty());
        assert!(generated.rules.is_empty());
    }

    #[test]
    fn complete_year_budgets_compare_years_with_average_yearly_amount() {
        let mut transactions = complete_year_transactions(2024, -100, "Coffee", "Coffee Shop");
        transactions.extend(complete_year_transactions(
            2025,
            -200,
            "Coffee",
            "Coffee Shop",
        ));
        let data = app_data(transactions);

        let generated = generate_automatic_configuration(&data, true).unwrap();

        assert_eq!(generated.summary.complete_years, 2);
        assert_eq!(generated.summary.budget_months, 24);
        assert_eq!(generated.budgets[0].monthly_budget, "1.50");
        assert_eq!(generated.budgets[0].yearly_budget, "18.00");
    }

    #[test]
    fn detected_transfer_generates_only_transfer_configuration() {
        let data = app_data(complete_year_transfer_pairs(2025));

        let generated = generate_automatic_configuration(&data, true).unwrap();

        assert_eq!(generated.budgets.len(), 1);
        assert_eq!(generated.budgets[0].code, TRANSFER_CODE);
        assert_eq!(generated.budgets[0].direction, "transfer");
        assert_eq!(generated.rules.len(), 1);
        assert_eq!(generated.rules[0].budget_code, TRANSFER_CODE);
        assert_eq!(generated.rules[0].direction, "transfer");
        assert!(generated.ignored_patterns.is_empty());
    }

    #[test]
    fn automatic_configuration_requires_smart_insights() {
        let data = app_data(complete_year_transfer_pairs(2025));

        let error = generate_automatic_configuration(&data, false).unwrap_err();

        assert!(format!("{error:#}").contains(SMART_INSIGHTS_REQUIRED_MESSAGE));
    }

    #[test]
    fn refund_patterns_are_ignored_but_transfers_are_not() {
        let mut transactions = complete_year_refund_pairs(2025);
        transactions.extend(complete_year_transfer_pairs(2025));
        let data = app_data(transactions);

        let generated = generate_automatic_configuration(&data, true).unwrap();

        assert!(generated.summary.ignored_patterns > 0);
        assert!(generated
            .ignored_patterns
            .iter()
            .any(|pattern| pattern.key.starts_with("refund:")));
        assert!(!generated
            .ignored_patterns
            .iter()
            .any(|pattern| pattern.key.starts_with("transfer:")));
    }

    #[test]
    fn detected_field_mappings_are_added_to_default_aliases() {
        let mut data = AppData::default();
        data.reports.push(ImportReport {
            rows_imported: 1,
            guessed_fields: FieldMap {
                date: Some("Booking Date Custom".to_string()),
                amount: Some("Money Column Custom".to_string()),
                ..Default::default()
            },
            ..Default::default()
        });

        let generated = generate_automatic_configuration(&data, true).unwrap();

        assert_eq!(generated.summary.field_mappings, 2);
        assert!(generated
            .aliases
            .iter()
            .any(|alias| { alias.canonical == "date" && alias.alias == "Booking Date Custom" }));
        assert!(generated
            .aliases
            .iter()
            .any(|alias| { alias.canonical == "amount" && alias.alias == "Money Column Custom" }));
    }

    #[test]
    fn existing_transaction_category_and_code_are_not_merged() {
        let mut transactions = complete_year_transactions(2025, -100, "Coffee", "Coffee Shop");
        for transaction in &mut transactions {
            transaction.category = "Old Category".to_string();
            transaction.budget_code = "OLD".to_string();
        }

        let generated = generate_automatic_configuration(&app_data(transactions), true).unwrap();

        assert!(!generated.budgets.iter().any(|budget| budget.code == "OLD"));
        assert!(!generated.rules.iter().any(|rule| rule.budget_code == "OLD"));
    }

    #[test]
    fn generated_budget_code_uses_readable_category_slug() {
        assert_eq!(
            generated_budget_code_for_category("Dining out & coffee", &[]),
            "DINING-OUT-COFFEE"
        );
        assert_eq!(generated_budget_code_for_category("!!!", &[]), "BUDGET");
    }

    #[test]
    fn generated_budget_code_avoids_existing_and_reserved_codes() {
        let existing = vec!["DINING".to_string(), "DINING-2".to_string()];
        assert_eq!(
            generated_budget_code_for_category("Dining", &existing),
            "DINING-3"
        );
        assert_eq!(generated_budget_code_for_category("Inc", &[]), "INC-2");
    }

    fn app_data(transactions: Vec<Transaction>) -> AppData {
        AppData {
            transactions,
            dedupe_mode: DedupeMode::Disabled,
            ..Default::default()
        }
    }

    fn complete_year_transactions(
        year: i32,
        cents_per_month: i64,
        description: &str,
        counterparty: &str,
    ) -> Vec<Transaction> {
        (1..=12)
            .map(|month| {
                tx(
                    &format!("{year}-{month:02}-03"),
                    cents_per_month,
                    description,
                    counterparty,
                    month as usize,
                )
            })
            .collect()
    }

    fn complete_year_transfer_pairs(year: i32) -> Vec<Transaction> {
        (1..=12)
            .flat_map(|month| {
                let base_row = 1_000 + month as usize * 10;
                [
                    account_tx(
                        &format!("{year}-{month:02}-03"),
                        -10000,
                        "Transfer to savings",
                        "Savings",
                        "Checking",
                        base_row + 1,
                    ),
                    account_tx(
                        &format!("{year}-{month:02}-04"),
                        10000,
                        "Transfer from checking",
                        "Checking",
                        "Savings",
                        base_row + 2,
                    ),
                ]
            })
            .collect()
    }

    fn complete_year_refund_pairs(year: i32) -> Vec<Transaction> {
        (1..=12)
            .flat_map(|month| {
                let base_row = 2_000 + month as usize * 10;
                [
                    tx(
                        &format!("{year}-{month:02}-06"),
                        -2500,
                        "Store purchase",
                        "Store",
                        base_row + 1,
                    ),
                    tx(
                        &format!("{year}-{month:02}-08"),
                        2500,
                        "Refund Store",
                        "Store",
                        base_row + 2,
                    ),
                ]
            })
            .collect()
    }

    fn tx(
        date: &str,
        cents: i64,
        description: &str,
        counterparty: &str,
        row: usize,
    ) -> Transaction {
        account_tx(date, cents, description, counterparty, "Checking", row)
    }

    fn account_tx(
        date: &str,
        cents: i64,
        description: &str,
        counterparty: &str,
        account: &str,
        row: usize,
    ) -> Transaction {
        Transaction {
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            amount: Decimal::new(cents, 2),
            description: description.to_string(),
            counterparty: counterparty.to_string(),
            tags: String::new(),
            account: account.to_string(),
            transaction_id: format!("id-{row}"),
            currency: "EUR".to_string(),
            source_file: "test.csv".to_string(),
            source_row: row,
            category: "Uncategorized".to_string(),
            budget_code: String::new(),
            notes: String::new(),
            strict_key: format!("strict-{row}"),
            loose_key: format!("loose-{row}"),
        }
    }
}
