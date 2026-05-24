use super::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum TransactionDetailAction {
    CreateRule,
    EditBudgetCode,
    MoveBudgetCode,
    DuplicateAsFake,
    MarkTransfer,
    MarkInvalid,
    Similar,
    FindPattern,
}

pub(super) fn visible_transaction_detail_actions(
    advanced_features: bool,
    smart_patterns_enabled: bool,
    markable_as_transfer: bool,
    budget_move_available: bool,
    auto_detected_classification: bool,
) -> Vec<TransactionDetailAction> {
    [
        TransactionDetailAction::CreateRule,
        TransactionDetailAction::EditBudgetCode,
        TransactionDetailAction::MoveBudgetCode,
        TransactionDetailAction::DuplicateAsFake,
        TransactionDetailAction::MarkTransfer,
        TransactionDetailAction::MarkInvalid,
        TransactionDetailAction::Similar,
        TransactionDetailAction::FindPattern,
    ]
    .into_iter()
    .filter(|action| {
        transaction_detail_action_visible(
            *action,
            advanced_features,
            smart_patterns_enabled,
            markable_as_transfer,
            budget_move_available,
            auto_detected_classification,
        )
    })
    .collect()
}

fn transaction_detail_action_visible(
    action: TransactionDetailAction,
    advanced_features: bool,
    smart_patterns_enabled: bool,
    markable_as_transfer: bool,
    budget_move_available: bool,
    auto_detected_classification: bool,
) -> bool {
    let visible = match action {
        TransactionDetailAction::CreateRule | TransactionDetailAction::EditBudgetCode => {
            advanced_features
        }
        TransactionDetailAction::MoveBudgetCode => budget_move_available,
        TransactionDetailAction::DuplicateAsFake
        | TransactionDetailAction::MarkTransfer
        | TransactionDetailAction::Similar => true,
        TransactionDetailAction::MarkInvalid => auto_detected_classification,
        TransactionDetailAction::FindPattern => smart_patterns_enabled,
    };
    visible && (action != TransactionDetailAction::MarkTransfer || markable_as_transfer)
}

pub(super) fn transaction_detail_config_action_enabled(ui_handles: &UiHandles) -> Option<bool> {
    match config_write_availability(ui_handles) {
        ActionAvailability::Available => {
            Some(ui_handles.loading_count.get() == 0 && !ui_handles.management_dialog_active.get())
        }
        ActionAvailability::Hidden => None,
        ActionAvailability::Disabled(_) => Some(false),
    }
}

pub(super) fn transaction_detail_config_action_blocked(
    ui_handles: &Rc<UiHandles>,
    busy_message: &str,
) -> bool {
    match config_write_availability(ui_handles.as_ref()) {
        ActionAvailability::Available => config_operation_is_active(ui_handles, busy_message),
        ActionAvailability::Hidden => true,
        ActionAvailability::Disabled(reason) => {
            show_status(ui_handles, &reason);
            true
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum TransactionDetailActionPlacement {
    Primary,
    Menu,
}

pub(super) fn transaction_detail_move_budget_code_placement(
    auto_detected_transfer: bool,
) -> TransactionDetailActionPlacement {
    if auto_detected_transfer {
        TransactionDetailActionPlacement::Menu
    } else {
        TransactionDetailActionPlacement::Primary
    }
}

pub(super) fn transaction_detail_move_action_text(
    advanced_features: bool,
) -> (&'static str, &'static str) {
    if advanced_features {
        (
            "Move Budget Code",
            "Move this transaction to another budget code",
        )
    } else {
        ("Move Category", "Move this transaction to another category")
    }
}

pub(super) fn queued_rule_operation_kind(
    rule: EditableRule,
    source: OperationSource,
) -> QueuedOperationKind {
    QueuedOperationKind::Rule {
        rule,
        ensure_budget: true,
        source,
    }
}

pub(super) fn append_transaction_detail_menu_action<F>(
    menu: &gtk::gio::Menu,
    action_group: &gtk::gio::SimpleActionGroup,
    action_name: &str,
    label: &str,
    enabled: bool,
    on_activate: F,
) -> gtk::gio::SimpleAction
where
    F: Fn() + 'static,
{
    let action = gtk::gio::SimpleAction::new(action_name, None);
    action.set_enabled(enabled);
    action.connect_activate(move |_, _| on_activate());
    action_group.add_action(&action);

    let label = tr(label);
    let detailed_action = format!("transaction-detail.{action_name}");
    menu.append(Some(&label), Some(&detailed_action));
    action
}
