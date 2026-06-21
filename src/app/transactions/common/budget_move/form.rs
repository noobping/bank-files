use super::model::{transaction_budget_target_is_changed, TransactionBudgetTarget};
use super::*;

pub(in crate::app::transactions::common) fn set_action_status(status: &gtk::Label, message: &str) {
    status.set_text(&tr(message));
    status.set_visible(true);
}

pub(in crate::app::transactions::common) struct TransactionBudgetMoveFormSensitivity<'a> {
    pub(in crate::app::transactions::common) stack: &'a gtk::Stack,
    pub(in crate::app::transactions::common) save_button: &'a gtk::Button,
    pub(in crate::app::transactions::common) tx: &'a Transaction,
    pub(in crate::app::transactions::common) selected_target:
        &'a Rc<RefCell<Option<TransactionBudgetTarget>>>,
    pub(in crate::app::transactions::common) initial: &'a EditableRule,
    pub(in crate::app::transactions::common) category: &'a gtk::ComboBoxText,
    pub(in crate::app::transactions::common) budget_code: &'a gtk::ComboBoxText,
    pub(in crate::app::transactions::common) direction: &'a gtk::ComboBoxText,
    pub(in crate::app::transactions::common) advanced_features: bool,
}

pub(in crate::app::transactions::common) fn connect_transaction_budget_move_form_save_sensitivity(
    controls: TransactionBudgetMoveFormSensitivity<'_>,
) {
    let TransactionBudgetMoveFormSensitivity {
        stack,
        save_button,
        tx,
        selected_target,
        initial,
        category,
        budget_code,
        direction,
        advanced_features,
    } = controls;
    let update: Rc<dyn Fn()> = Rc::new({
        let stack = stack.clone();
        let save_button = save_button.clone();
        let tx = tx.clone();
        let selected_target = Rc::clone(selected_target);
        let initial = initial.clone();
        let category = category.clone();
        let budget_code = budget_code.clone();
        let direction = direction.clone();
        move || {
            let using_form = stack.visible_child_name().as_deref() == Some("form");
            if using_form {
                save_button.set_sensitive(transaction_budget_move_form_is_changed(
                    &initial,
                    &category,
                    &budget_code,
                    &direction,
                ));
            } else {
                save_button.set_sensitive(selected_target.borrow().as_ref().is_some_and(
                    |target| transaction_budget_target_is_changed(&tx, target, advanced_features),
                ));
            }
        }
    });

    for combo in [category, budget_code, direction] {
        let update_for_change = Rc::clone(&update);
        combo.connect_changed(move |_| update_for_change());
    }

    let update_for_page = Rc::clone(&update);
    stack.connect_visible_child_name_notify(move |_| update_for_page());
    update();
}

fn transaction_budget_move_form_is_changed(
    initial: &EditableRule,
    category: &gtk::ComboBoxText,
    budget_code: &gtk::ComboBoxText,
    direction: &gtk::ComboBoxText,
) -> bool {
    transaction_budget_move_form_values_changed(
        initial,
        &ui::combo_text(category),
        &ui::combo_text(budget_code),
        &ui::combo_active_id(direction),
    )
}

pub(in crate::app::transactions::common) fn transaction_budget_move_form_values_changed(
    initial: &EditableRule,
    category: &str,
    budget_code: &str,
    direction: &str,
) -> bool {
    !same_form_text(category, &initial.category)
        || !same_form_text(budget_code, &initial.budget_code)
        || !same_form_text(direction, &initial.direction)
}

fn same_form_text(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}
