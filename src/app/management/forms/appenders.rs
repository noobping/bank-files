use super::*;

const EDIT_DETAILS_ICON: &str = "document-edit-symbolic";
const COLLAPSE_DETAILS_ICON: &str = "go-up-symbolic";

struct CollapsibleFormCard {
    form_box: gtk::Box,
    drag_handle: gtk::Button,
    title: gtk::Label,
    subtitle: gtk::Label,
    details: gtk::Box,
    revert_button: gtk::Button,
    delete_button: gtk::Button,
}

fn collapsible_form_card(title: &str, subtitle: &str, delete_tooltip: &str) -> CollapsibleFormCard {
    let form_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    form_box.add_css_class("card");
    form_box.set_margin_top(4);
    form_box.set_margin_bottom(4);
    form_box.set_margin_start(4);
    form_box.set_margin_end(4);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header.set_margin_top(12);
    header.set_margin_bottom(12);
    header.set_margin_start(12);
    header.set_margin_end(12);

    let drag_handle = ui::icon_button("list-drag-handle-symbolic", "Move rule");
    drag_handle.add_css_class("flat");
    drag_handle.set_visible(false);
    drag_handle.set_focus_on_click(false);

    let summary = gtk::Box::new(gtk::Orientation::Vertical, 2);
    summary.set_hexpand(true);
    let title_label = ui::wrapped_label(title);
    title_label.add_css_class("title-4");
    let subtitle_label = ui::wrapped_label(subtitle);
    subtitle_label.add_css_class("dim-label");
    summary.append(&title_label);
    summary.append(&subtitle_label);

    let edit_button = ui::icon_button(EDIT_DETAILS_ICON, "Edit details");
    edit_button.add_css_class("flat");
    let revert_button = ui::icon_button("document-revert-symbolic", "Revert details");
    revert_button.add_css_class("flat");
    revert_button.set_sensitive(false);
    let delete_button = ui::icon_button("user-trash-symbolic", delete_tooltip);
    delete_button.add_css_class("destructive-action");

    let actions = ui::linked_button_group();
    actions.append(&edit_button);
    actions.append(&revert_button);
    actions.append(&delete_button);

    header.append(&drag_handle);
    header.append(&summary);
    header.append(&actions);
    form_box.append(&header);

    let details = gtk::Box::new(gtk::Orientation::Vertical, 0);
    details.set_visible(false);
    details.set_sensitive(false);
    form_box.append(&details);

    let details_for_edit = details.clone();
    edit_button.connect_clicked(move |button| {
        let expanded = !details_for_edit.is_visible();
        details_for_edit.set_visible(expanded);
        details_for_edit.set_sensitive(expanded);
        button.set_icon_name(if expanded {
            COLLAPSE_DETAILS_ICON
        } else {
            EDIT_DETAILS_ICON
        });
        button.set_tooltip_text(Some(&tr(if expanded {
            "Collapse details"
        } else {
            "Edit details"
        })));
    });

    CollapsibleFormCard {
        form_box,
        drag_handle,
        title: title_label,
        subtitle: subtitle_label,
        details,
        revert_button,
        delete_button,
    }
}

fn set_text_combo(combo: &gtk::ComboBoxText, value: &str) {
    let value = value.trim();
    combo.set_active_id(if value.is_empty() { None } else { Some(value) });
    if let Some(entry) = combo
        .child()
        .and_then(|child| child.downcast::<gtk::Entry>().ok())
    {
        entry.set_text(value);
    }
}

fn set_option_combo(combo: &gtk::ComboBoxText, value: &str) {
    combo.set_active_id(Some(value.trim()));
    if combo.active_id().is_none() {
        combo.set_active(Some(0));
    }
}

