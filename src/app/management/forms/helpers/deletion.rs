use super::*;

pub(in crate::app) fn set_budget_form_deleted(form: &BudgetForm, is_deleted: bool) {
    set_budget_delete_state(
        &form.form_box,
        &form.delete_button,
        &form.deleted,
        is_deleted,
    );
}

pub(in crate::app) fn set_budget_delete_state(
    form_box: &gtk::Box,
    delete_button: &gtk::Button,
    deleted: &Rc<Cell<bool>>,
    is_deleted: bool,
) {
    deleted.set(is_deleted);
    if is_deleted {
        form_box.add_css_class("warning-card");
        delete_button.remove_css_class("destructive-action");
        delete_button.remove_css_class("suggested-action");
        ui::set_button_icon(delete_button, "edit-undo-symbolic");
        delete_button.set_tooltip_text(Some(&tr("Undo budget deletion")));
    } else {
        form_box.remove_css_class("warning-card");
        delete_button.remove_css_class("suggested-action");
        delete_button.add_css_class("destructive-action");
        ui::set_button_icon(delete_button, "user-trash-symbolic");
        delete_button.set_tooltip_text(Some(&tr("Delete budget")));
    }
}
