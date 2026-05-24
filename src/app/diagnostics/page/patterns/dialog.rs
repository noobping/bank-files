use super::*;
use std::collections::HashMap;

pub(super) fn show_transaction_pattern_rule_dialog(
    pattern: &analytics::TransactionPattern,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let initial = editable_rule_for_pattern(pattern, &state.borrow());

    let shell = build_action_dialog_shell(
        "Create Rule",
        "Create a categorization rule from this detected transaction pattern.",
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
            ("tags", "Tags"),
            ("description", "Description"),
            ("counterparty", "Counterparty"),
            ("account", "Account"),
            ("transaction_id", "Transaction ID"),
        ],
        &initial.field,
    );
    let search = ui::text_combo(&initial.search, pattern_rule_search_values(pattern));
    let is_regex = gtk::Switch::builder()
        .active(initial.is_regex)
        .valign(gtk::Align::Center)
        .build();
    let category = ui::text_combo(&initial.category, app_category_values(&state.borrow()));
    let budget_code = ui::text_combo(
        &initial.budget_code,
        app_budget_code_values(&state.borrow()),
    );
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
        .content_width(650)
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

fn editable_rule_for_pattern(
    pattern: &analytics::TransactionPattern,
    data: &AppData,
) -> EditableRule {
    let matches = data
        .transactions
        .iter()
        .filter(|transaction| analytics::transaction_matches_pattern(transaction, pattern))
        .collect::<Vec<_>>();
    let is_transfer = matches!(pattern.kind, analytics::TransactionPatternKind::Transfer);
    let category = if is_transfer {
        tr("Transfers")
    } else {
        most_common_text(
            matches
                .iter()
                .map(|transaction| transaction.category.trim())
                .filter(|category| !category.is_empty() && *category != "Uncategorized"),
        )
        .unwrap_or_else(|| pattern.label.clone())
    };
    let budget_code = if is_transfer {
        "TRANSFER".to_string()
    } else {
        most_common_text(
            matches
                .iter()
                .map(|transaction| transaction.budget_code.trim())
                .filter(|code| !code.is_empty()),
        )
        .unwrap_or_else(|| {
            if pattern.amount > Decimal::ZERO {
                "INC-OTHER".to_string()
            } else {
                "OTHER".to_string()
            }
        })
    };
    let direction = if is_transfer {
        "transfer"
    } else if pattern.amount > Decimal::ZERO {
        "income"
    } else if pattern.amount < Decimal::ZERO {
        "expense"
    } else {
        "any"
    };

    EditableRule {
        priority: 130,
        active: true,
        field: "tags".to_string(),
        search: pattern.label.clone(),
        is_regex: false,
        category,
        budget_code,
        direction: direction.to_string(),
        amount_min: String::new(),
        amount_max: String::new(),
        notes: tr("Generated from detected transaction pattern."),
    }
}

fn most_common_text<'a>(items: impl Iterator<Item = &'a str>) -> Option<String> {
    let mut counts = HashMap::<String, usize>::new();
    for item in items {
        *counts.entry(item.to_string()).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
        .map(|(item, _)| item)
}
