mod action_registry;
mod apply;
mod controller;
mod details;
mod messages;
mod model;
mod presentation;
mod rows;
mod widgets;

pub(in crate::app) use action_registry::{
    register_operation_queue_menu_action, register_operation_queue_widget,
    update_operation_queue_action_widgets,
};
pub(in crate::app) use controller::{
    connect_operation_queue, enqueue_rule_operation, enqueue_rule_removal_operation,
};
pub(in crate::app) use messages::{budget_move_queued_status, operation_already_queued_status};
pub(in crate::app) use model::{
    OperationQueue, OperationQueueWidgets, OperationSource, QueuedOperationKind,
};
pub(in crate::app) use widgets::{
    build_operation_queue_widgets, refresh_active_operation_queue_ui,
};

#[cfg(test)]
mod tests;
