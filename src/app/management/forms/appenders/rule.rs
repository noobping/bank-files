use super::super::*;
use super::card::collapsible_form_card;
use super::state::{
    attach_details_grid, connect_combo_summary, connect_delete_button, connect_entry_summary,
    connect_spin_summary, connect_switch_summary, set_option_combo, set_summary,
};
use super::summaries::{rule_summary, RuleSummaryWidgets};
use super::values::{rule_value, RuleValueWidgets};

pub(in crate::app) fn append_rule_form(
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<RuleForm>>>,
    rule: EditableRule,
    persisted: bool,
    advanced_autofill: &Rc<Cell<bool>>,
) {
    let original_direction = persisted
        .then(|| BudgetDirection::from_config(&rule.direction))
        .flatten();
    let card = collapsible_form_card("Rule", "", "Delete rule");
    card.drag_handle.set_tooltip_text(Some(&tr("Move rule")));
    card.drag_handle.set_visible(true);
    connect_rule_form_reorder(container, forms, &card.drag_handle, &card.form_box);

    let grid = form_grid();
    let active = gtk::Switch::builder()
        .active(rule.active)
        .valign(gtk::Align::Center)
        .build();
    let priority = gtk::SpinButton::with_range(0.0, 1000.0, 1.0);
    priority.set_value(rule.priority as f64);
    let field = combo_from_options(
        &[
            ("any", "Everything"),
            ("description", "Description"),
            ("counterparty", "Counterparty"),
            ("tags", "Tags"),
            ("account", "Account"),
            ("transaction_id", "Transaction ID"),
        ],
        &rule.field,
    );
    let search = ui::text_combo(&rule.search, editable_rule_search_values());
    let is_regex = gtk::Switch::builder()
        .active(rule.is_regex)
        .valign(gtk::Align::Center)
        .build();
    let category = ui::text_combo(&rule.category, editable_category_values());
    let budget_code = ui::text_combo(&rule.budget_code, editable_budget_code_values());
    let direction = combo_from_options(
        &[
            ("any", "All transactions"),
            ("expense", "Expenses"),
            ("income", "Income"),
            ("transfer", "Transfers"),
        ],
        if rule.direction.trim().is_empty() {
            "any"
        } else {
            &rule.direction
        },
    );
    connect_budget_fields_autofill(
        &category,
        &budget_code,
        &direction,
        editable_budget_autofill_entries(),
        advanced_autofill,
    );
    let amount_min = entry(&rule.amount_min, "Optional");
    let amount_max = entry(&rule.amount_max, "Optional");
    let notes = entry(&rule.notes, "Note");

    add_labeled(&grid, 0, "Active", &active);
    add_labeled(&grid, 1, "Priority", &priority);
    add_labeled(&grid, 2, "Field", &field);
    add_labeled(&grid, 3, "Search Text", &search);
    add_labeled(&grid, 4, "Regex", &is_regex);
    add_labeled(&grid, 5, "Category", &category);
    add_labeled(&grid, 6, "Budget code", &budget_code);
    add_labeled(&grid, 7, "Direction", &direction);
    add_labeled(&grid, 8, "Min amount", &amount_min);
    add_labeled(&grid, 9, "Max amount", &amount_max);
    add_labeled(&grid, 10, "Note", &notes);
    attach_details_grid(&card, &grid);

    let update_summary: Rc<dyn Fn()> = {
        let row = card.row.clone();
        let active = active.clone();
        let priority = priority.clone();
        let field = field.clone();
        let search = search.clone();
        let is_regex = is_regex.clone();
        let category = category.clone();
        let budget_code = budget_code.clone();
        let direction = direction.clone();
        let amount_min = amount_min.clone();
        let amount_max = amount_max.clone();
        Rc::new(move || {
            set_summary(
                &row,
                rule_summary(RuleSummaryWidgets {
                    active: &active,
                    priority: &priority,
                    field: &field,
                    search: &search,
                    is_regex: &is_regex,
                    category: &category,
                    budget_code: &budget_code,
                    direction: &direction,
                    amount_min: &amount_min,
                    amount_max: &amount_max,
                }),
            );
        })
    };
    connect_switch_summary(&active, &update_summary);
    connect_spin_summary(&priority, &update_summary);
    connect_combo_summary(&field, &update_summary);
    connect_combo_summary(&search, &update_summary);
    connect_switch_summary(&is_regex, &update_summary);
    connect_combo_summary(&category, &update_summary);
    connect_combo_summary(&budget_code, &update_summary);
    connect_combo_summary(&direction, &update_summary);
    connect_entry_summary(&amount_min, &update_summary);
    connect_entry_summary(&amount_max, &update_summary);
    update_summary();

    let original_rule = rule_value(RuleValueWidgets {
        active: &active,
        priority: &priority,
        field: &field,
        search: &search,
        is_regex: &is_regex,
        category: &category,
        budget_code: &budget_code,
        direction: &direction,
        amount_min: &amount_min,
        amount_max: &amount_max,
        notes: &notes,
    });
    let update_revert_state: Rc<dyn Fn()> = {
        let revert_button = card.revert_button.clone();
        let original_rule = original_rule.clone();
        let active = active.clone();
        let priority = priority.clone();
        let field = field.clone();
        let search = search.clone();
        let is_regex = is_regex.clone();
        let category = category.clone();
        let budget_code = budget_code.clone();
        let direction = direction.clone();
        let amount_min = amount_min.clone();
        let amount_max = amount_max.clone();
        let notes = notes.clone();
        Rc::new(move || {
            revert_button.set_sensitive(
                rule_value(RuleValueWidgets {
                    active: &active,
                    priority: &priority,
                    field: &field,
                    search: &search,
                    is_regex: &is_regex,
                    category: &category,
                    budget_code: &budget_code,
                    direction: &direction,
                    amount_min: &amount_min,
                    amount_max: &amount_max,
                    notes: &notes,
                }) != original_rule,
            );
        })
    };
    connect_switch_summary(&active, &update_revert_state);
    connect_spin_summary(&priority, &update_revert_state);
    connect_combo_summary(&field, &update_revert_state);
    connect_combo_summary(&search, &update_revert_state);
    connect_switch_summary(&is_regex, &update_revert_state);
    connect_combo_summary(&category, &update_revert_state);
    connect_combo_summary(&budget_code, &update_revert_state);
    connect_combo_summary(&direction, &update_revert_state);
    connect_entry_summary(&amount_min, &update_revert_state);
    connect_entry_summary(&amount_max, &update_revert_state);
    connect_entry_summary(&notes, &update_revert_state);
    update_revert_state();

    let update_for_revert = Rc::clone(&update_summary);
    let update_revert_for_revert = Rc::clone(&update_revert_state);
    let active_for_revert = active.clone();
    let priority_for_revert = priority.clone();
    let field_for_revert = field.clone();
    let search_for_revert = search.clone();
    let is_regex_for_revert = is_regex.clone();
    let category_for_revert = category.clone();
    let budget_code_for_revert = budget_code.clone();
    let direction_for_revert = direction.clone();
    let amount_min_for_revert = amount_min.clone();
    let amount_max_for_revert = amount_max.clone();
    let notes_for_revert = notes.clone();
    card.revert_button.connect_clicked(move |_| {
        active_for_revert.set_active(original_rule.active);
        priority_for_revert.set_value(original_rule.priority as f64);
        set_option_combo(&field_for_revert, &original_rule.field);
        set_text_combo(&search_for_revert, &original_rule.search);
        is_regex_for_revert.set_active(original_rule.is_regex);
        set_text_combo(&category_for_revert, &original_rule.category);
        set_text_combo(&budget_code_for_revert, &original_rule.budget_code);
        let direction = if original_rule.direction.trim().is_empty() {
            "any"
        } else {
            original_rule.direction.trim()
        };
        set_option_combo(&direction_for_revert, direction);
        amount_min_for_revert.set_text(&original_rule.amount_min);
        amount_max_for_revert.set_text(&original_rule.amount_max);
        notes_for_revert.set_text(&original_rule.notes);
        update_for_revert();
        update_revert_for_revert();
    });

    let deleted = Rc::new(Cell::new(false));
    connect_delete_button(&card.delete_button, &deleted, &card.form_box);

    container.append(&card.form_box);
    forms.borrow_mut().push(RuleForm {
        form_box: card.form_box,
        deleted,
        active,
        priority,
        field,
        search,
        is_regex,
        category,
        budget_code,
        direction,
        amount_min,
        amount_max,
        notes,
        original_direction: Rc::new(RefCell::new(original_direction)),
    });
}
