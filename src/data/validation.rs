use super::*;
use crate::model::{BudgetDirection, BudgetIncomeBasis};

pub(in crate::data) fn validate_editable_rules(rules: &[EditableRule]) -> Result<()> {
    for (index, rule) in rules.iter().enumerate() {
        if rule.search.trim().is_empty() {
            continue;
        }
        if rule.is_regex {
            regex::RegexBuilder::new(&rule.search)
                .case_insensitive(true)
                .build()
                .with_context(|| format!("Rule {} has an invalid regex", index + 1))?;
        }
        validate_optional_decimal(&rule.amount_min)
            .with_context(|| format!("Rule {} has an invalid minimum amount", index + 1))?;
        validate_optional_decimal(&rule.amount_max)
            .with_context(|| format!("Rule {} has an invalid maximum amount", index + 1))?;
    }
    Ok(())
}

pub(in crate::data) fn validate_editable_budgets(budgets: &[EditableBudget]) -> Result<()> {
    for (index, budget) in budgets.iter().enumerate() {
        validate_budget_amount(&budget.monthly_budget)
            .with_context(|| format!("Budget {} has an invalid monthly budget", index + 1))?;
        validate_budget_amount(&budget.yearly_budget)
            .with_context(|| format!("Budget {} has an invalid yearly budget", index + 1))?;
        validate_budget_direction(&budget.direction)
            .with_context(|| format!("Budget {} has an invalid direction", index + 1))?;
        validate_budget_income_basis(&budget.income_basis)
            .with_context(|| format!("Budget {} has an invalid income basis", index + 1))?;
    }
    Ok(())
}

pub(in crate::data) fn validate_budget_amount(input: &str) -> Result<()> {
    if BudgetAmount::parse(input).is_some() {
        Ok(())
    } else {
        anyhow::bail!("Invalid budget amount: {input}")
    }
}

pub(in crate::data) fn validate_budget_direction(input: &str) -> Result<()> {
    if BudgetDirection::is_valid_config(input) {
        Ok(())
    } else {
        anyhow::bail!("Invalid budget direction: {input}")
    }
}

pub(in crate::data) fn validate_budget_income_basis(input: &str) -> Result<()> {
    if BudgetIncomeBasis::is_valid_config(input) {
        Ok(())
    } else {
        anyhow::bail!("Invalid budget income basis: {input}")
    }
}

pub(in crate::data) fn validate_optional_decimal(input: &str) -> Result<()> {
    if input.trim().is_empty() || parse_decimal(input).is_some() {
        Ok(())
    } else {
        anyhow::bail!("Invalid amount: {input}")
    }
}

pub(crate) fn form_search_from_pattern(pattern: &str) -> (String, bool) {
    match unescape_regex_literal(pattern) {
        Some(search) => (search, false),
        None => (pattern.to_string(), true),
    }
}

pub(crate) fn pattern_from_form(rule: &EditableRule) -> String {
    if rule.is_regex {
        rule.search.trim().to_string()
    } else {
        regex::escape(rule.search.trim())
    }
}

pub(in crate::data) fn unescape_regex_literal(pattern: &str) -> Option<String> {
    let mut out = String::new();
    let mut chars = pattern.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            out.push(chars.next()?);
        } else if is_regex_meta(ch) {
            return None;
        } else {
            out.push(ch);
        }
    }
    Some(out)
}

pub(in crate::data) fn is_regex_meta(ch: char) -> bool {
    matches!(
        ch,
        '^' | '$' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|'
    )
}

pub(in crate::data) fn csv_cell(headers: &[String], row: &csv::StringRecord, name: &str) -> String {
    headers
        .iter()
        .position(|h| h == name)
        .and_then(|idx| row.get(idx))
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub(in crate::data) fn parse_bool_cell(input: &str) -> bool {
    let input = crate::util::normalize_key(input);
    !FALSE_ALIASES
        .lines()
        .any(|alias| crate::util::normalize_key(alias) == input)
}

pub(in crate::data) fn non_empty(value: String, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

pub(in crate::data) fn writer_to_string(wtr: csv::Writer<Vec<u8>>) -> Result<String> {
    let bytes = wtr.into_inner()?;
    String::from_utf8(bytes).context("CSV is not valid UTF-8")
}
