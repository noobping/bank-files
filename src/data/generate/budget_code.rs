use super::PLANNED_INCOME_CODE;
use std::collections::HashSet;

pub fn generated_budget_code_for_category(category: &str, existing_codes: &[String]) -> String {
    let reserved = existing_codes
        .iter()
        .map(|code| budget_code_key(code))
        .collect::<HashSet<_>>();
    generated_budget_code_with_reserved(category, &reserved)
}

pub(super) fn human_category(label: &str) -> String {
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

pub(super) fn budget_code_key(code: &str) -> String {
    code.trim().to_ascii_lowercase()
}

pub(super) fn generated_budget_code_with_reserved(
    category: &str,
    reserved: &HashSet<String>,
) -> String {
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

pub(super) fn clean_label(label: &str) -> String {
    label.split_whitespace().collect::<Vec<_>>().join(" ")
}
