use super::super::*;
use super::details::{operation_details, operation_status_text};
use super::model::{
    OperationQueue, OperationSource, QueuedOperation, QueuedOperationKind, QueuedOperationStatus,
};

pub(super) const APPLY_ALL_TOOLTIP: &str = "Apply all pending queued operations";
pub(super) const EMPTY_QUEUE_TEXT: &str = "No pending operations.";
pub(super) const EMPTY_QUEUE_SEARCH_TEXT: &str = "No queued operations found.";
pub(super) const OPERATION_QUEUE_SEARCH_PLACEHOLDER: &str = "Search queued operations";
pub(super) const OPERATION_QUEUE_TITLE: &str = "Processing Queue";

pub(super) fn operation_matches_query(operation: &QueuedOperation, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let keywords = operation_keywords(operation);
    query.split_whitespace().all(|term| keywords.contains(term))
}

fn operation_keywords(operation: &QueuedOperation) -> String {
    [
        operation_title(&operation.kind),
        operation_status_text(&operation.status),
        operation_subtitle(&operation.kind),
        operation_details(&operation.kind, &operation.status),
    ]
    .join(" ")
    .to_lowercase()
}

pub(super) fn queue_summary(queue: &OperationQueue) -> String {
    let operations = queue.operations();
    let actionable = operations
        .iter()
        .filter(|operation| operation.status.is_actionable())
        .count();
    let failed = operations
        .iter()
        .filter(|operation| matches!(operation.status, QueuedOperationStatus::Failed(_)))
        .count();
    if operations.is_empty() {
        tr(EMPTY_QUEUE_TEXT)
    } else if failed > 0 {
        trf(
            "{count} pending, {failed} failed.",
            &[
                ("count", actionable.to_string()),
                ("failed", failed.to_string()),
            ],
        )
    } else {
        trf(
            "{count} pending operation(s).",
            &[("count", actionable.to_string())],
        )
    }
}

pub(super) fn operation_title(kind: &QueuedOperationKind) -> String {
    let source = match kind {
        QueuedOperationKind::Rule { source, .. }
        | QueuedOperationKind::RuleRemoval { source, .. } => source,
    };
    tr(match source {
        OperationSource::CreateRule => "Create rule",
        OperationSource::ChangeBudgetCode => "Change budget code",
        OperationSource::MarkTransfer => "Mark transfer",
        OperationSource::UndoTransfer => "Undo transfer mark",
        OperationSource::MarkInvalid => "Mark invalid detection",
    })
}

pub(super) fn operation_queue_actions_are_idle(processing: bool, loading_count: u32) -> bool {
    !processing && loading_count == 0
}

pub(super) fn operation_apply_button_sensitive(
    status: &QueuedOperationStatus,
    processing: bool,
    loading_count: u32,
) -> bool {
    status.is_actionable() && operation_queue_actions_are_idle(processing, loading_count)
}

pub(super) fn operation_subtitle(kind: &QueuedOperationKind) -> String {
    match kind {
        QueuedOperationKind::Rule { rule, .. } => rule_assignment_summary(
            &rule.field,
            &truncate(&rule.search, 48),
            &truncate(&rule.category, 32),
            &truncate(&rule.budget_code, 20),
            &rule.direction,
            true,
        ),
        QueuedOperationKind::RuleRemoval { rule_match, .. } => trf(
            "Remove {rule}",
            &[("rule", rule_match_summary(rule_match, true))],
        ),
    }
}
