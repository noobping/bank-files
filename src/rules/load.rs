use super::defaults::{localized_default_budgets, localized_default_rules, parse_bool};
use super::Rule;
use crate::model::{
    budget_special_kind_for_config, BudgetAmount, BudgetCode, BudgetDirection, BudgetIncomeBasis,
    BudgetSpecialKind,
};
use crate::util::parse_decimal;

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

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
        let special = budget_special_kind_for_config(&budget_special_cell(&headers, &row), &code);
        let category = non_empty(cell(&headers, &row, "category"), "Uncategorized");
        let direction = budget_direction_for_load(
            &cell(&headers, &row, "direction"),
            &code,
            &category,
            special,
        );
        codes.push(BudgetCode {
            code,
            parent_code: cell(&headers, &row, "parent_code"),
            special,
            category,
            monthly_budget: BudgetAmount::parse_optional(&cell(&headers, &row, "monthly_budget")),
            yearly_budget: BudgetAmount::parse_optional(&cell(&headers, &row, "yearly_budget")),
            direction,
            income_basis: budget_income_basis_for_load(
                &cell(&headers, &row, "income_basis"),
                special,
            ),
            notes: cell(&headers, &row, "notes"),
        });
    }
    Ok(codes)
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

fn non_empty(value: String, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn budget_special_cell(headers: &[String], row: &csv::StringRecord) -> String {
    let special = cell(headers, row, "special");
    if special.trim().is_empty() {
        cell(headers, row, "kind")
    } else {
        special
    }
}

fn budget_direction_for_load(
    input: &str,
    code: &str,
    category: &str,
    special: BudgetSpecialKind,
) -> BudgetDirection {
    match special {
        BudgetSpecialKind::PlannedIncome | BudgetSpecialKind::Refunded => BudgetDirection::Income,
        BudgetSpecialKind::Transfer => BudgetDirection::Transfer,
        BudgetSpecialKind::Refunding => BudgetDirection::Expense,
        BudgetSpecialKind::None => BudgetDirection::parse(input, code, category),
    }
}

fn budget_income_basis_for_load(input: &str, special: BudgetSpecialKind) -> BudgetIncomeBasis {
    if matches!(special, BudgetSpecialKind::None) {
        BudgetIncomeBasis::parse(input)
    } else {
        BudgetIncomeBasis::RealIncome
    }
}
