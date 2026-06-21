use super::*;
use crate::model::{
    budget_special_kind_for_config, BudgetDirection, BudgetIncomeBasis, BudgetSpecialKind,
};
use std::collections::{HashMap, HashSet};

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
        validate_budget_special(&budget.special)
            .with_context(|| format!("Budget {} has an invalid special kind", index + 1))?;
        validate_budget_direction(&budget.direction)
            .with_context(|| format!("Budget {} has an invalid direction", index + 1))?;
        validate_budget_income_basis(&budget.income_basis)
            .with_context(|| format!("Budget {} has an invalid income basis", index + 1))?;
    }
    validate_budget_parent_links(budgets)?;
    Ok(())
}

fn validate_budget_parent_links(budgets: &[EditableBudget]) -> Result<()> {
    let by_code = budgets
        .iter()
        .filter(|budget| !budget.code.trim().is_empty())
        .map(|budget| (budget_code_key(&budget.code), budget))
        .collect::<HashMap<_, _>>();

    for budget in budgets
        .iter()
        .filter(|budget| !budget.code.trim().is_empty())
    {
        let parent_code = budget.parent_code.trim();
        if parent_code.is_empty() {
            continue;
        }

        let code_key = budget_code_key(&budget.code);
        let parent_key = budget_code_key(parent_code);
        if code_key == parent_key {
            anyhow::bail!("Budget {} cannot be its own parent", budget.code.trim());
        }

        let Some(parent) = by_code.get(&parent_key).copied() else {
            anyhow::bail!(
                "Budget {} has unknown parent budget code {}",
                budget.code.trim(),
                parent_code
            );
        };
        validate_parent_special_kind(budget, "child")?;
        validate_parent_special_kind(parent, "parent")?;

        if editable_budget_direction(budget) != editable_budget_direction(parent) {
            anyhow::bail!(
                "Budget {} and parent {} must use the same direction",
                budget.code.trim(),
                parent.code.trim()
            );
        }
    }

    for budget in budgets
        .iter()
        .filter(|budget| !budget.code.trim().is_empty())
    {
        validate_budget_parent_chain(budget, &by_code)?;
    }

    Ok(())
}

fn validate_parent_special_kind(budget: &EditableBudget, role: &str) -> Result<()> {
    if editable_budget_special(budget) == BudgetSpecialKind::None {
        Ok(())
    } else {
        anyhow::bail!(
            "Budget {} cannot be used as a {role} budget in a parent link",
            budget.code.trim()
        )
    }
}

fn validate_budget_parent_chain(
    budget: &EditableBudget,
    by_code: &HashMap<String, &EditableBudget>,
) -> Result<()> {
    let root = budget_code_key(&budget.code);
    let mut seen = HashSet::new();
    let mut current = budget;
    while !current.parent_code.trim().is_empty() {
        let parent_key = budget_code_key(&current.parent_code);
        if !seen.insert(parent_key.clone()) || parent_key == root {
            anyhow::bail!(
                "Budget {} has a circular parent budget link",
                budget.code.trim()
            );
        }
        let Some(parent) = by_code.get(&parent_key).copied() else {
            break;
        };
        current = parent;
    }
    Ok(())
}

fn editable_budget_special(budget: &EditableBudget) -> BudgetSpecialKind {
    budget_special_kind_for_config(&budget.special, &budget.code)
}

fn editable_budget_direction(budget: &EditableBudget) -> BudgetDirection {
    match editable_budget_special(budget) {
        BudgetSpecialKind::PlannedIncome | BudgetSpecialKind::Refunded => BudgetDirection::Income,
        BudgetSpecialKind::Transfer => BudgetDirection::Transfer,
        BudgetSpecialKind::Refunding => BudgetDirection::Expense,
        BudgetSpecialKind::None => {
            BudgetDirection::parse(&budget.direction, &budget.code, &budget.category)
        }
    }
}

fn budget_code_key(code: &str) -> String {
    code.trim().to_ascii_lowercase()
}

pub(in crate::data) fn validate_budget_amount(input: &str) -> Result<()> {
    if BudgetAmount::parse(input).is_some() {
        Ok(())
    } else {
        anyhow::bail!("Invalid budget amount: {input}")
    }
}

pub(in crate::data) fn validate_budget_special(input: &str) -> Result<()> {
    if crate::model::budget_special_kind_is_valid_config(input) {
        Ok(())
    } else {
        anyhow::bail!("Invalid budget special kind: {input}")
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
