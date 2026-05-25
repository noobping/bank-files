use super::super::*;
use super::model::{QueuedOperationKind, QueuedOperationStatus};
use super::presentation::operation_title;

pub(super) fn operation_details_icon_name(expanded: bool) -> &'static str {
    if expanded {
        "pan-down-symbolic"
    } else {
        "pan-end-symbolic"
    }
}

pub(super) fn toggle_operation_details(
    row: &gtk::ListBoxRow,
    details_revealer: &gtk::Revealer,
    expand_icon: &gtk::Image,
) {
    let expanded = !details_revealer.reveals_child();
    details_revealer.set_reveal_child(expanded);
    expand_icon.set_icon_name(Some(operation_details_icon_name(expanded)));
    row.set_tooltip_text(Some(&tr(if expanded {
        "Hide operation details"
    } else {
        "Show operation details"
    })));
}

pub(super) fn point_is_inside_child(
    child: &impl IsA<gtk::Widget>,
    target: &impl IsA<gtk::Widget>,
    x: f64,
    y: f64,
) -> bool {
    child
        .as_ref()
        .compute_bounds(target)
        .map(|bounds| {
            let left = f64::from(bounds.x());
            let top = f64::from(bounds.y());
            let right = left + f64::from(bounds.width());
            let bottom = top + f64::from(bounds.height());
            x >= left && x <= right && y >= top && y <= bottom
        })
        .unwrap_or(false)
}

pub(super) fn operation_details(
    kind: &QueuedOperationKind,
    status: &QueuedOperationStatus,
) -> String {
    let mut lines = Vec::new();
    match kind {
        QueuedOperationKind::Rule {
            rule,
            ensure_budget,
            ..
        } => {
            lines.push(operation_detail_line("Action", operation_title(kind)));
            lines.push(operation_detail_line(
                "Status",
                operation_status_text(status),
            ));
            lines.push(operation_detail_line(
                "Field",
                tr(rule_field_label(&rule.field)),
            ));
            lines.push(operation_detail_line("Match", rule.search.trim()));
            lines.push(operation_detail_line(
                "Regular expression",
                tr(if rule.is_regex { "Yes" } else { "No" }),
            ));
            lines.push(operation_detail_line("Category", rule.category.trim()));
            lines.push(operation_detail_line(
                "Budget code",
                rule.budget_code.trim(),
            ));
            lines.push(operation_detail_line(
                "Direction",
                tr(rule_direction_label(&rule.direction)),
            ));
            if !rule.amount_min.trim().is_empty() {
                lines.push(operation_detail_line(
                    "Minimum amount",
                    rule.amount_min.trim(),
                ));
            }
            if !rule.amount_max.trim().is_empty() {
                lines.push(operation_detail_line(
                    "Maximum amount",
                    rule.amount_max.trim(),
                ));
            }
            if !rule.notes.trim().is_empty() {
                lines.push(operation_detail_line("Notes", rule.notes.trim()));
            }
            lines.push(operation_detail_line(
                "Create missing budget",
                tr(if *ensure_budget { "Yes" } else { "No" }),
            ));
        }
        QueuedOperationKind::RuleRemoval { rule_match, .. } => {
            lines.push(operation_detail_line("Action", operation_title(kind)));
            lines.push(operation_detail_line(
                "Status",
                operation_status_text(status),
            ));
            lines.push(operation_detail_line(
                "Field",
                tr(rule_field_label(&rule_match.field)),
            ));
            lines.push(operation_detail_line(
                "Match",
                rule_match_search(rule_match),
            ));
            lines.push(operation_detail_line(
                "Category",
                rule_match.category.trim(),
            ));
            lines.push(operation_detail_line(
                "Budget code",
                rule_match.budget_code.trim(),
            ));
            lines.push(operation_detail_line(
                "Direction",
                tr(rule_direction_label(&rule_match.direction)),
            ));
            if let Some(amount_min) = rule_match.amount_min {
                lines.push(operation_detail_line(
                    "Minimum amount",
                    amount_min.to_string(),
                ));
            }
            if let Some(amount_max) = rule_match.amount_max {
                lines.push(operation_detail_line(
                    "Maximum amount",
                    amount_max.to_string(),
                ));
            }
            if !rule_match.notes.trim().is_empty() {
                lines.push(operation_detail_line("Notes", rule_match.notes.trim()));
            }
        }
    }

    if let QueuedOperationStatus::Failed(message) = status {
        lines.push(operation_detail_line("Error", message));
    }

    lines.join("\n")
}

fn operation_detail_line(label: &str, value: impl AsRef<str>) -> String {
    trf(
        "{label}: {value}",
        &[("label", tr(label)), ("value", value.as_ref().to_string())],
    )
}

pub(super) fn operation_status_text(status: &QueuedOperationStatus) -> String {
    tr(match status {
        QueuedOperationStatus::Pending => "Pending",
        QueuedOperationStatus::Applying => "Applying",
        QueuedOperationStatus::Applied => "Applied",
        QueuedOperationStatus::Failed(_) => "Failed",
    })
}