fn connect_simple_budget_direction_inference(
    category: &gtk::ComboBoxText,
    code: &gtk::ComboBoxText,
    direction: &gtk::ComboBoxText,
) {
    let category_for_update = category.clone();
    let code_for_update = code.clone();
    let direction_for_update = direction.clone();
    let update_direction: Rc<dyn Fn()> = Rc::new(move || {
        let inferred = BudgetDirection::parse(
            "",
            &ui::combo_text(&code_for_update),
            &ui::combo_text(&category_for_update),
        );
        set_option_combo(&direction_for_update, inferred.as_str());
    });
    connect_combo_summary(category, &update_direction);
    connect_combo_summary(code, &update_direction);
    update_direction();
}

fn connect_delete_button(button: &gtk::Button, deleted: &Rc<Cell<bool>>, form_box: &gtk::Box) {
    let deleted_for_button = Rc::clone(deleted);
    let form_box_for_delete = form_box.clone();
    button.connect_clicked(move |_| {
        deleted_for_button.set(true);
        form_box_for_delete.set_visible(false);
    });
}

fn connect_budget_delete_button(
    button: &gtk::Button,
    deleted: &Rc<Cell<bool>>,
    form_box: &gtk::Box,
) {
    let button_for_delete = button.clone();
    let deleted_for_button = Rc::clone(deleted);
    let form_box_for_delete = form_box.clone();
    button.connect_clicked(move |_| {
        let should_delete = !deleted_for_button.get();
        set_budget_delete_state(
            &form_box_for_delete,
            &button_for_delete,
            &deleted_for_button,
            should_delete,
        );
    });
}

fn attach_details_grid(card: &CollapsibleFormCard, grid: &gtk::Grid) {
    grid.set_margin_start(12);
    grid.set_margin_end(12);
    grid.set_margin_bottom(12);
    card.details.append(grid);
}

fn connect_entry_summary(entry: &gtk::Entry, update: &Rc<dyn Fn()>) {
    let update_for_entry = Rc::clone(update);
    entry.connect_changed(move |_| update_for_entry());
}

fn connect_combo_summary(combo: &gtk::ComboBoxText, update: &Rc<dyn Fn()>) {
    let update_for_combo = Rc::clone(update);
    combo.connect_changed(move |_| update_for_combo());

    if let Some(entry) = combo
        .child()
        .and_then(|child| child.downcast::<gtk::Entry>().ok())
    {
        let update_for_entry = Rc::clone(update);
        entry.connect_changed(move |_| update_for_entry());
    }
}

fn connect_switch_summary(switch: &gtk::Switch, update: &Rc<dyn Fn()>) {
    let update_for_switch = Rc::clone(update);
    switch.connect_active_notify(move |_| update_for_switch());
}

fn connect_spin_summary(spin: &gtk::SpinButton, update: &Rc<dyn Fn()>) {
    let update_for_spin = Rc::clone(update);
    spin.connect_value_changed(move |_| update_for_spin());
}

fn set_summary(title: &gtk::Label, subtitle: &gtk::Label, values: (String, String)) {
    title.set_text(&values.0);
    subtitle.set_text(&values.1);
}

fn combo_display_text(combo: &gtk::ComboBoxText) -> String {
    combo
        .active_text()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| combo_active_id(combo))
}

fn entry_summary(entry: &gtk::Entry, fallback: &str) -> String {
    let text = entry.text().trim().to_string();
    if text.is_empty() {
        fallback.to_string()
    } else {
        text
    }
}

fn entry_summary_fixed_budget(entry: &gtk::Entry, fallback: &str) -> String {
    entry_summary_text(
        &planned_income::fixed_budget_amount_text(&entry.text()),
        fallback,
    )
}

struct RuleSummaryWidgets<'a> {
    active: &'a gtk::Switch,
    priority: &'a gtk::SpinButton,
    field: &'a gtk::ComboBoxText,
    search: &'a gtk::ComboBoxText,
    is_regex: &'a gtk::Switch,
    category: &'a gtk::ComboBoxText,
    budget_code: &'a gtk::ComboBoxText,
    direction: &'a gtk::ComboBoxText,
    amount_min: &'a gtk::Entry,
    amount_max: &'a gtk::Entry,
}

