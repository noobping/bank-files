use super::*;

pub(in crate::app) const TRANSACTION_RULE_FIELD_OPTIONS: &[(&str, &str)] = &[
    ("any", "Everything"),
    ("counterparty", "Counterparty"),
    ("description", "Description"),
    ("tags", "Tags"),
    ("account", "Account"),
    ("transaction_id", "Transaction ID"),
];

const RULE_DIALOG_TITLE: &str = "Create Rule";
const RULE_DIALOG_SUBMIT_LABEL: &str = "Save";
const RULE_DIALOG_SUBMIT_ICON: &str = "document-save-symbolic";
const RULE_DIALOG_SUBMIT_TOOLTIP: &str = "Save rule";
const RULE_DIALOG_SEARCH_PLACEHOLDER: &str = "Search rule fields";
const RULE_QUEUE_HELP: &str = "Save adds this rule to the processing queue.";
const RULE_QUEUE_ADDED: &str = "Rule added to processing queue.";
const RULE_SEARCH_REQUIRED: &str = "Enter search text first.";
const RULE_CATEGORY_REQUIRED: &str = "Enter a category first.";

const RULE_DIRECTION_OPTIONS: &[(&str, &str)] = &[
    ("any", "All transactions"),
    ("expense", "Expenses"),
    ("income", "Income"),
    ("transfer", "Transfers"),
];

pub(in crate::app) struct RuleDialogSpec {
    pub(in crate::app) subtitle: &'static str,
    pub(in crate::app) content_width: i32,
    pub(in crate::app) field_options: &'static [(&'static str, &'static str)],
    pub(in crate::app) search_values: Vec<String>,
}

struct RuleDialogFields {
    active: gtk::Switch,
    priority: gtk::SpinButton,
    field: gtk::ComboBoxText,
    search: gtk::ComboBoxText,
    is_regex: gtk::Switch,
    category: gtk::ComboBoxText,
    budget_code: gtk::ComboBoxText,
    direction: gtk::ComboBoxText,
    amount_min: gtk::Entry,
    amount_max: gtk::Entry,
    notes: gtk::Entry,
}

pub(in crate::app) fn show_rule_enqueue_dialog(
    initial: EditableRule,
    spec: RuleDialogSpec,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let popup = build_action_form_dialog(
        RULE_DIALOG_TITLE,
        spec.subtitle,
        RULE_DIALOG_SUBMIT_LABEL,
        RULE_DIALOG_SUBMIT_ICON,
        RULE_DIALOG_SUBMIT_TOOLTIP,
        RULE_DIALOG_SEARCH_PLACEHOLDER,
        spec.content_width,
    );
    let save_button = popup.submit_button.clone();
    register_config_widget(ui_handles, &save_button);

    let status = popup.status.clone();
    let fields = build_rule_dialog_fields(&initial, &spec, state, ui_handles, &save_button);
    append_rule_dialog_fields(&popup.page, &fields, &status);
    popup.dialog.set_focus(Some(&fields.search));

    let ui_for_save = Rc::clone(ui_handles);
    let dialog_for_save = popup.dialog.clone();
    ui::connect_button_activation(&save_button, move |button| {
        let Some(rule) = rule_from_dialog_fields(&fields, &status) else {
            return;
        };

        if enqueue_rule_operation(&ui_for_save, rule, true, OperationSource::CreateRule).queued() {
            button.set_sensitive(false);
            status.set_text(&tr(RULE_QUEUE_ADDED));
            dialog_for_save.close();
        } else {
            status.set_text(&tr(operation_already_queued_status()));
        }
    });

    popup.dialog.present(Some(&ui_handles.window));
}

fn build_rule_dialog_fields(
    initial: &EditableRule,
    spec: &RuleDialogSpec,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    save_button: &gtk::Button,
) -> RuleDialogFields {
    let active = gtk::Switch::builder()
        .active(initial.active)
        .valign(gtk::Align::Center)
        .build();
    let priority = gtk::SpinButton::with_range(0.0, 1000.0, 1.0);
    priority.set_value(initial.priority as f64);
    let field = ui::combo_from_options(spec.field_options, &initial.field);
    let search = ui::text_combo(&initial.search, spec.search_values.clone());
    let is_regex = gtk::Switch::builder()
        .active(initial.is_regex)
        .valign(gtk::Align::Center)
        .build();
    let category = ui::text_combo(&initial.category, app_category_values(&state.borrow()));
    let budget_code = ui::text_combo(
        &initial.budget_code,
        app_budget_code_values(&state.borrow()),
    );
    let direction = ui::combo_from_options(RULE_DIRECTION_OPTIONS, &initial.direction);
    connect_budget_fields_autofill(
        &category,
        &budget_code,
        &direction,
        app_budget_autofill_entries(&state.borrow()),
        &ui_handles.advanced_autofill,
    );
    ui::focus_button_after_combo_selections(
        save_button,
        &[&field, &search, &category, &budget_code, &direction],
    );

    RuleDialogFields {
        active,
        priority,
        field,
        search,
        is_regex,
        category,
        budget_code,
        direction,
        amount_min: ui::entry(&initial.amount_min, "Optional"),
        amount_max: ui::entry(&initial.amount_max, "Optional"),
        notes: ui::entry(&initial.notes, "Note"),
    }
}

fn append_rule_dialog_fields(page: &gtk::Box, fields: &RuleDialogFields, status: &gtk::Label) {
    let grid = ui::form_grid();
    ui::add_labeled(&grid, 0, "Active", &fields.active);
    ui::add_labeled(&grid, 1, "Priority", &fields.priority);
    ui::add_labeled(&grid, 2, "Field", &fields.field);
    ui::add_labeled(&grid, 3, "Search Text", &fields.search);
    ui::add_labeled(&grid, 4, "Regex", &fields.is_regex);
    ui::add_labeled(&grid, 5, "Category", &fields.category);
    ui::add_labeled(&grid, 6, "Budget code", &fields.budget_code);
    ui::add_labeled(&grid, 7, "Direction", &fields.direction);
    ui::add_labeled(&grid, 8, "Min amount", &fields.amount_min);
    ui::add_labeled(&grid, 9, "Max amount", &fields.amount_max);
    ui::add_labeled(&grid, 10, "Note", &fields.notes);
    page.append(&grid);

    status.set_text(&tr(RULE_QUEUE_HELP));
    page.append(status);
}

fn rule_from_dialog_fields(fields: &RuleDialogFields, status: &gtk::Label) -> Option<EditableRule> {
    let search_text = ui::combo_text(&fields.search);
    let category_text = ui::combo_text(&fields.category);
    if search_text.is_empty() {
        status.set_text(&tr(RULE_SEARCH_REQUIRED));
        fields.search.grab_focus();
        return None;
    }
    if category_text.is_empty() {
        status.set_text(&tr(RULE_CATEGORY_REQUIRED));
        fields.category.grab_focus();
        return None;
    }

    Some(EditableRule {
        priority: fields.priority.value_as_int(),
        active: fields.active.is_active(),
        field: ui::combo_active_id(&fields.field),
        search: search_text,
        is_regex: fields.is_regex.is_active(),
        category: category_text,
        budget_code: ui::combo_text(&fields.budget_code),
        direction: ui::combo_active_id(&fields.direction),
        amount_min: fields.amount_min.text().trim().to_string(),
        amount_max: fields.amount_max.text().trim().to_string(),
        notes: fields.notes.text().trim().to_string(),
    })
}
