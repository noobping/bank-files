use super::super::{EditableRule, TransactionRuleMatch};
use super::action_registry::operation_queue_action_enabled;
use super::apply::{apply_summary_message, group_and_combine_queued_rules};
use super::details::{operation_details, operation_details_icon_name};
use super::model::{
    ApplyCounts, EnqueueOperationResult, OperationQueue, OperationSource, QueuedOperation,
    QueuedOperationKind, QueuedOperationStatus, RuleCombineSummary,
};
use super::presentation::{operation_apply_button_sensitive, operation_matches_query};
use super::widgets::operation_queue_button_is_suggested;

#[path = "detail_tests.rs"]
mod detail_tests;

fn rule_match(search: &str) -> TransactionRuleMatch {
    TransactionRuleMatch {
        priority: 120,
        field: "counterparty".to_string(),
        pattern: regex::escape(search),
        matched_text: search.to_string(),
        category: "Transfers".to_string(),
        budget_code: "TRANSFER".to_string(),
        direction: "transfer".to_string(),
        amount_min: None,
        amount_max: None,
        notes: String::new(),
    }
}

fn rule(search: &str) -> EditableRule {
    EditableRule {
        search: search.to_string(),
        ..EditableRule::new_default()
    }
}

#[test]
fn enqueue_assigns_stable_ids_and_counts_actionable_items() {
    let queue = OperationQueue::new();

    let first = queue.enqueue_rule(rule("alpha"), true, OperationSource::CreateRule);
    let second = queue.enqueue_rule(rule("beta"), true, OperationSource::ChangeBudgetCode);

    assert_eq!(first, EnqueueOperationResult::Queued(1));
    assert_eq!(second, EnqueueOperationResult::Queued(2));
    assert_eq!(queue.actionable_count(), 2);
}

#[test]
fn duplicate_rule_undo_operations_are_not_enqueued_twice() {
    let queue = OperationQueue::new();
    let first = queue.enqueue_rule_undo(rule_match("Savings"), OperationSource::UndoTransfer);
    let duplicate = queue.enqueue_rule_undo(rule_match("Savings"), OperationSource::UndoTransfer);

    assert_eq!(first, EnqueueOperationResult::Queued(1));
    assert_eq!(duplicate, EnqueueOperationResult::AlreadyQueued(1));
    assert_eq!(queue.operations().len(), 1);
}

#[test]
fn duplicate_rule_operations_are_not_enqueued_twice() {
    let queue = OperationQueue::new();
    let first = queue.enqueue_rule(rule("alpha"), true, OperationSource::CreateRule);
    let duplicate = queue.enqueue_rule(rule("alpha"), true, OperationSource::CreateRule);
    let other_source = queue.enqueue_rule(rule("alpha"), true, OperationSource::ChangeBudgetCode);

    assert_eq!(first, EnqueueOperationResult::Queued(1));
    assert_eq!(duplicate, EnqueueOperationResult::AlreadyQueued(1));
    assert_eq!(other_source, EnqueueOperationResult::Queued(2));
    assert_eq!(queue.operations().len(), 2);
}

#[test]
fn queued_action_sensitivity_follows_duplicate_state() {
    assert!(operation_queue_action_enabled(true, false, true));
    assert!(!operation_queue_action_enabled(true, true, true));
    assert!(!operation_queue_action_enabled(false, false, true));
    assert!(!operation_queue_action_enabled(true, false, false));
}

#[test]
fn pending_remove_deletes_item() {
    let queue = OperationQueue::new();
    let id = queue
        .enqueue_rule(rule("alpha"), true, OperationSource::CreateRule)
        .id();

    assert!(queue.remove(id));
    assert!(queue.operations().is_empty());
}

#[test]
fn applying_item_cannot_be_removed() {
    let queue = OperationQueue::new();
    let id = queue
        .enqueue_rule(rule("alpha"), true, OperationSource::CreateRule)
        .id();

    assert!(queue.mark_applying(id));
    assert!(!queue.remove(id));
    assert_eq!(queue.operations().len(), 1);
}