fn rule_summary(widgets: RuleSummaryWidgets<'_>) -> (String, String) {
    let title = format!(
        "{} · {}",
        entry_summary_text(&ui::combo_text(widgets.category), "Uncategorized"),
        entry_summary_text(&ui::combo_text(widgets.budget_code), "No budget code")
    );
    let match_kind = tr(if widgets.is_regex.is_active() {
        "regex"
    } else {
        "text"
    });
    let state = tr(if widgets.active.is_active() {
        "active"
    } else {
        "inactive"
    });
    let mut parts = vec![
        format!(
            "{}: {}",
            combo_display_text(widgets.field),
            ui::combo_text(widgets.search)
        ),
        combo_display_text(widgets.direction),
        format!(
            "{state} · {} {} · {match_kind}",
            tr("priority"),
            widgets.priority.value_as_int()
        ),
    ];
    let min = widgets.amount_min.text().trim().to_string();
    let max = widgets.amount_max.text().trim().to_string();
    if !min.is_empty() || !max.is_empty() {
        parts.push(format!("{} {min}..{max}", tr("amount")));
    }
    (title, parts.join(" · "))
}

fn budget_summary(
    code: &gtk::ComboBoxText,
    category: &gtk::ComboBoxText,
    monthly_budget: &gtk::Entry,
    yearly_budget: &gtk::Entry,
    direction: &gtk::ComboBoxText,
    income_basis: &gtk::ComboBoxText,
    show_code: bool,
) -> (String, String) {
    let code_text = ui::combo_text(code);
    let category_text = ui::combo_text(category);
    let planned_income = planned_income::is_budget_code(&code_text);
    let title = if show_code {
        format!(
            "{} · {}",
            entry_summary_text(&code_text, "No code"),
            entry_summary_text(&category_text, "Uncategorized")
        )
    } else {
        entry_summary_text(&category_text, "Uncategorized")
    };
    let mut parts = vec![
        combo_display_text(direction),
        format!(
            "{} {}",
            tr("monthly"),
            if planned_income {
                entry_summary_fixed_budget(monthly_budget, "-")
            } else {
                entry_summary(monthly_budget, "-")
            }
        ),
        format!(
            "{} {}",
            tr("yearly"),
            if planned_income {
                entry_summary_fixed_budget(yearly_budget, "-")
            } else {
                entry_summary(yearly_budget, "-")
            }
        ),
    ];
    if !planned_income
        && budget_values_use_percentage(&monthly_budget.text(), &yearly_budget.text())
    {
        parts.push(combo_display_text(income_basis));
    }
    (title, parts.join(" · "))
}

fn alias_summary(canonical: &gtk::ComboBoxText, alias: &gtk::Entry) -> (String, String) {
    let alias = entry_summary(alias, "No bank column");
    let canonical = combo_display_text(canonical);
    (
        format!("{alias} · {canonical}"),
        format!("{alias} -> {canonical}"),
    )
}

fn entry_summary_text(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.trim().to_string()
    }
}

struct RuleValueWidgets<'a> {
    active: &'a gtk::Switch,
    priority: &'a gtk::SpinButton,
    field: &'a gtk::ComboBoxText,
    search: &'a gtk::ComboBoxText,
    is_regex: &'a gtk::Switch,
    category: &'a gtk::ComboBoxText,
    budget_code: &'a gtk::ComboBoxText,
    direction: &'a gtk::ComboBoxText,
    amount_min: &'a gtk::Entry,
    amount_max: &'a gtk::Entry,
    notes: &'a gtk::Entry,
}

fn rule_value(widgets: RuleValueWidgets<'_>) -> EditableRule {
    EditableRule {
        priority: widgets.priority.value_as_int(),
        active: widgets.active.is_active(),
        field: combo_active_id(widgets.field),
        search: ui::combo_text(widgets.search),
        is_regex: widgets.is_regex.is_active(),
        category: ui::combo_text(widgets.category),
        budget_code: ui::combo_text(widgets.budget_code),
        direction: combo_active_id(widgets.direction),
        amount_min: widgets.amount_min.text().trim().to_string(),
        amount_max: widgets.amount_max.text().trim().to_string(),
        notes: widgets.notes.text().trim().to_string(),
    }
}

