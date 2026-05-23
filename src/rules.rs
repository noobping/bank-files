use crate::model::{BudgetAmount, BudgetCode, BudgetDirection, BudgetIncomeBasis, Transaction};
use crate::util::{normalize_key, parse_decimal};
use anyhow::{Context, Result};
use regex::RegexBuilder;
use rust_decimal::Decimal;
use std::fs;
use std::path::Path;

const RULE_FIELD_ALIASES: &str = include_str!("../data/defaults/rule_field_aliases.tsv");
const DIRECTION_ALIASES: &str = include_str!("../data/defaults/direction_aliases.tsv");
const FALLBACK_CATEGORIES_EN: &str = include_str!("../data/defaults/fallback_categories.tsv");
const FALLBACK_CATEGORIES_NL: &str = include_str!("../data/defaults/fallback_categories.nl.tsv");
const FALLBACK_CATEGORIES_DE: &str = include_str!("../data/defaults/fallback_categories.de.tsv");
const FALSE_ALIASES: &str = include_str!("../data/defaults/false_aliases.txt");
const DEFAULT_RULES_EN: &str = include_str!("../data/defaults/editable_rules.csv");
const DEFAULT_RULES_NL: &str = include_str!("../data/defaults/editable_rules.nl.csv");
const DEFAULT_RULES_DE: &str = include_str!("../data/defaults/editable_rules.de.csv");
const DEFAULT_BUDGETS_EN: &str = include_str!("../data/defaults/budgetcodes.csv");
const DEFAULT_BUDGETS_NL: &str = include_str!("../data/defaults/budgetcodes.nl.csv");
const DEFAULT_BUDGETS_DE: &str = include_str!("../data/defaults/budgetcodes.de.csv");

#[derive(Debug, Clone)]
pub struct Rule {
    pub priority: i32,
    pub active: bool,
    pub field: String,
    pub pattern: String,
    pub category: String,
    pub budget_code: String,
    pub direction: String,
    pub amount_min: Option<Decimal>,
    pub amount_max: Option<Decimal>,
    pub notes: String,
}

pub fn load_rules(config_dir: &Path) -> Result<Vec<Rule>> {
    let path = config_dir.join("rules.csv");
    let contents = if path.exists() {
        fs::read_to_string(&path)
            .with_context(|| format!("Could not read rules: {}", path.display()))?
    } else {
        localized_default_rules().to_string()
    };
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(contents.as_bytes());
    let headers: Vec<String> = rdr.headers()?.iter().map(|h| h.to_string()).collect();
    let mut rules = Vec::new();

    for row in rdr.records() {
        let row = row?;
        let rule = Rule {
            priority: cell(&headers, &row, "priority").parse().unwrap_or(0),
            active: parse_bool(&cell(&headers, &row, "active")),
            field: non_empty(cell(&headers, &row, "field"), "any"),
            pattern: cell(&headers, &row, "pattern"),
            category: non_empty(cell(&headers, &row, "category"), "Uncategorized"),
            budget_code: cell(&headers, &row, "budget_code"),
            direction: cell(&headers, &row, "direction"),
            amount_min: parse_decimal(&cell(&headers, &row, "amount_min")),
            amount_max: parse_decimal(&cell(&headers, &row, "amount_max")),
            notes: cell(&headers, &row, "notes"),
        };
        if !rule.pattern.trim().is_empty() {
            rules.push(rule);
        }
    }

    rules.sort_by_key(|rule| std::cmp::Reverse(rule.priority));
    Ok(rules)
}

pub fn load_budget_codes(config_dir: &Path) -> Result<Vec<BudgetCode>> {
    let path = config_dir.join("budgetcodes.csv");
    let contents = if path.exists() {
        fs::read_to_string(&path)
            .with_context(|| format!("Could not read budget codes: {}", path.display()))?
    } else {
        localized_default_budgets().to_string()
    };
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(contents.as_bytes());
    let headers: Vec<String> = rdr.headers()?.iter().map(|h| h.to_string()).collect();
    let mut codes = Vec::new();

    for row in rdr.records() {
        let row = row?;
        let code = cell(&headers, &row, "code");
        if code.trim().is_empty() {
            continue;
        }
        let category = non_empty(cell(&headers, &row, "category"), "Uncategorized");
        let direction =
            BudgetDirection::parse(&cell(&headers, &row, "direction"), &code, &category);
        codes.push(BudgetCode {
            code,
            category,
            monthly_budget: BudgetAmount::parse_optional(&cell(&headers, &row, "monthly_budget")),
            yearly_budget: BudgetAmount::parse_optional(&cell(&headers, &row, "yearly_budget")),
            direction,
            income_basis: BudgetIncomeBasis::parse(&cell(&headers, &row, "income_basis")),
            notes: cell(&headers, &row, "notes"),
        });
    }
    Ok(codes)
}

