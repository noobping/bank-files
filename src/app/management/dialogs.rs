use super::*;

pub(in crate::app) fn show_new_rule_dialog(
    parent: &adw::Dialog,
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<RuleForm>>>,
    scrolled_window: &gtk::ScrolledWindow,
    status: &gtk::Label,
    filter_entry: &gtk::SearchEntry,
    advanced_autofill: &Rc<Cell<bool>>,
) {
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
    let search = ui::text_combo("", editable_rule_search_values());
    let is_regex = gtk::Switch::builder()
        .active(false)
        .valign(gtk::Align::Center)
        .build();
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
    page.append(&grid);
    page.append(&dialog_status);
    dialog.set_focus(Some(&search));

    let container_for_add = container.clone();
    let forms_for_add = Rc::clone(forms);
    let scrolled_window_for_add = scrolled_window.clone();
    let status_for_add = status.clone();
    let dialog_for_add = dialog.clone();
    let filter_entry_for_add = filter_entry.clone();
    let advanced_autofill_for_add = Rc::clone(advanced_autofill);
    add_button.connect_clicked(move |_| {
        let search_text = ui::combo_text(&search);
        let category_text = ui::combo_text(&category);
        if search_text.is_empty() {
            dialog_status.set_text(&tr("Enter search text first."));
            search.grab_focus();
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
        );
        filter_rule_forms(&filter_entry_for_add.text(), &forms_for_add.borrow());
        status_for_add.set_text(&tr("New rule added. Press Save to keep it."));
        scroll_to_bottom(&scrolled_window_for_add);
        dialog_for_add.close();
    });

    dialog.present(Some(parent));
}

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
            generated_budget_code_for_category(&category_text, &existing_codes)
        };

        let direction_text = budget_direction_for_save(&combo_active_id(&direction));
        let budget = EditableBudget {
            code: code_text,
            category: category_text,
            monthly_budget: monthly_budget.text().trim().to_string(),
            yearly_budget: yearly_budget.text().trim().to_string(),
            direction: direction_text,
            income_basis: combo_active_id(&income_basis),
            notes: notes.text().trim().to_string(),
        };
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

pub(in crate::app) fn show_new_alias_dialog(
    parent: &adw::Dialog,
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<AliasForm>>>,
    scrolled_window: &gtk::ScrolledWindow,
    status: &gtk::Label,
    filter_entry: &gtk::SearchEntry,
) {
    let alias = EditableAlias::new_default();
    let (dialog, page, add_button, dialog_status) = new_record_dialog(
        "New Field Name",
        "Map a bank column to a fixed field. It is only saved when you press Save.",
        "Add",
    );

    let grid = form_grid();
    let canonical = field_alias_combo(&alias.canonical);
    let alias_entry = entry("", "Column name from bank CSV");
    add_labeled(&grid, 0, "Fixed field", &canonical);
    add_labeled(&grid, 1, "Bank column", &alias_entry);
    page.append(&grid);
    page.append(&dialog_status);
    dialog.set_focus(Some(&alias_entry));

    let container_for_add = container.clone();
    let forms_for_add = Rc::clone(forms);
    let scrolled_window_for_add = scrolled_window.clone();
    let status_for_add = status.clone();
    let dialog_for_add = dialog.clone();
    let filter_entry_for_add = filter_entry.clone();
    add_button.connect_clicked(move |_| {
        let alias_text = alias_entry.text().trim().to_string();
        if alias_text.is_empty() {
            dialog_status.set_text(&tr("Enter the bank column first."));
            alias_entry.grab_focus();
            return;
        }

        let alias = EditableAlias {
            canonical: combo_active_id(&canonical),
            alias: alias_text,
        };
        append_alias_form(&container_for_add, &forms_for_add, alias);
        filter_alias_forms(&filter_entry_for_add.text(), &forms_for_add.borrow());
        status_for_add.set_text(&tr("New field name added. Press Save to keep it."));
        scroll_to_bottom(&scrolled_window_for_add);
        dialog_for_add.close();
    });

    dialog.present(Some(parent));
}

pub(in crate::app) fn new_record_dialog(
    title: &str,
    subtitle: &str,
    add_label: &str,
) -> (adw::Dialog, gtk::Box, gtk::Button, gtk::Label) {
    let shell = build_action_dialog_shell(
        title,
        subtitle,
        add_label,
        "list-add-symbolic",
        "Add to changes",
        "Search",
    );
    shell.set_form_only();

    let page = ui::page_box();
    shell.add_form_page(&ui::action_dialog_scroll(&page));

    let dialog_status = ui::wrapped_label("");
    dialog_status.add_css_class("dim-label");

    let add_button = shell.submit_button.clone();
    let dialog = adw::Dialog::builder()
        .title(tr(title))
        .content_width(680)
        .content_height(-1)
        .default_widget(&add_button)
        .child(&shell.root)
        .build();

    (dialog, page, add_button, dialog_status)
}

pub(in crate::app) fn scroll_to_bottom(scrolled_window: &gtk::ScrolledWindow) {
    let scrolled_window = scrolled_window.clone();
    adw::glib::idle_add_local_once(move || {
        let adjustment = scrolled_window.vadjustment();
        let bottom = (adjustment.upper() - adjustment.page_size()).max(adjustment.lower());
        adjustment.set_value(bottom);
    });
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