struct BudgetValueWidgets<'a> {
    code: &'a gtk::ComboBoxText,
    category: &'a gtk::ComboBoxText,
    monthly_budget: &'a gtk::Entry,
    yearly_budget: &'a gtk::Entry,
    direction: &'a gtk::ComboBoxText,
    income_basis: &'a gtk::ComboBoxText,
    notes: &'a gtk::Entry,
}

fn budget_value(widgets: BudgetValueWidgets<'_>) -> EditableBudget {
    let code = ui::combo_text(widgets.code);
    let planned_income = planned_income::is_budget_code(&code);
    EditableBudget {
        code,
        category: ui::combo_text(widgets.category),
        monthly_budget: if planned_income {
            planned_income::fixed_budget_amount_text(&widgets.monthly_budget.text())
        } else {
            widgets.monthly_budget.text().trim().to_string()
        },
        yearly_budget: if planned_income {
            planned_income::fixed_budget_amount_text(&widgets.yearly_budget.text())
        } else {
            widgets.yearly_budget.text().trim().to_string()
        },
        direction: if planned_income {
            "income".to_string()
        } else {
            combo_active_id(widgets.direction)
        },
        income_basis: if planned_income {
            "real".to_string()
        } else {
            combo_active_id(widgets.income_basis)
        },
        notes: widgets.notes.text().trim().to_string(),
    }
}

fn alias_value(canonical: &gtk::ComboBoxText, alias: &gtk::Entry) -> EditableAlias {
    EditableAlias {
        canonical: combo_active_id(canonical),
        alias: alias.text().trim().to_string(),
    }
}

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
        let title = card.title.clone();
        let subtitle = card.subtitle.clone();
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
                &title,
                &subtitle,
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

