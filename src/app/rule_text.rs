use super::*;

pub(in crate::app) fn rule_match_summary(
    rule_match: &TransactionRuleMatch,
    show_budget_code: bool,
) -> String {
    let match_text = rule_match_search(rule_match);
    rule_assignment_summary(
        &rule_match.field,
        &match_text,
        &rule_match.category,
        &rule_match.budget_code,
        &rule_match.direction,
        show_budget_code,
    )
}

pub(in crate::app) fn rule_match_search(rule_match: &TransactionRuleMatch) -> String {
    let (search, is_regex) = data::form_search_from_pattern(&rule_match.pattern);
    if is_regex {
        rule_match.pattern.trim().to_string()
    } else {
        search
    }
}

pub(in crate::app) fn rule_assignment_summary(
    field: &str,
    search: &str,
    category: &str,
    budget_code: &str,
    direction: &str,
    show_budget_code: bool,
) -> String {
    let target = if show_budget_code && !budget_code.trim().is_empty() {
        format!("{} / {}", category.trim(), budget_code.trim())
    } else {
        category.trim().to_string()
    };

    trf(
        "Rule {field}: {search} -> {target} ({direction})",
        &[
            ("field", tr(rule_field_label(field))),
            ("search", search.trim().to_string()),
            ("target", target),
            ("direction", tr(rule_direction_label(direction))),
        ],
    )
}

pub(in crate::app) fn rule_field_label(field: &str) -> &'static str {
    match field {
        "counterparty" => "Counterparty",
        "description" => "Description",
        "tags" => "Tags",
        "account" => "Account",
        "transaction_id" => "Transaction ID",
        _ => "Everything",
    }
}

pub(in crate::app) fn rule_direction_label(direction: &str) -> &'static str {
    match direction {
        "expense" => "Expenses",
        "income" => "Income",
        "transfer" => "Transfers",
        _ => "All transactions",
    }
}
