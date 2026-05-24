use super::*;
use std::collections::HashMap;

pub(super) fn show_transaction_pattern_rule_dialog(
    pattern: &analytics::TransactionPattern,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let initial = editable_rule_for_pattern(pattern, &state.borrow());
    show_rule_enqueue_dialog(
        initial,
        RuleDialogSpec {
            subtitle: "Create a categorization rule from this detected transaction pattern.",
            content_width: 650,
            field_options: PATTERN_RULE_FIELD_OPTIONS,
            search_values: pattern_rule_search_values(pattern),
        },
        state,
        ui_handles,
    );
}

fn editable_rule_for_pattern(
    pattern: &analytics::TransactionPattern,
    data: &AppData,
) -> EditableRule {
    let matches = data
        .transactions
        .iter()
        .filter(|transaction| analytics::transaction_matches_pattern(transaction, pattern))
        .collect::<Vec<_>>();
    let is_transfer = matches!(pattern.kind, analytics::TransactionPatternKind::Transfer);
    let category = if is_transfer {
        tr("Transfers")
    } else {
        most_common_text(
            matches
                .iter()
                .map(|transaction| transaction.category.trim())
                .filter(|category| !category.is_empty() && *category != "Uncategorized"),
        )
        .unwrap_or_else(|| pattern.label.clone())
    };
    let budget_code = if is_transfer {
        "TRANSFER".to_string()
    } else {
        most_common_text(
            matches
                .iter()
                .map(|transaction| transaction.budget_code.trim())
                .filter(|code| !code.is_empty()),
        )
        .unwrap_or_else(|| {
            if pattern.amount > Decimal::ZERO {
                "INC-OTHER".to_string()
            } else {
                "OTHER".to_string()
            }
        })
    };
    let direction = if is_transfer {
        "transfer"
    } else if pattern.amount > Decimal::ZERO {
        "income"
    } else if pattern.amount < Decimal::ZERO {
        "expense"
    } else {
        "any"
    };

    EditableRule {
        priority: 130,
        active: true,
        field: "tags".to_string(),
        search: pattern.label.clone(),
        is_regex: false,
        category,
        budget_code,
        direction: direction.to_string(),
        amount_min: String::new(),
        amount_max: String::new(),
        notes: tr("Generated from detected transaction pattern."),
    }
}

fn most_common_text<'a>(items: impl Iterator<Item = &'a str>) -> Option<String> {
    let mut counts = HashMap::<String, usize>::new();
    for item in items {
        *counts.entry(item.to_string()).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
        .map(|(item, _)| item)
}
