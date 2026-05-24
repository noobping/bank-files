use super::form_options::{budget_code_combo, category_combo, rule_search_combo};
use super::rule_helpers::editable_rule_for_transaction;
use super::*;

pub(super) fn show_transaction_rule_dialog(
    tx: &Transaction,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    direction_override: Option<&str>,
) {
    let initial = editable_rule_for_transaction(tx, direction_override);

    let shell = build_action_dialog_shell(
        "Create Rule",
        "Create a categorization rule from this transaction.",
        "Save",
        "document-save-symbolic",
        "Save rule",
        "Search rule fields",
    );
    shell.set_form_only();
    let save_button = shell.submit_button.clone();
    register_config_widget(ui_handles, &save_button);

    let page = ui::page_box();
    let grid = ui::form_grid();
    let active = gtk::Switch::builder()
        .active(initial.active)
        .valign(gtk::Align::Center)
        .build();
    let priority = gtk::SpinButton::with_range(0.0, 1000.0, 1.0);
    priority.set_value(initial.priority as f64);
    let field = ui::combo_from_options(
        &[
            ("any", "Everything"),
            ("counterparty", "Counterparty"),
            ("description", "Description"),
            ("tags", "Tags"),
            ("account", "Account"),
            ("transaction_id", "Transaction ID"),
        ],
        &initial.field,
    );
    let search = rule_search_combo(tx, &initial.search);
    let is_regex = gtk::Switch::builder()
        .active(initial.is_regex)
        .valign(gtk::Align::Center)
        .build();
    let category = category_combo(&state.borrow(), &initial.category);
    let budget_code = budget_code_combo(&state.borrow(), &initial.budget_code);
    let direction = ui::combo_from_options(
        &[
            ("any", "All transactions"),
            ("expense", "Expenses"),
            ("income", "Income"),
            ("transfer", "Transfers"),
        ],
        &initial.direction,
    );
    connect_budget_fields_autofill(
        &category,
        &budget_code,
        &direction,
        app_budget_autofill_entries(&state.borrow()),
        &ui_handles.advanced_autofill,
    );
    ui::focus_button_after_combo_selections(
        &save_button,
        &[&field, &search, &category, &budget_code, &direction],
    );
    let amount_min = ui::entry(&initial.amount_min, "Optional");
    let amount_max = ui::entry(&initial.amount_max, "Optional");
    let notes = ui::entry(&initial.notes, "Note");

    ui::add_labeled(&grid, 0, "Active", &active);
    ui::add_labeled(&grid, 1, "Priority", &priority);
    ui::add_labeled(&grid, 2, "Field", &field);
    ui::add_labeled(&grid, 3, "Search Text", &search);
    ui::add_labeled(&grid, 4, "Regex", &is_regex);
    ui::add_labeled(&grid, 5, "Category", &category);
    ui::add_labeled(&grid, 6, "Budget code", &budget_code);
    ui::add_labeled(&grid, 7, "Direction", &direction);
    ui::add_labeled(&grid, 8, "Min amount", &amount_min);
    ui::add_labeled(&grid, 9, "Max amount", &amount_max);
    ui::add_labeled(&grid, 10, "Note", &notes);
    page.append(&grid);

    let status = ui::wrapped_label(&tr("Save adds this rule to the processing queue."));
    status.add_css_class("dim-label");
    page.append(&status);
    shell.add_form_page(&ui::action_dialog_scroll(&page));

    let dialog = ui::content_dialog(tr("Create Rule"), &shell.root)
        .content_width(680)
        .default_widget(&save_button)
        .build();

    let ui_for_save = Rc::clone(ui_handles);
    let dialog_for_save = dialog.clone();
    ui::connect_button_activation(&save_button, move |button| {
        let search_text = ui::combo_text(&search);
        let category_text = ui::combo_text(&category);
        if search_text.is_empty() {
            status.set_text(&tr("Enter search text first."));
            search.grab_focus();
            return;
        }
        if category_text.is_empty() {
            status.set_text(&tr("Enter a category first."));
            category.grab_focus();
            return;
        }

        let rule = EditableRule {
            priority: priority.value_as_int(),
            active: active.is_active(),
            field: ui::combo_active_id(&field),
            search: search_text,
            is_regex: is_regex.is_active(),
            category: category_text,
            budget_code: ui::combo_text(&budget_code),
            direction: ui::combo_active_id(&direction),
            amount_min: amount_min.text().trim().to_string(),
            amount_max: amount_max.text().trim().to_string(),
            notes: notes.text().trim().to_string(),
        };

        if enqueue_rule_operation(&ui_for_save, rule, true, OperationSource::CreateRule).queued() {
            button.set_sensitive(false);
            status.set_text(&tr("Rule added to processing queue."));
            dialog_for_save.close();
        } else {
            status.set_text(&tr("Operation is already in the processing queue."));
        }
    });

    dialog.present(Some(&ui_handles.window));
}
