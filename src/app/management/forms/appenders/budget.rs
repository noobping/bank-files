use super::super::*;
use super::card::{collapsible_form_card, enable_budget_card_reorder};
use super::state::{
    attach_details_grid, connect_budget_delete_button, connect_combo_summary,
    connect_entry_summary, set_option_combo, set_summary,
};
use super::summaries::{budget_summary, BudgetSummaryWidgets};
use super::values::{budget_value, BudgetValueWidgets};

pub(in crate::app) fn append_budget_form(
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<BudgetForm>>>,
    budget: EditableBudget,
    persisted: bool,
    advanced_autofill: &Rc<Cell<bool>>,
    advanced_features: bool,
) {
    let special_config = budget.special.clone();
    let special_kind = crate::model::budget_special_kind_for_config(&special_config, &budget.code);
    let is_special_neutral_budget =
        special_kind.is_neutral() || budget_is_special_neutral(&budget.code);
    let hide_special_controls =
        budget_special_controls_are_hidden(advanced_features, is_special_neutral_budget);
    let original_direction = persisted
        .then(|| BudgetDirection::parse(&budget.direction, &budget.code, &budget.category));
    let card = collapsible_form_card("Budget", "", "Delete budget");
    enable_budget_card_reorder(container, forms, &card, advanced_features);

    let grid = form_grid();
    let code = ui::text_combo(&budget.code, editable_budget_code_values());
    let category = ui::text_combo(&budget.category, editable_category_values());
    let monthly_budget = entry(&budget.monthly_budget, "500 or 10% of income");
    let yearly_budget = entry(&budget.yearly_budget, "5000 or 10% of yearly income");
    let direction = combo_from_options(
        &[
            ("expense", "Expenses"),
            ("income", "Income"),
            ("transfer", "Transfers"),
        ],
        ui::budget_direction_id(&budget.direction),
    );
    let income_basis = combo_from_options(
        &[("real", "Real income"), ("planned", "Planned income")],
        ui::budget_income_basis_id(&budget.income_basis),
    );
    if is_special_neutral_budget {
        direction.set_sensitive(false);
        income_basis.set_sensitive(false);
    } else if advanced_features {
        connect_budget_fields_autofill(
            &category,
            &code,
            &direction,
            editable_budget_autofill_entries(),
            advanced_autofill,
        );
    } else if !budget_direction_editable(advanced_features, persisted) {
        direction.set_sensitive(false);
    }
    let notes = entry(&budget.notes, "Note");
    let income_basis_label = if advanced_features {
        add_labeled(&grid, 0, "Code", &code);
        add_labeled(&grid, 1, "Category", &category);
        add_labeled(&grid, 2, "Monthly budget", &monthly_budget);
        add_labeled(&grid, 3, "Yearly budget", &yearly_budget);
        add_labeled(&grid, 4, "Direction", &direction);
        let income_basis_label = add_labeled(&grid, 5, "Percentage basis", &income_basis);
        add_labeled(&grid, 6, "Note", &notes);
        Some(income_basis_label)
    } else {
        code.set_visible(false);
        add_labeled(&grid, 0, "Category", &category);
        add_labeled(&grid, 1, "Monthly budget", &monthly_budget);
        add_labeled(&grid, 2, "Yearly budget", &yearly_budget);
        if hide_special_controls {
            direction.set_visible(false);
            income_basis.set_visible(false);
            add_labeled(&grid, 3, "Note", &notes);
            None
        } else {
            add_labeled(&grid, 3, "Direction", &direction);
            let income_basis_label = add_labeled(&grid, 4, "Percentage basis", &income_basis);
            add_labeled(&grid, 5, "Note", &notes);
            Some(income_basis_label)
        }
    };
    if let Some(income_basis_label) = &income_basis_label {
        bind_percentage_basis_visibility(
            &monthly_budget,
            &yearly_budget,
            income_basis_label,
            &income_basis,
        );
    }
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
    connect_combo_summary(&direction, &update_summary);
    connect_combo_summary(&income_basis, &update_summary);
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
    connect_combo_summary(&direction, &update_revert_state);
    connect_combo_summary(&income_basis, &update_revert_state);
    connect_entry_summary(&notes, &update_revert_state);
    update_revert_state();

    let update_for_revert = Rc::clone(&update_summary);
    let update_revert_for_revert = Rc::clone(&update_revert_state);
    let code_for_revert = code.clone();
    let category_for_revert = category.clone();
    let monthly_budget_for_revert = monthly_budget.clone();
    let yearly_budget_for_revert = yearly_budget.clone();
    let direction_for_revert = direction.clone();
    let income_basis_for_revert = income_basis.clone();
    let notes_for_revert = notes.clone();
    card.revert_button.connect_clicked(move |_| {
        set_text_combo(&code_for_revert, &original_budget.code);
        set_text_combo(&category_for_revert, &original_budget.category);
        monthly_budget_for_revert.set_text(&original_budget.monthly_budget);
        yearly_budget_for_revert.set_text(&original_budget.yearly_budget);
        set_option_combo(
            &direction_for_revert,
            ui::budget_direction_id(&original_budget.direction),
        );
        set_option_combo(
            &income_basis_for_revert,
            ui::budget_income_basis_id(&original_budget.income_basis),
        );
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
        original_code: Rc::new(RefCell::new(budget.code.trim().to_string())),
        original_direction: Rc::new(RefCell::new(original_direction)),
        auto_code: Rc::new(Cell::new(
            !advanced_features && !persisted && !is_special_neutral_budget,
        )),
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
