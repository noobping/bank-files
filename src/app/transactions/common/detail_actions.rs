use super::budget_move::{
    show_transaction_budget_code_dialog, transaction_budget_move_available,
    transaction_is_markable_as_transfer,
};
use super::detail_mark::{append_mark_refund_action, append_mark_transfer_action};
use super::detail_primary::{
    append_primary_move_budget_action, append_primary_refund_undo_action,
    append_primary_transfer_undo_action, append_similar_action,
};
use super::rule_dialog::show_transaction_rule_dialog;
use super::rule_helpers::{
    invalid_auto_detection_rule_for_transaction, suggested_budget_code, suggested_category,
};
use super::rule_ops::apply_invalid_auto_detection_rule;
use super::*;

use super::detail_state::{
    append_transaction_detail_menu_action, queued_rule_operation_kind,
    transaction_detail_config_action_blocked, transaction_detail_config_action_enabled,
    transaction_detail_move_action_text, transaction_detail_move_budget_code_placement,
    visible_transaction_detail_actions, TransactionDetailAction, TransactionDetailActionPlacement,
};
pub(super) fn transaction_detail_actions(
    tx: &Transaction,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> gtk::Box {
    let actions = gtk::Box::new(gtk::Orientation::Vertical, 6);
    actions.set_hexpand(true);

    let primary_actions = ui::linked_button_group();
    primary_actions.set_halign(gtk::Align::Start);
    let menu = gtk::gio::Menu::new();
    let menu_actions = gtk::gio::SimpleActionGroup::new();
    let mut has_menu_items = false;

    let advanced_features = ui_handles.advanced_features.get();
    let auto_detected_classification =
        crate::rules::transaction_classification_is_auto_detected(tx);
    let (markable_as_transfer, markable_as_refund, budget_move_available, budgets) = {
        let data = state.borrow();
        (
            transaction_is_markable_as_transfer(tx, &data.budgets),
            !analytics::transaction_is_refund(tx, &data.budgets),
            transaction_budget_move_available(tx, &data.budgets, advanced_features),
            data.budgets.clone(),
        )
    };
    let special_marked = !markable_as_transfer || !markable_as_refund;
    let visible_actions = visible_transaction_detail_actions(
        advanced_features,
        markable_as_transfer,
        markable_as_refund,
        budget_move_available,
        auto_detected_classification,
        tx.rule_match.is_some(),
    );
    let config_menu_action_enabled = transaction_detail_config_action_enabled(ui_handles.as_ref());
    let move_budget_code_placement = transaction_detail_move_budget_code_placement(special_marked);

    if visible_actions.contains(&TransactionDetailAction::MoveBudgetCode)
        && move_budget_code_placement == TransactionDetailActionPlacement::Primary
    {
        append_primary_move_budget_action(
            tx,
            state,
            ui_handles,
            &primary_actions,
            advanced_features,
        );
    }

    if visible_actions.contains(&TransactionDetailAction::MoveBudgetCode)
        && move_budget_code_placement == TransactionDetailActionPlacement::Menu
    {
        if let Some(enabled) = config_menu_action_enabled {
            let tx_for_change = tx.clone();
            let state_for_change = Rc::clone(state);
            let ui_for_change = Rc::clone(ui_handles);
            let (move_label, _) = transaction_detail_move_action_text(advanced_features);
            append_transaction_detail_menu_action(
                &menu,
                &menu_actions,
                "move-budget-code",
                move_label,
                enabled,
                move || {
                    if transaction_detail_config_action_blocked(
                        &ui_for_change,
                        "Another edit or save is already running.",
                    ) {
                        return;
                    }
                    show_transaction_budget_code_dialog(
                        &tx_for_change,
                        &state_for_change,
                        &ui_for_change,
                    );
                },
            );
            has_menu_items = true;
        }
    }

    if visible_actions.contains(&TransactionDetailAction::UndoTransfer) {
        if let Some(enabled) = config_menu_action_enabled {
            append_primary_transfer_undo_action(tx, ui_handles, &primary_actions, enabled);
        }
    }

    if visible_actions.contains(&TransactionDetailAction::UndoRefund) {
        if let Some(enabled) = config_menu_action_enabled {
            append_primary_refund_undo_action(tx, ui_handles, &primary_actions, enabled);
        }
    }

    if visible_actions.contains(&TransactionDetailAction::Similar) {
        append_similar_action(tx, state, ui_handles, &primary_actions);
    }

    if visible_actions.contains(&TransactionDetailAction::MarkInvalid) {
        if let Some(enabled) = config_menu_action_enabled {
            let tx_for_invalid = tx.clone();
            let ui_for_invalid = Rc::clone(ui_handles);
            let invalid_operation = queued_rule_operation_kind(
                invalid_auto_detection_rule_for_transaction(tx),
                OperationSource::MarkInvalid,
            );
            let action = append_transaction_detail_menu_action(
                &menu,
                &menu_actions,
                "mark-invalid",
                "Mark auto detection invalid",
                enabled,
                move || {
                    if transaction_detail_config_action_blocked(
                        &ui_for_invalid,
                        "Another edit or save is already running.",
                    ) {
                        return;
                    }
                    apply_invalid_auto_detection_rule(&tx_for_invalid, &ui_for_invalid);
                },
            );
            register_operation_queue_menu_action(
                ui_handles,
                &primary_actions,
                &action,
                invalid_operation,
            );
            has_menu_items = true;
        }
    }

    if visible_actions.contains(&TransactionDetailAction::MarkTransfer) {
        if let Some(enabled) = config_menu_action_enabled {
            append_mark_transfer_action(
                tx,
                &budgets,
                ui_handles,
                &primary_actions,
                &menu,
                &menu_actions,
                enabled,
            );
            has_menu_items = true;
        }
    }

    if visible_actions.contains(&TransactionDetailAction::MarkRefund) {
        if let Some(enabled) = config_menu_action_enabled {
            append_mark_refund_action(
                tx,
                &budgets,
                ui_handles,
                &primary_actions,
                &menu,
                &menu_actions,
                enabled,
            );
            has_menu_items = true;
        }
    }

    if visible_actions.contains(&TransactionDetailAction::DuplicateAsFake) {
        let tx_for_fake = tx.clone();
        let state_for_fake = Rc::clone(state);
        let ui_for_fake = Rc::clone(ui_handles);
        append_transaction_detail_menu_action(
            &menu,
            &menu_actions,
            "duplicate-as-fake",
            "Duplicate as Fake",
            true,
            move || {
                duplicate_transaction_as_fake(&state_for_fake, &ui_for_fake, &tx_for_fake);
            },
        );
        has_menu_items = true;
    }

    if visible_actions.contains(&TransactionDetailAction::CreateRule) {
        if let Some(enabled) = config_menu_action_enabled {
            let tx_for_rule = tx.clone();
            let state_for_rule = Rc::clone(state);
            let ui_for_rule = Rc::clone(ui_handles);
            append_transaction_detail_menu_action(
                &menu,
                &menu_actions,
                "create-rule",
                "Create rule",
                enabled,
                move || {
                    if transaction_detail_config_action_blocked(
                        &ui_for_rule,
                        "Another edit or save is already running.",
                    ) {
                        return;
                    }
                    show_transaction_rule_dialog(&tx_for_rule, &state_for_rule, &ui_for_rule, None);
                },
            );
            has_menu_items = true;
        }
    }

    if visible_actions.contains(&TransactionDetailAction::EditBudgetCode) {
        if let Some(enabled) = config_menu_action_enabled {
            let tx_for_budget = tx.clone();
            let state_for_budget = Rc::clone(state);
            let ui_for_budget = Rc::clone(ui_handles);
            append_transaction_detail_menu_action(
                &menu,
                &menu_actions,
                "edit-budget-code",
                "Edit Budget",
                enabled,
                move || {
                    if transaction_detail_config_action_blocked(
                        &ui_for_budget,
                        "Another edit or save is already running.",
                    ) {
                        return;
                    }
                    let code = {
                        let data = state_for_budget.borrow();
                        suggested_budget_code(&tx_for_budget, None, &data.budgets)
                    };
                    let category = suggested_category(&tx_for_budget, None);
                    show_budget_edit_dialog(&code, &category, &state_for_budget, &ui_for_budget);
                },
            );
            has_menu_items = true;
        }
    }

    if has_menu_items {
        let more_menu_button = gtk::MenuButton::builder()
            .icon_name("view-more-symbolic")
            .tooltip_text(tr("More"))
            .build();
        more_menu_button.set_focus_on_click(false);
        more_menu_button.insert_action_group("transaction-detail", Some(&menu_actions));
        more_menu_button.set_menu_model(Some(&menu));
        primary_actions.append(&more_menu_button);
    }

    if primary_actions.first_child().is_some() {
        actions.append(&primary_actions);
    }
    actions
}
