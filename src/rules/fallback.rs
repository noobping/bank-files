use super::defaults::{fallback_categories, AUTO_DETECTED_CATEGORY_NOTE};
use super::text::transaction_tag_text;
use crate::model::{BudgetCode, Transaction};
use crate::util::normalize_key;

use rust_decimal::Decimal;

pub(super) fn fallback_category(
    tx: &Transaction,
    budgets: &[BudgetCode],
    smart_insights_enabled: bool,
) -> CategoryAssignment {
    let text = normalize_key(&format!(
        "{} {} {}",
        tx.description,
        tx.counterparty,
        transaction_tag_text(tx)
    ));
    let amount = tx.amount;

    if smart_insights_enabled {
        for rule in fallback_category_rules() {
            if fallback_direction_matches(rule.direction, amount)
                && any_keywords(&text, rule.keywords)
                && configured_budget_code_exists(budgets, rule.budget_code)
            {
                return CategoryAssignment {
                    category: rule.category.to_string(),
                    budget_code: rule.budget_code.to_string(),
                    notes: Some(crate::i18n::gettext(AUTO_DETECTED_CATEGORY_NOTE)),
                };
            }
        }
    }

    let (category, budget_code) = if amount > Decimal::ZERO {
        other_income_fallback()
    } else {
        other_expense_fallback()
    };
    CategoryAssignment {
        category: category.to_string(),
        budget_code: budget_code.to_string(),
        notes: None,
    }
}

pub(super) struct CategoryAssignment {
    pub(super) category: String,
    pub(super) budget_code: String,
    pub(super) notes: Option<String>,
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
