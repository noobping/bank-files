pub(in crate::app) fn operation_added_status() -> &'static str {
    "Operation added to queue."
}

pub(in crate::app) fn operation_already_queued_status() -> &'static str {
    "Operation is already in the processing queue."
}

pub(super) fn operation_already_queued_tooltip() -> &'static str {
    "This operation is already in the processing queue."
}

pub(in crate::app) fn budget_move_queued_status(advanced_features: bool) -> &'static str {
    if advanced_features {
        "Budget code move added to processing queue."
    } else {
        "Category move added to processing queue."
    }
}
