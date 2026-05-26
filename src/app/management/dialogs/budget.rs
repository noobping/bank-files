use super::shared::{new_record_dialog, scroll_to_bottom};
use super::*;

pub(in crate::app) struct NewBudgetDialogRequest<'a> {
    pub(in crate::app) parent: &'a adw::Dialog,
    pub(in crate::app) container: &'a gtk::Box,
    pub(in crate::app) forms: &'a Rc<RefCell<Vec<BudgetForm>>>,
    pub(in crate::app) scrolled_window: &'a gtk::ScrolledWindow,
    pub(in crate::app) status: &'a gtk::Label,
    pub(in crate::app) filter_entry: &'a gtk::SearchEntry,
    pub(in crate::app) advanced_autofill: &'a Rc<Cell<bool>>,
    pub(in crate::app) advanced_features: bool,
}

pub(in crate::app) fn show_new_budget_dialog(request: NewBudgetDialogRequest<'_>) {
    let NewBudgetDialogRequest {
        parent,
        container,
        forms,
        scrolled_window,
        status,
        filter_entry,
        advanced_autofill,
        advanced_features,
    } = request;
    let budget = EditableBudget::new_default();
    let (dialog, page, add_button, dialog_status) = new_record_dialog(
        if advanced_features {
            "New Budget"
        } else {
            "New Category"
        },
        if advanced_features {
            "Create a budget code. It is only saved when you press Save."
        } else {
            "Create a category and choose whether it is spending, income, or transfer. It is only saved when you press Save."
        },
        "Add",
    );

    let grid = form_grid();
    let code = ui::text_combo("", editable_budget_code_values());
    let category = ui::text_combo("", editable_category_values());
    let monthly_budget = entry(&budget.monthly_budget, "500 or 10% of income");
    let yearly_budget = entry(&budget.yearly_budget, "5000 or 10% of yearly income");
    let direction = combo_from_options(
        &[
            ("expense", "Expenses"),
            ("income", "Income"),
            ("transfer", "Transfers"),
        ],
        "expense",
    );
    let income_basis = combo_from_options(
        &[("real", "Real income"), ("planned", "Planned income")],
        ui::budget_income_basis_id(&budget.income_basis),
    );
    if advanced_features {
        connect_budget_fields_autofill(
            &category,
            &code,
            &direction,
            editable_budget_autofill_entries(),
            advanced_autofill,
        );
    }
    let notes = entry("", "Note");
    let income_basis_label = if advanced_features {
        add_labeled(&grid, 0, "Code", &code);
        add_labeled(&grid, 1, "Category", &category);
        add_labeled(&grid, 2, "Monthly budget", &monthly_budget);
        add_labeled(&grid, 3, "Yearly budget", &yearly_budget);
        add_labeled(&grid, 4, "Direction", &direction);
        let income_basis_label = add_labeled(&grid, 5, "Percentage basis", &income_basis);
        add_labeled(&grid, 6, "Note", &notes);
        income_basis_label
    } else {
        code.set_visible(false);
        add_labeled(&grid, 0, "Category", &category);
        add_labeled(&grid, 1, "Monthly budget", &monthly_budget);
        add_labeled(&grid, 2, "Yearly budget", &yearly_budget);
        add_labeled(&grid, 3, "Direction", &direction);
        let income_basis_label = add_labeled(&grid, 4, "Percentage basis", &income_basis);
        add_labeled(&grid, 5, "Note", &notes);
        income_basis_label
    };
    bind_percentage_basis_visibility(
        &monthly_budget,
        &yearly_budget,
        &income_basis_label,
        &income_basis,
    );
    page.append(&grid);
    page.append(&dialog_status);
    dialog.set_focus(Some(if advanced_features { &code } else { &category }));

    let container_for_add = container.clone();
    let forms_for_add = Rc::clone(forms);
    let scrolled_window_for_add = scrolled_window.clone();
    let status_for_add = status.clone();
    let dialog_for_add = dialog.clone();
    let filter_entry_for_add = filter_entry.clone();
    let advanced_autofill_for_add = Rc::clone(advanced_autofill);
    add_button.connect_clicked(move |_| {
        let category_text = ui::combo_text(&category);
        if category_text.is_empty() {
            dialog_status.set_text(&tr("Enter a category first."));
            category.grab_focus();
            return;
        }
        let code_text = if advanced_features {
            let code_text = ui::combo_text(&code);
            if code_text.is_empty() {
                dialog_status.set_text(&tr("Enter a budget code first."));
                code.grab_focus();
                return;
            }
            code_text
        } else {
            let existing_codes = forms_for_add
                .borrow()
                .iter()
                .filter(|form| !form.deleted.get())
                .map(|form| ui::combo_text(&form.code))
                .collect::<Vec<_>>();
            transfer_budget::code_for_new_budget(
                &category_text,
                &combo_active_id(&direction),
                &existing_codes,
            )
        };

        let direction_text = budget_direction_for_save(&combo_active_id(&direction));
        let budget = refund_budget::normalize_editable_budget(
            transfer_budget::normalize_editable_budget(EditableBudget {
                code: code_text,
                special: String::new(),
                category: category_text,
                monthly_budget: monthly_budget.text().trim().to_string(),
                yearly_budget: yearly_budget.text().trim().to_string(),
                direction: direction_text,
                income_basis: combo_active_id(&income_basis),
                notes: notes.text().trim().to_string(),
            }),
        );
        append_budget_form(
            &container_for_add,
            &forms_for_add,
            budget,
            false,
            &advanced_autofill_for_add,
            advanced_features,
        );
        filter_budget_forms(&filter_entry_for_add.text(), &forms_for_add.borrow());
        status_for_add.set_text(&tr(if advanced_features {
            "New budget added. Press Save to keep it."
        } else {
            "New category added. Press Save to keep it."
        }));
        scroll_to_bottom(&scrolled_window_for_add);
        dialog_for_add.close();
    });

    dialog.present(Some(parent));
}

fn budget_direction_for_save(selected_direction: &str) -> String {
    BudgetDirection::parse(selected_direction, "", "")
        .as_str()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_budget_direction_uses_selected_value() {
        assert_eq!(budget_direction_for_save("income"), "income");
        assert_eq!(budget_direction_for_save("transfer"), "transfer");
        assert_eq!(budget_direction_for_save("expense"), "expense");
    }

    #[test]
    fn new_budget_direction_falls_back_to_expense() {
        assert_eq!(budget_direction_for_save("unknown"), "expense");
    }
}