pub(in crate::app) fn append_planned_income_budget_form(
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<BudgetForm>>>,
    budget: EditableBudget,
    advanced_features: bool,
) {
    let card = collapsible_form_card("Planned Income", "", "Delete planned income budget");

    let grid = form_grid();
    let code = ui::text_combo(
        planned_income::BUDGET_CODE,
        [planned_income::BUDGET_CODE.to_string()],
    );
    code.set_sensitive(false);
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
        let title = card.title.clone();
        let subtitle = card.subtitle.clone();
        let code = code.clone();
        let category = category.clone();
        let monthly_budget = monthly_budget.clone();
        let yearly_budget = yearly_budget.clone();
        let direction = direction.clone();
        let income_basis = income_basis.clone();
        Rc::new(move || {
            set_summary(
                &title,
                &subtitle,
                budget_summary(
                    &code,
                    &category,
                    &monthly_budget,
                    &yearly_budget,
                    &direction,
                    &income_basis,
                    advanced_features,
                ),
            );
        })
    };
    connect_combo_summary(&category, &update_summary);
    connect_entry_summary(&monthly_budget, &update_summary);
    connect_entry_summary(&yearly_budget, &update_summary);
    update_summary();

    let original_budget = budget_value(BudgetValueWidgets {
        code: &code,
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
    connect_combo_summary(&category, &update_revert_state);
    connect_entry_summary(&monthly_budget, &update_revert_state);
    connect_entry_summary(&yearly_budget, &update_revert_state);
    connect_entry_summary(&notes, &update_revert_state);
    update_revert_state();

    let update_for_revert = Rc::clone(&update_summary);
    let update_revert_for_revert = Rc::clone(&update_revert_state);
    let category_for_revert = category.clone();
    let monthly_budget_for_revert = monthly_budget.clone();
    let yearly_budget_for_revert = yearly_budget.clone();
    let notes_for_revert = notes.clone();
    card.revert_button.connect_clicked(move |_| {
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
        original_code: Rc::new(RefCell::new(planned_income::BUDGET_CODE.to_string())),
        original_direction: Rc::new(RefCell::new(Some(BudgetDirection::Income))),
        auto_code: Rc::new(Cell::new(false)),
        code,
        category,
        monthly_budget,
        yearly_budget,
        direction,
        income_basis,
        notes,
        delete_button: card.delete_button,
    });
}

pub(in crate::app) fn append_budget_form(
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<BudgetForm>>>,
    budget: EditableBudget,
    persisted: bool,
    advanced_autofill: &Rc<Cell<bool>>,
    advanced_features: bool,
) {
    let original_direction = persisted
        .then(|| BudgetDirection::parse(&budget.direction, &budget.code, &budget.category));
    let card = collapsible_form_card("Budget", "", "Delete budget");

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
    if advanced_features {
        connect_budget_fields_autofill(
            &category,
            &code,
            &direction,
            editable_budget_autofill_entries(),
            advanced_autofill,
        );
    } else {
        direction.set_sensitive(false);
        if !persisted {
            connect_simple_budget_direction_inference(&category, &code, &direction);
        }
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
    attach_details_grid(&card, &grid);

    let update_summary: Rc<dyn Fn()> = {
        let title = card.title.clone();
        let subtitle = card.subtitle.clone();
        let code = code.clone();
        let category = category.clone();
        let monthly_budget = monthly_budget.clone();
        let yearly_budget = yearly_budget.clone();
        let direction = direction.clone();
        let income_basis = income_basis.clone();
        Rc::new(move || {
            set_summary(
                &title,
                &subtitle,
                budget_summary(
                    &code,
                    &category,
                    &monthly_budget,
                    &yearly_budget,
                    &direction,
                    &income_basis,
                    advanced_features,
                ),
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
        auto_code: Rc::new(Cell::new(!advanced_features && !persisted)),
        code,
        category,
        monthly_budget,
        yearly_budget,
        direction,
        income_basis,
        notes,
        delete_button: card.delete_button,
    });
}

pub(in crate::app) fn append_alias_form(
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<AliasForm>>>,
    alias: EditableAlias,
) {
    let card = collapsible_form_card("Field name", "", "Delete field name");

    let grid = form_grid();
    let canonical = field_alias_combo(&alias.canonical);
    let alias_entry = entry(&alias.alias, "Column name from bank CSV");
    add_labeled(&grid, 0, "Fixed field", &canonical);
    add_labeled(&grid, 1, "Bank column", &alias_entry);
    attach_details_grid(&card, &grid);

    let update_summary: Rc<dyn Fn()> = {
        let title = card.title.clone();
        let subtitle = card.subtitle.clone();
        let canonical = canonical.clone();
        let alias_entry = alias_entry.clone();
        Rc::new(move || set_summary(&title, &subtitle, alias_summary(&canonical, &alias_entry)))
    };
    connect_combo_summary(&canonical, &update_summary);
    connect_entry_summary(&alias_entry, &update_summary);
    update_summary();

    let original_alias = alias_value(&canonical, &alias_entry);
    let update_revert_state: Rc<dyn Fn()> = {
        let revert_button = card.revert_button.clone();
        let original_alias = original_alias.clone();
        let canonical = canonical.clone();
        let alias_entry = alias_entry.clone();
        Rc::new(move || {
            revert_button.set_sensitive(alias_value(&canonical, &alias_entry) != original_alias);
        })
    };
    connect_combo_summary(&canonical, &update_revert_state);
    connect_entry_summary(&alias_entry, &update_revert_state);
    update_revert_state();

    let update_for_revert = Rc::clone(&update_summary);
    let update_revert_for_revert = Rc::clone(&update_revert_state);
    let canonical_for_revert = canonical.clone();
    let alias_for_revert = alias_entry.clone();
    card.revert_button.connect_clicked(move |_| {
        set_option_combo(&canonical_for_revert, &original_alias.canonical);
        alias_for_revert.set_text(&original_alias.alias);
        update_for_revert();
        update_revert_for_revert();
    });

    let deleted = Rc::new(Cell::new(false));
    connect_delete_button(&card.delete_button, &deleted, &card.form_box);

    container.append(&card.form_box);
    forms.borrow_mut().push(AliasForm {
        form_box: card.form_box,
        deleted,
        canonical,
        alias: alias_entry,
    });
}
