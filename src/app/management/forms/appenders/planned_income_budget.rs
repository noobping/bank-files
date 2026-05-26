use super::super::*;
use super::card::{collapsible_form_card, enable_budget_card_reorder};
use super::state::{
    attach_details_grid, connect_budget_delete_button, connect_combo_summary,
    connect_entry_summary, set_summary,
};
use super::summaries::{budget_summary, BudgetSummaryWidgets};
use super::values::{budget_value, BudgetValueWidgets};

pub(in crate::app) fn append_planned_income_budget_form(
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<BudgetForm>>>,
    budget: EditableBudget,
    advanced_features: bool,
) {
    let special_config = budget.special.clone();
    let original_code = budget.code.trim().to_string();
    let card = collapsible_form_card("Planned Income", "", "Delete planned income budget");
    enable_budget_card_reorder(container, forms, &card, advanced_features);

    let grid = form_grid();
    let code = ui::text_combo(&budget.code, editable_budget_code_values());
    code.set_sensitive(advanced_features);
    let category = ui::text_combo(&budget.category, editable_category_values());
    let monthly_budget = entry(
        &planned_income::fixed_budget_amount_text(&budget.monthly_budget),
        "500",
    );
    let yearly_budget = entry(
        &planned_income::fixed_budget_amount_text(&budget.yearly_budget),
        "5000",
    );
    let direction = combo_from_options(&[("income", "Income")], "income");
    let income_basis = combo_from_options(&[("real", "Real income")], "real");
    let notes = entry(&budget.notes, "Note");
    planned_income::connect_fixed_budget_entry(&monthly_budget);
    planned_income::connect_fixed_budget_entry(&yearly_budget);

    let reminder_row = if advanced_features {
        add_labeled(&grid, 0, "Code", &code);
        add_labeled(&grid, 1, "Category", &category);
        add_labeled(&grid, 2, "Monthly planned income", &monthly_budget);
        add_labeled(&grid, 3, "Yearly planned income", &yearly_budget);
        add_labeled(&grid, 4, "Note", &notes);
        5
    } else {
        code.set_visible(false);
        add_labeled(&grid, 0, "Category", &category);
        add_labeled(&grid, 1, "Monthly planned income", &monthly_budget);
        add_labeled(&grid, 2, "Yearly planned income", &yearly_budget);
        add_labeled(&grid, 3, "Note", &notes);
        4
    };
    let reminder = ui::wrapped_label(&tr(planned_income::NET_INCOME_REMINDER));
    reminder.add_css_class("dim-label");
    grid.attach(&reminder, 0, reminder_row, 2, 1);
    attach_details_grid(&card, &grid);

    let update_summary: Rc<dyn Fn()> = {
        let row = card.row.clone();
        let code = code.clone();
        let category = category.clone();
        let monthly_budget = monthly_budget.clone();
        let yearly_budget = yearly_budget.clone();
        let direction = direction.clone();
        let income_basis = income_basis.clone();
        let special_for_summary = special_config.clone();
        Rc::new(move || {
            set_summary(
                &row,
                budget_summary(BudgetSummaryWidgets {
                    code: &code,
                    category: &category,
                    monthly_budget: &monthly_budget,
                    yearly_budget: &yearly_budget,
                    direction: &direction,
                    income_basis: &income_basis,
                    special: &special_for_summary,
                    show_code: advanced_features,
                }),
            );
        })
    };
    connect_combo_summary(&code, &update_summary);
    connect_combo_summary(&category, &update_summary);
    connect_entry_summary(&monthly_budget, &update_summary);
    connect_entry_summary(&yearly_budget, &update_summary);
    update_summary();

    let original_budget = budget_value(BudgetValueWidgets {
        code: &code,
        special: &special_config,
        category: &category,
        monthly_budget: &monthly_budget,
        yearly_budget: &yearly_budget,
        direction: &direction,
        income_basis: &income_basis,
        notes: &notes,
    });
    let update_revert_state: Rc<dyn Fn()> = {
        let revert_button = card.revert_button.clone();
        let original_budget = original_budget.clone();
        let code = code.clone();
        let category = category.clone();
        let monthly_budget = monthly_budget.clone();
        let yearly_budget = yearly_budget.clone();
        let direction = direction.clone();
        let income_basis = income_basis.clone();
        let notes = notes.clone();
        Rc::new(move || {
            revert_button.set_sensitive(
                budget_value(BudgetValueWidgets {
                    code: &code,
                    special: &original_budget.special,
                    category: &category,
                    monthly_budget: &monthly_budget,
                    yearly_budget: &yearly_budget,
                    direction: &direction,
                    income_basis: &income_basis,
                    notes: &notes,
                }) != original_budget,
            );
        })
    };
    connect_combo_summary(&code, &update_revert_state);
    connect_combo_summary(&category, &update_revert_state);
    connect_entry_summary(&monthly_budget, &update_revert_state);
    connect_entry_summary(&yearly_budget, &update_revert_state);
    connect_entry_summary(&notes, &update_revert_state);
    update_revert_state();

    let update_for_revert = Rc::clone(&update_summary);
    let update_revert_for_revert = Rc::clone(&update_revert_state);
    let code_for_revert = code.clone();
    let category_for_revert = category.clone();
    let monthly_budget_for_revert = monthly_budget.clone();
    let yearly_budget_for_revert = yearly_budget.clone();
    let notes_for_revert = notes.clone();
    card.revert_button.connect_clicked(move |_| {
        set_text_combo(&code_for_revert, &original_budget.code);
        set_text_combo(&category_for_revert, &original_budget.category);
        monthly_budget_for_revert.set_text(&original_budget.monthly_budget);
        yearly_budget_for_revert.set_text(&original_budget.yearly_budget);
        notes_for_revert.set_text(&original_budget.notes);
        update_for_revert();
        update_revert_for_revert();
    });

    let deleted = Rc::new(Cell::new(false));
    connect_budget_delete_button(&card.delete_button, &deleted, &card.form_box);
    container.append(&card.form_box);
    forms.borrow_mut().push(BudgetForm {
        form_box: card.form_box,
        deleted,
        original_code: Rc::new(RefCell::new(original_code)),
        original_direction: Rc::new(RefCell::new(Some(BudgetDirection::Income))),
        auto_code: Rc::new(Cell::new(false)),
        code,
        special: special_config,
        category,
        monthly_budget,
        yearly_budget,
        direction,
        income_basis,
        notes,
        delete_button: card.delete_button,
    });
}