pub fn apply_rules(transactions: &mut [Transaction], rules: &[Rule], budgets: &[BudgetCode]) {
    for tx in transactions {
        let mut matched = false;
        for rule in rules.iter().filter(|r| r.active) {
            if rule_matches(rule, tx) {
                tx.category = rule.category.clone();
                tx.budget_code = rule.budget_code.clone();
                tx.notes = rule.notes.clone();
                matched = true;
                break;
            }
        }

        if !matched {
            let (category, budget_code) = fallback_category(tx, budgets);
            tx.category = category;
            tx.budget_code = budget_code;
        }
    }
}

fn rule_matches(rule: &Rule, tx: &Transaction) -> bool {
    if !direction_matches(&rule.direction, tx.amount) {
        return false;
    }
    let abs = tx.amount.abs();
    if let Some(min) = rule.amount_min {
        if abs < min.abs() {
            return false;
        }
    }
    if let Some(max) = rule.amount_max {
        if abs > max.abs() {
            return false;
        }
    }

    let text = match canonical_rule_field(&rule.field) {
        Some("description") => tx.description.clone(),
        Some("counterparty") => tx.counterparty.clone(),
        Some("tags") => transaction_tag_text(tx),
        Some("account") => tx.account.clone(),
        Some("transaction id") => tx.transaction_id.clone(),
        _ => format!(
            "{} {} {} {} {}",
            tx.description,
            tx.counterparty,
            transaction_tag_text(tx),
            tx.account,
            tx.transaction_id
        ),
    };

    let Ok(re) = RegexBuilder::new(&rule.pattern)
        .case_insensitive(true)
        .build()
    else {
        return normalize_key(&text).contains(&normalize_key(&rule.pattern));
    };
    re.is_match(&text)
}

fn transaction_tag_text(tx: &Transaction) -> String {
    let tags = tx.tags.trim();
    let description = tx.description.trim();
    match (tags.is_empty(), description.is_empty()) {
        (true, true) => String::new(),
        (true, false) => description.to_string(),
        (false, true) => tags.to_string(),
        (false, false) => format!("{tags} {description}"),
    }
}

fn direction_matches(direction: &str, amount: Decimal) -> bool {
    let Some(direction) = canonical_direction(direction) else {
        return true;
    };
    match direction {
        "expense" => amount < Decimal::ZERO,
        "income" => amount > Decimal::ZERO,
        "transfer" => true,
        _ => true,
    }
}

fn fallback_category(tx: &Transaction, budgets: &[BudgetCode]) -> (String, String) {
    let text = normalize_key(&format!(
        "{} {} {}",
        tx.description,
        tx.counterparty,
        transaction_tag_text(tx)
    ));
    let amount = tx.amount;

    for rule in fallback_category_rules() {
        if fallback_direction_matches(rule.direction, amount)
            && any_keywords(&text, rule.keywords)
            && configured_budget_code_exists(budgets, rule.budget_code)
        {
            return (rule.category.to_string(), rule.budget_code.to_string());
        }
    }

    let (category, budget_code) = if amount > Decimal::ZERO {
        other_income_fallback()
    } else {
        other_expense_fallback()
    };
    (category.to_string(), budget_code.to_string())
}

fn configured_budget_code_exists(budgets: &[BudgetCode], code: &str) -> bool {
    budgets
        .iter()
        .any(|budget| budget.code.trim().eq_ignore_ascii_case(code.trim()))
}

fn other_income_fallback() -> (&'static str, &'static str) {
    match crate::i18n::active_language() {
        crate::i18n::Language::English => ("Other income", "INC-OTHER"),
        crate::i18n::Language::Dutch => ("Overige inkomsten", "INC-OTHER"),
        crate::i18n::Language::German => ("Sonstige Einnahmen", "INC-OTHER"),
    }
}

fn other_expense_fallback() -> (&'static str, &'static str) {
    match crate::i18n::active_language() {
        crate::i18n::Language::English => ("Other", "OTHER"),
        crate::i18n::Language::Dutch => ("Overig", "OTHER"),
        crate::i18n::Language::German => ("Sonstiges", "OTHER"),
    }
}

