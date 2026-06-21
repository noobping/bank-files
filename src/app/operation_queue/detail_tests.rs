use super::*;

#[test]
fn operation_details_icon_tracks_expansion_state() {
    assert_eq!(operation_details_icon_name(false), "pan-end-symbolic");
    assert_eq!(operation_details_icon_name(true), "pan-down-symbolic");
}

#[test]
fn operation_details_keep_full_rule_text() {
    let long_search =
        "a very long shop counterparty with multiple words that should not be truncated";
    let kind = QueuedOperationKind::Rule {
        rule: EditableRule {
            field: "counterparty".to_string(),
            search: long_search.to_string(),
            is_regex: true,
            category: "Groceries and daily shopping".to_string(),
            budget_code: "GROCERY-LONG".to_string(),
            direction: "expense".to_string(),
            amount_min: "-100".to_string(),
            notes: "Generated from repeated matches".to_string(),
            ..EditableRule::new_default()
        },
        ensure_budget: true,
        source: OperationSource::CreateRule,
    };

    let details = operation_details(&kind, &QueuedOperationStatus::Pending);

    assert!(details.contains(long_search));
    assert!(details.contains("Groceries and daily shopping"));
    assert!(details.contains("GROCERY-LONG"));
    assert!(details.contains("-100"));
    assert!(details.contains("Generated from repeated matches"));
}

#[test]
fn operation_details_include_failure_message() {
    let kind = QueuedOperationKind::Rule {
        rule: rule("alpha"),
        ensure_budget: false,
        source: OperationSource::CreateRule,
    };

    let details = operation_details(
        &kind,
        &QueuedOperationStatus::Failed("full failure text".to_string()),
    );

    assert!(details.contains("alpha"));
    assert!(details.contains("full failure text"));
}

#[test]
fn failed_item_can_be_retried() {
    let queue = OperationQueue::new();
    let id = queue
        .enqueue_rule(rule("alpha"), true, OperationSource::CreateRule)
        .id();
    queue.mark_failed(id, "nope".to_string());

    assert_eq!(queue.actionable_ids(), vec![id]);
    assert!(queue.mark_applying(id));
}
