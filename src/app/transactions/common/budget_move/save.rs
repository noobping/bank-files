use super::direction::transaction_budget_direction_change;
use super::form::{set_action_status, transaction_budget_move_form_values_changed};
use super::model::*;
use super::*;

pub(in crate::app::transactions::common) struct TransactionBudgetMoveSave {
    pub(in crate::app::transactions::common) state: Rc<RefCell<AppData>>,
    pub(in crate::app::transactions::common) ui_handles: Rc<UiHandles>,
    pub(in crate::app::transactions::common) dialog: adw::Dialog,
    pub(in crate::app::transactions::common) tx: Transaction,
    pub(in crate::app::transactions::common) initial: EditableRule,
    pub(in crate::app::transactions::common) selected_target:
        Rc<RefCell<Option<TransactionBudgetTarget>>>,
    pub(in crate::app::transactions::common) stack: gtk::Stack,
    pub(in crate::app::transactions::common) list_status: gtk::Label,
    pub(in crate::app::transactions::common) form_status: Option<gtk::Label>,
    pub(in crate::app::transactions::common) form_category: Option<gtk::ComboBoxText>,
    pub(in crate::app::transactions::common) form_budget_code: Option<gtk::ComboBoxText>,
    pub(in crate::app::transactions::common) form_direction: Option<gtk::ComboBoxText>,
    pub(in crate::app::transactions::common) advanced_features: bool,
}

pub(in crate::app::transactions::common) fn connect_transaction_budget_move_save(
    save_button: &gtk::Button,
    controls: TransactionBudgetMoveSave,
) {
    let TransactionBudgetMoveSave {
        state,
        ui_handles,
        dialog,
        tx,
        initial,
        selected_target,
        stack,
        list_status,
        form_status,
        form_category,
        form_budget_code,
        form_direction,
        advanced_features,
    } = controls;

    ui::connect_button_activation(save_button, move |button| {
        let using_form = stack.visible_child_name().as_deref() == Some("form");
        let active_status = if using_form {
            form_status.as_ref().unwrap_or(&list_status).clone()
        } else {
            list_status.clone()
        };

        let move_values = if using_form {
            let Some(category) = form_category.as_ref() else {
                set_action_status(&active_status, "Open More Options first.");
                return;
            };
            let Some(budget_code) = form_budget_code.as_ref() else {
                set_action_status(&active_status, "Open More Options first.");
                return;
            };
            let Some(direction) = form_direction.as_ref() else {
                set_action_status(&active_status, "Open More Options first.");
                return;
            };
            let budget_code_text = ui::combo_text(budget_code);
            if budget_code_text.is_empty() {
                set_action_status(&active_status, "Enter a budget code first.");
                budget_code.grab_focus();
                return;
            }
            let category_text = ui::combo_text(category);
            if category_text.is_empty() {
                set_action_status(&active_status, "Choose a budget code first.");
                budget_code.grab_focus();
                return;
            }
            let direction_text = ui::combo_active_id(direction);
            if !transaction_budget_move_form_values_changed(
                &initial,
                &category_text,
                &budget_code_text,
                &direction_text,
            ) {
                set_action_status(&active_status, "Choose a different category first.");
                return;
            }
            TransactionBudgetMoveValues {
                category: category_text,
                budget_code: budget_code_text,
                direction: direction_text,
            }
        } else {
            let Some(target) = selected_target.borrow().clone() else {
                set_action_status(&active_status, "Choose a category first.");
                return;
            };
            if !transaction_budget_target_is_changed(&tx, &target, advanced_features) {
                set_action_status(&active_status, "Choose a different category first.");
                return;
            }
            if !transaction_budget_target_allowed(
                &tx,
                &state.borrow().budgets,
                &target,
                advanced_features,
            ) {
                set_action_status(
                    &active_status,
                    "This move changes direction. Enable Advanced Features to continue.",
                );
                return;
            }
            TransactionBudgetMoveValues {
                category: target.category,
                budget_code: target.code,
                direction: target.direction.as_str().to_string(),
            }
        };

        let direction_changes = if advanced_features {
            transaction_budget_direction_change(
                &tx,
                &state.borrow().budgets,
                &move_values.budget_code,
                &move_values.category,
                &move_values.direction,
            )
            .into_iter()
            .collect()
        } else {
            Vec::new()
        };
        let rule = transaction_budget_move_rule(&initial, move_values, advanced_features);

        let button = button.clone();
        let status = active_status.clone();
        let ui_for_save = Rc::clone(&ui_handles);
        let dialog_for_confirm = dialog.clone();
        let dialog_for_save = dialog.clone();
        confirm_budget_direction_changes(&dialog_for_confirm, direction_changes, move || {
            if enqueue_rule_operation(&ui_for_save, rule, true, OperationSource::ChangeBudgetCode)
                .queued()
            {
                button.set_sensitive(false);
                set_action_status(&status, budget_move_queued_status(advanced_features));
                dialog_for_save.close();
            } else {
                set_action_status(&status, operation_already_queued_status());
            }
        });
    });
}