#[test]
fn apply_buttons_are_disabled_while_loading_or_processing() {
    assert!(operation_apply_button_sensitive(
        &QueuedOperationStatus::Pending,
        false,
        0,
    ));
    assert!(!operation_apply_button_sensitive(
        &QueuedOperationStatus::Pending,
        true,
        0,
    ));
    assert!(!operation_apply_button_sensitive(
        &QueuedOperationStatus::Pending,
        false,
        1,
    ));
    assert!(!operation_apply_button_sensitive(
        &QueuedOperationStatus::Applied,
        false,
        0,
    ));
}

#[test]
fn applied_and_failed_items_can_be_removed() {
    let queue = OperationQueue::new();
    let applied = queue
        .enqueue_rule(rule("alpha"), true, OperationSource::CreateRule)
        .id();
    let failed = queue
        .enqueue_rule(rule("beta"), true, OperationSource::CreateRule)
        .id();

    queue.mark_applied(applied);
    queue.mark_failed(failed, "nope".to_string());

    assert!(queue.remove(applied));
    assert!(queue.remove(failed));
    assert!(queue.operations().is_empty());
}

#[test]
fn apply_summary_includes_rule_combine_result() {
    let message = apply_summary_message(
        ApplyCounts {
            applied: 2,
            failed: 0,
        },
        Some(RuleCombineSummary {
            before_count: 8,
            after_count: 5,
        }),
    );

    assert!(message.contains('2'));
    assert!(message.contains('8'));
    assert!(message.contains('5'));
}

#[test]
fn queued_rule_combine_groups_compatible_rules_first() {
    let software = EditableRule {
        search: "hosting".to_string(),
        category: "Software".to_string(),
        budget_code: "SOFT".to_string(),
        ..EditableRule::new_default()
    };

    let (rules, summary) = group_and_combine_queued_rules(&[rule("alpha"), software, rule("beta")]);

    assert_eq!(
        summary,
        Some(RuleCombineSummary {
            before_count: 3,
            after_count: 2,
        })
    );
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].search, "(?:alpha|beta)");
    assert!(rules[0].is_regex);
    assert_eq!(rules[1].search, "hosting");
}

#[test]
fn operation_search_matches_rule_details() {
    let operation = QueuedOperation {
        id: 1,
        kind: QueuedOperationKind::Rule {
            rule: EditableRule {
                field: "counterparty".to_string(),
                search: "Coffee Shop".to_string(),
                category: "Food".to_string(),
                budget_code: "FOOD".to_string(),
                ..EditableRule::new_default()
            },
            ensure_budget: true,
            source: OperationSource::CreateRule,
        },
        status: QueuedOperationStatus::Pending,
    };

    assert!(operation_matches_query(&operation, "coffee food"));
    assert!(operation_matches_query(&operation, "coffee shop"));
    assert!(!operation_matches_query(&operation, "transport"));
}

#[test]
fn queue_button_is_suggested_only_for_pending_operations() {
    assert!(!operation_queue_button_is_suggested(0));
    assert!(operation_queue_button_is_suggested(1));
}

#[test]
fn clear_applied_removes_only_successful_items() {
    let queue = OperationQueue::new();
    let applied = queue
        .enqueue_rule(rule("alpha"), true, OperationSource::CreateRule)
        .id();
    let failed = queue
        .enqueue_rule(rule("beta"), true, OperationSource::CreateRule)
        .id();
    let pending = queue
        .enqueue_rule(rule("gamma"), true, OperationSource::CreateRule)
        .id();

    queue.mark_applied(applied);
    queue.mark_failed(failed, "nope".to_string());

    assert_eq!(queue.applied_count(), 1);
    assert_eq!(queue.clear_applied(), 1);
    assert_eq!(queue.applied_count(), 0);
    assert_eq!(
        queue
            .operations()
            .iter()
            .map(|operation| operation.id)
            .collect::<Vec<_>>(),
        vec![failed, pending]
    );
}
