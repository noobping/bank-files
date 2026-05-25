use std::collections::HashSet;

const PLANNED_INCOME_CODE: &str = "INC";

pub fn generated_budget_code_for_category(category: &str, existing_codes: &[String]) -> String {
    let reserved = existing_codes
        .iter()
        .map(|code| budget_code_key(code))
        .collect::<HashSet<_>>();
    generated_budget_code_with_reserved(category, &reserved)
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