#[derive(Clone, Copy)]
struct FallbackCategoryRule {
    category: &'static str,
    budget_code: &'static str,
    direction: &'static str,
    keywords: &'static str,
}

fn canonical_rule_field(field: &str) -> Option<&'static str> {
    canonical_from_alias_table(field, RULE_FIELD_ALIASES)
}

fn canonical_direction(direction: &str) -> Option<&'static str> {
    if normalize_key(direction).is_empty() {
        return Some("any");
    }
    canonical_from_alias_table(direction, DIRECTION_ALIASES)
}

fn canonical_from_alias_table(input: &str, table: &'static str) -> Option<&'static str> {
    let input = normalize_key(input);
    table.lines().skip(1).find_map(|line| {
        let mut cols = line.splitn(2, '\t');
        let canonical = cols.next()?.trim();
        let aliases = cols.next()?.trim();
        aliases
            .split('|')
            .any(|alias| normalize_key(alias) == input)
            .then_some(canonical)
    })
}

fn fallback_category_rules() -> impl Iterator<Item = FallbackCategoryRule> {
    fallback_categories().lines().skip(1).filter_map(|line| {
        let mut cols = line.splitn(4, '\t');
        Some(FallbackCategoryRule {
            category: cols.next()?.trim(),
            budget_code: cols.next()?.trim(),
            direction: cols.next()?.trim(),
            keywords: cols.next()?.trim(),
        })
    })
}

fn fallback_categories() -> &'static str {
    match crate::i18n::active_language() {
        crate::i18n::Language::English => FALLBACK_CATEGORIES_EN,
        crate::i18n::Language::Dutch => FALLBACK_CATEGORIES_NL,
        crate::i18n::Language::German => FALLBACK_CATEGORIES_DE,
    }
}

fn fallback_direction_matches(direction: &str, amount: Decimal) -> bool {
    match direction {
        "expense" => amount < Decimal::ZERO,
        "income" => amount > Decimal::ZERO,
        "transfer" => true,
        _ => true,
    }
}

fn any_keywords(text: &str, keywords: &str) -> bool {
    keywords
        .split('|')
        .any(|keyword| text.contains(&normalize_key(keyword)))
}

fn cell(headers: &[String], row: &csv::StringRecord, name: &str) -> String {
    headers
        .iter()
        .position(|h| h == name)
        .and_then(|idx| row.get(idx))
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn parse_bool(input: &str) -> bool {
    let input = normalize_key(input);
    !FALSE_ALIASES
        .lines()
        .any(|alias| normalize_key(alias) == input)
}

fn non_empty(value: String, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn localized_default_rules() -> &'static str {
    localized_default(DEFAULT_RULES_EN, DEFAULT_RULES_NL, DEFAULT_RULES_DE)
}

fn localized_default_budgets() -> &'static str {
    localized_default(DEFAULT_BUDGETS_EN, DEFAULT_BUDGETS_NL, DEFAULT_BUDGETS_DE)
}

fn localized_default(
    english: &'static str,
    dutch: &'static str,
    german: &'static str,
) -> &'static str {
    match crate::i18n::active_language() {
        crate::i18n::Language::English => english,
        crate::i18n::Language::Dutch => dutch,
        crate::i18n::Language::German => german,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn fallback_recognizes_mismanagement_losses() {
        let tx = tx("Mismanagement loss belegging", "Broker Demo", -230000);
        let budgets = vec![BudgetCode {
            code: "LOSS".to_string(),
            category: "Losses & fees".to_string(),
            monthly_budget: None,
            yearly_budget: None,
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        }];
        let (category, budget_code) = fallback_category(&tx, &budgets);

        assert!(matches!(
            category.as_str(),
            "Losses & fees" | "Verlies en kosten"
        ));
        assert_eq!(budget_code, "LOSS");
    }

    fn tx(description: &str, counterparty: &str, cents: i64) -> Transaction {
        Transaction {
            date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            amount: Decimal::new(cents, 2),
            description: description.to_string(),
            counterparty: counterparty.to_string(),
            tags: String::new(),
            account: "NL00TEST".to_string(),
            transaction_id: "test-id".to_string(),
            currency: "EUR".to_string(),
            source_file: "test.csv".to_string(),
            source_row: 1,
            category: String::new(),
            budget_code: String::new(),
            notes: String::new(),
            strict_key: "strict".to_string(),
            loose_key: "loose".to_string(),
        }
    }
}
