use super::shared::{new_record_dialog, scroll_to_bottom};
use super::*;

pub(in crate::app) struct NewRuleDialogRequest<'a> {
    pub(in crate::app) parent: &'a adw::Dialog,
    pub(in crate::app) container: &'a gtk::Box,
    pub(in crate::app) forms: &'a Rc<RefCell<Vec<RuleForm>>>,
    pub(in crate::app) scrolled_window: &'a gtk::ScrolledWindow,
    pub(in crate::app) status: &'a gtk::Label,
    pub(in crate::app) filter_entry: &'a gtk::SearchEntry,
    pub(in crate::app) advanced_autofill: &'a Rc<Cell<bool>>,
    pub(in crate::app) advanced_features: bool,
}

pub(in crate::app) fn show_new_rule_dialog(request: NewRuleDialogRequest<'_>) {
    let NewRuleDialogRequest {
        parent,
        container,
        forms,
        scrolled_window,
        status,
        filter_entry,
        advanced_autofill,
        advanced_features,
    } = request;
    let rule = EditableRule::new_default();
    let (dialog, page, add_button, dialog_status) = new_record_dialog(
        "New Rule",
        "Fill in the rule first. It is only saved when you press Save.",
        "Add",
    );

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
    let search = rule_search_text_view("");
    let search_area = rule_search_text_area(&search);
    let search_combo = advanced_features.then(|| ui::text_combo("", editable_rule_search_values()));
    let is_regex = gtk::Switch::builder()
        .active(false)
        .valign(gtk::Align::Center)
        .build();
    let search_chips_editor =
        (!advanced_features).then(|| rule_search_chips_editor(&rule, &search, &is_regex));
    let category = ui::text_combo("", editable_category_values());
    let budget_code = ui::text_combo(&rule.budget_code, editable_budget_code_values());
    let direction = combo_from_options(
        &[
            ("any", "All transactions"),
            ("expense", "Expenses"),
            ("income", "Income"),
            ("transfer", "Transfers"),
        ],
        "expense",
    );
    connect_budget_fields_autofill(
        &category,
        &budget_code,
        &direction,
        editable_budget_autofill_entries(),
        advanced_autofill,
    );
    let amount_min = entry("", "Optional");
    let amount_max = entry("", "Optional");
    let notes = entry("", "Note");

    if advanced_features {
        add_labeled(&grid, 0, "Active", &active);
        add_labeled(&grid, 1, "Priority", &priority);
        add_labeled(&grid, 2, "Field", &field);
        if let Some(search_combo) = &search_combo {
            add_labeled(&grid, 3, "Search Text", search_combo);
        } else {
            add_labeled(&grid, 3, "Search Text", &search_area);
        }
        add_labeled(&grid, 4, "Regex", &is_regex);
        add_labeled(&grid, 5, "Category", &category);
        add_labeled(&grid, 6, "Budget code", &budget_code);
        add_labeled(&grid, 7, "Direction", &direction);
        add_labeled(&grid, 8, "Min amount", &amount_min);
        add_labeled(&grid, 9, "Max amount", &amount_max);
        add_labeled(&grid, 10, "Note", &notes);
    } else if let Some(editor) = &search_chips_editor {
        add_labeled(&grid, 0, "Search Text", &editor.container);
        add_labeled(&grid, 1, "Category", &category);
        add_labeled(&grid, 2, "Note", &notes);
    }
    page.append(&grid);
    page.append(&dialog_status);
    if let Some(editor) = &search_chips_editor {
        dialog.set_focus(Some(&editor.entry));
    } else {
        dialog.set_focus(Some(&search));
    }

    let container_for_add = container.clone();
    let forms_for_add = Rc::clone(forms);
    let scrolled_window_for_add = scrolled_window.clone();
    let status_for_add = status.clone();
    let dialog_for_add = dialog.clone();
    let filter_entry_for_add = filter_entry.clone();
    let advanced_autofill_for_add = Rc::clone(advanced_autofill);
    add_button.connect_clicked(move |_| {
        let search_text = search_combo
            .as_ref()
            .map(ui::combo_text)
            .unwrap_or_else(|| rule_search_text(&search));
        let category_text = ui::combo_text(&category);
        if search_text.is_empty() {
            dialog_status.set_text(&tr("Enter search text first."));
            if let Some(editor) = &search_chips_editor {
                editor.focus_entry();
            } else if let Some(search_combo) = &search_combo {
                search_combo.grab_focus();
            } else {
                search.grab_focus();
            }
            return;
        }
        if category_text.is_empty() {
            dialog_status.set_text(&tr("Enter a category first."));
            category.grab_focus();
            return;
        }

        let rule = EditableRule {
            priority: priority.value_as_int(),
            active: active.is_active(),
            field: combo_active_id(&field),
            search: search_text,
            is_regex: is_regex.is_active(),
            category: category_text,
            budget_code: ui::combo_text(&budget_code),
            direction: combo_active_id(&direction),
            amount_min: amount_min.text().trim().to_string(),
            amount_max: amount_max.text().trim().to_string(),
            notes: notes.text().trim().to_string(),
        };
        append_rule_form(
            &container_for_add,
            &forms_for_add,
            rule,
            false,
            &advanced_autofill_for_add,
            advanced_features,
        );
        filter_rule_forms(&filter_entry_for_add.text(), &forms_for_add.borrow());
        status_for_add.set_text(&tr("New rule added. Press Save to keep it."));
        scroll_to_bottom(&scrolled_window_for_add);
        dialog_for_add.close();
    });

    dialog.present(Some(parent));
}
