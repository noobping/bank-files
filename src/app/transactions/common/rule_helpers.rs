use super::*;

pub(super) fn invalid_auto_detection_rule_for_transaction(tx: &Transaction) -> EditableRule {
    replacement_direction_rule(
        tx,
        150,
        tr("Marked invalid auto detection from transaction detail."),
    )
}

pub(super) fn transfer_undo_rule_for_transaction(tx: &Transaction) -> EditableRule {
    replacement_direction_rule(
        tx,
        160,
        tr("Marked transfer undone from transaction detail."),
    )
}

fn replacement_direction_rule(tx: &Transaction, priority: i32, notes: String) -> EditableRule {
    let (field, search) = transaction_rule_match(tx);
    let (category, budget_code, direction) = non_transfer_rule_values(tx);

    EditableRule {
        priority,
        active: true,
        field,
        search,
        is_regex: false,
        category,
        budget_code,
        direction: direction.to_string(),
        amount_min: String::new(),
        amount_max: String::new(),
        notes,
    }
}

fn non_transfer_rule_values(tx: &Transaction) -> (String, String, &'static str) {
    if tx.amount > Decimal::ZERO {
        (tr("Other income"), "INC-OTHER".to_string(), "income")
    } else {
        (tr("Other"), "OTHER".to_string(), "expense")
    }
}

pub(super) fn editable_rule_for_transaction(
    tx: &Transaction,
    direction_override: Option<&str>,
) -> EditableRule {
    let direction = direction_override.unwrap_or_else(|| transaction_direction_id(tx));
    let (field, search) = transaction_rule_match(tx);
    let category = suggested_category(tx, Some(direction));
    let budget_code = suggested_budget_code(tx, Some(direction));

    EditableRule {
        priority: 140,
        active: true,
        field,
        search,
        is_regex: false,
        category,
        budget_code,
        direction: direction.to_string(),
        amount_min: String::new(),
        amount_max: String::new(),
        notes: tr("Generated from transaction detail."),
    }
}

fn transaction_rule_match(tx: &Transaction) -> (String, String) {
    for (field, value) in [
        ("counterparty", tx.counterparty.trim()),
        ("tags", tx.tags.trim()),
        ("description", tx.description.trim()),
        ("account", tx.account.trim()),
        ("transaction_id", tx.transaction_id.trim()),
    ] {
        if !value.is_empty() {
            return (field.to_string(), value.to_string());
        }
    }
    ("any".to_string(), transaction_search_text(tx))
}

pub(super) fn transaction_direction_id(tx: &Transaction) -> &'static str {
    if tx.budget_code.trim().eq_ignore_ascii_case("TRANSFER") {
        "transfer"
    } else if tx.amount > Decimal::ZERO {
        "income"
    } else {
        "expense"
    }
}

pub(super) fn suggested_category(tx: &Transaction, direction: Option<&str>) -> String {
    match direction.unwrap_or_else(|| transaction_direction_id(tx)) {
        "transfer" => tr("Transfers"),
        "income" => non_empty_transaction_text(&tx.category).unwrap_or_else(|| tr("Other income")),
        _ => non_empty_transaction_text(&tx.category).unwrap_or_else(|| tr("Other")),
    }
}

pub(super) fn suggested_budget_code(tx: &Transaction, direction: Option<&str>) -> String {
    let direction = direction.unwrap_or_else(|| transaction_direction_id(tx));
    let current = tx.budget_code.trim();
    if !current.is_empty()
        && !matches!(current, "OTHER" | "INC-OTHER")
        && !current.eq_ignore_ascii_case("TRANSFER")
    {
        return current.to_string();
    }
    match direction {
        "transfer" => "TRANSFER".to_string(),
        "income" => "INC-OTHER".to_string(),
        _ => suggested_expense_code(tx),
    }
}

fn suggested_expense_code(tx: &Transaction) -> String {
    for value in [
        tx.category.trim(),
        tx.counterparty.trim(),
        tx.description.trim(),
    ] {
        let code = value
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .take(12)
            .collect::<String>()
            .to_ascii_uppercase();
        if code.len() >= 3 && !matches!(code.as_str(), "OTHER" | "UNCATEGORIZ") {
            return code;
        }
    }
    "OTHER".to_string()
}

fn non_empty_transaction_text(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty() && value != "Uncategorized").then(|| value.to_string())
}
