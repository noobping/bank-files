use super::*;

const MANAGEMENT_DIALOG_PARENT_INSET: i32 = 48;
const MANAGEMENT_DIALOG_MIN_WIDTH: i32 = 320;
const MANAGEMENT_DIALOG_MIN_HEIGHT: i32 = 360;
const MANAGEMENT_DIALOG_FALLBACK_WIDTH: i32 = 1040;
const MANAGEMENT_DIALOG_FALLBACK_HEIGHT: i32 = 760;
const MANAGEMENT_FORM_RENDER_BATCH_SIZE: usize = 18;

pub(in crate::app) fn show_management_dialog(
    window: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    initial_tab: &str,
) -> bool {
    if !try_begin_config_operation(ui_handles, "Rules, budgets, and fields is already open.") {
        return false;
    }

    let finish_called = Rc::new(Cell::new(false));
    let ui_for_finish = Rc::clone(ui_handles);
    let finish_management_dialog: Rc<dyn Fn()> = Rc::new(move || {
        if finish_called.replace(true) {
            return;
        }
        finish_config_operation(&ui_for_finish);
    });
    let dialog_closed = Rc::new(Cell::new(false));
    let save_running = Rc::new(Cell::new(false));
    let advanced_features = ui_handles.advanced_features.get();
    show_verbose_status(
        ui_handles.as_ref(),
        format!(
            "management dialog opening; tab={initial_tab}; advanced_features={advanced_features}"
        ),
    );

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let stack = adw::ViewStack::new();
    stack.set_vexpand(true);

    let header = adw::HeaderBar::new();
    header.set_show_start_title_buttons(false);
    header.set_show_end_title_buttons(false);
    let switcher = adw::ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();
    header.set_title_widget(Some(&switcher));
    let cancel_button = gtk::Button::with_label(&tr("Cancel"));
    cancel_button.add_css_class("flat");
    let add_button = ui::plain_text_icon_button("list-add-symbolic", "New", "New item");
    let save_button = ui::primary_text_icon_button(
        "document-save-symbolic",
        "Save",
        "Save rules, budgets, and field names",
    );
    let action_buttons = ui::linked_button_group();
    action_buttons.append(&add_button);
    action_buttons.append(&save_button);
    header.pack_start(&cancel_button);
    header.pack_end(&action_buttons);
    root.append(&header);

    let filter_placeholder = if advanced_features {
        "Filter rules, budgets, and field names"
    } else {
        "Filter budgets and field names"
    };
    let filter_entry = gtk::SearchEntry::builder()
        .placeholder_text(tr(filter_placeholder))
        .hexpand(true)
        .build();
    let filter_row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    filter_row.set_margin_top(8);
    filter_row.set_margin_bottom(8);
    filter_row.set_margin_start(12);
    filter_row.set_margin_end(12);
    filter_row.append(&filter_entry);
    let filter_search_bar = gtk::SearchBar::builder()
        .child(&filter_row)
        .show_close_button(true)
        .search_mode_enabled(false)
        .build();
    filter_search_bar.connect_entry(&filter_entry);
    root.append(&filter_search_bar);

    let rules_forms: Rc<RefCell<Vec<RuleForm>>> = Rc::new(RefCell::new(Vec::new()));
    let budgets_forms: Rc<RefCell<Vec<BudgetForm>>> = Rc::new(RefCell::new(Vec::new()));
    let aliases_forms: Rc<RefCell<Vec<AliasForm>>> = Rc::new(RefCell::new(Vec::new()));

    let rules_box = ui::page_box();
    rules_box.append(&ui::section_title(
        "Categorization Rules",
        "Create rules with plain search text, or turn on Regex for patterns.",
    ));
    let rules_list = gtk::Box::new(gtk::Orientation::Vertical, 8);
    rules_box.append(&rules_list);
    append_management_loading(&rules_list, "Loading rules...");
    let add_rule_button =
        ui::plain_text_icon_button("list-add-symbolic", "New Rule", "Create a new rule");
    let group_rules_button = ui::plain_text_icon_button(
        "view-sort-ascending-symbolic",
        "Group",
        "Move compatible rules next to each other before combining",
    );
    let combine_rules_button = ui::plain_text_icon_button(
        "view-refresh-symbolic",
        "Combine",
        "Combine adjacent compatible rules",
    );
    let rules_actions = ui::linked_button_group();
    rules_actions.append(&add_rule_button);
    rules_actions.append(&group_rules_button);
    rules_actions.append(&combine_rules_button);
    rules_box.append(&rules_actions);

    let budgets_box = ui::page_box();
    let budgets_title = if advanced_features {
        "Budget Codes"
    } else {
        "Budgets"
    };
    let budgets_description = if advanced_features {
        "Use fixed amounts or percentages; choose real or planned income for percentages."
    } else {
        "Use categories with fixed amounts or percentages."
    };
    budgets_box.append(&ui::section_title(budgets_title, budgets_description));
    let budgets_list = gtk::Box::new(gtk::Orientation::Vertical, 8);
    budgets_box.append(&budgets_list);
    append_management_loading(&budgets_list, "Loading budgets...");
    let add_budget_button = ui::plain_text_icon_button(
        "list-add-symbolic",
        if advanced_features {
            "New Budget"
        } else {
            "New Category"
        },
        if advanced_features {
            "Create a new budget"
        } else {
            "Create a new category with monthly or yearly amounts"
        },
    );
    let move_budget_code_button = ui::plain_text_icon_button(
        "send-to-symbolic",
        "Move Code",
        "Move rules from one budget code to another",
    );
    let use_real_income_button = ui::plain_text_icon_button(
        "view-refresh-symbolic",
        "Real Income",
        "Set every budget percentage basis to real income",
    );
    let use_planned_income_button = ui::plain_text_icon_button(
        "view-refresh-symbolic",
        "Planned Income",
        "Set every budget percentage basis to planned income",
    );
    let use_monthly_values_button = ui::plain_text_icon_button(
        "go-previous-symbolic",
        "Monthly",
        "Convert budget values to monthly values",
    );
    let use_yearly_values_button = ui::plain_text_icon_button(
        "go-next-symbolic",
        "Yearly",
        "Convert budget values to yearly values",
    );
    let budget_actions = adw::WrapBox::builder()
        .orientation(gtk::Orientation::Horizontal)
        .child_spacing(8)
        .child_spacing_unit(adw::LengthUnit::Px)
        .line_spacing(6)
        .line_spacing_unit(adw::LengthUnit::Px)
        .natural_line_length(720)
        .natural_line_length_unit(adw::LengthUnit::Px)
        .wrap_policy(adw::WrapPolicy::Natural)
        .hexpand(true)
        .halign(gtk::Align::Fill)
        .build();
    let budget_create_actions = ui::linked_button_group();
    budget_create_actions.append(&add_budget_button);
    if advanced_features {
        budget_create_actions.append(&move_budget_code_button);
    }
    budget_actions.append(&budget_create_actions);
    if advanced_features {
        let budget_income_actions = ui::linked_button_group();
        budget_income_actions.append(&use_real_income_button);
        budget_income_actions.append(&use_planned_income_button);
        let budget_value_actions = ui::linked_button_group();
        budget_value_actions.append(&use_monthly_values_button);
        budget_value_actions.append(&use_yearly_values_button);
        budget_actions.append(&budget_income_actions);
        budget_actions.append(&budget_value_actions);
    }
    budgets_box.append(&budget_actions);

    let aliases_box = ui::page_box();
    aliases_box.append(&ui::section_title(
        "Normalize CSV Fields",
        "Map bank columns to fixed fields such as date, amount, and description.",
    ));
    let aliases_list = gtk::Box::new(gtk::Orientation::Vertical, 8);
    aliases_box.append(&aliases_list);
    append_management_loading(&aliases_list, "Loading field names...");
    let add_alias_button = ui::plain_text_icon_button(
        "list-add-symbolic",
        "New Field Name",
        "Create a new field name",
    );
    let aliases_actions = ui::linked_button_group();
    aliases_actions.append(&add_alias_button);
    aliases_box.append(&aliases_actions);

    let rules_scroll = ui::scroll(&rules_box);
    let budgets_scroll = ui::scroll(&budgets_box);
    let aliases_scroll = ui::scroll(&aliases_box);
    if advanced_features {
        stack
            .add_titled(&rules_scroll, Some("rules"), &tr("Rules"))
            .set_icon_name(Some("document-edit-symbolic"));
    }
    stack
        .add_titled(&budgets_scroll, Some("budgets"), &tr("Budgets"))
        .set_icon_name(Some("view-list-symbolic"));
    stack
        .add_titled(&aliases_scroll, Some("aliases"), &tr("Normalize"))
        .set_icon_name(Some("format-justify-left-symbolic"));
    let initial_filter = (advanced_features && initial_tab == "active-rules").then_some("active");
    let initial_tab = match initial_tab {
        "active-rules" | "rules" if advanced_features => "rules",
        "budgets" | "aliases" => initial_tab,
        _ => "budgets",
    };
    stack.set_visible_child_name(initial_tab);
    root.append(&stack);

    let switcher_bar = adw::ViewSwitcherBar::builder()
        .stack(&stack)
        .reveal(false)
        .build();
    root.append(&switcher_bar);

    let status_bar = build_status_bar();
    connect_embedded_status_bar(window, &status_bar, Rc::clone(&ui_handles.status_autohide));
    set_page_actions_menu_namespace(&status_bar.page_actions_button, "management");
    status_bar.page_actions_button.set_sensitive(false);
    let status_handle = StatusHandle::from_status_bar(&status_bar);
    status_handle.set_text(&tr("Loading management data..."));
    status_handle.set_loading(true);
    root.append(&status_bar.container);
    let status = status_bar.label.clone();

    let (content_width, content_height) = management_dialog_content_size(window);
    let management_title = if advanced_features {
        "Rules, budgets, and fields"
    } else {
        "Budgets and fields"
    };
    let management_dialog = adw::Dialog::builder()
        .title(tr(management_title))
        .width_request(MANAGEMENT_DIALOG_MIN_WIDTH)
        .height_request(MANAGEMENT_DIALOG_MIN_HEIGHT)
        .content_width(content_width)
        .content_height(content_height)
        .child(&root)
        .build();
    let management_dialog_for_cancel = management_dialog.clone();
    let dialog_closed_for_cancel = Rc::clone(&dialog_closed);
    let save_running_for_cancel = Rc::clone(&save_running);
    let finish_for_cancel = Rc::clone(&finish_management_dialog);
    cancel_button.connect_clicked(move |_| {
        dialog_closed_for_cancel.set(true);
        management_dialog_for_cancel.close();
        if !save_running_for_cancel.get() {
            finish_for_cancel();
        }
    });
    let dialog_closed_for_closed = Rc::clone(&dialog_closed);
    let save_running_for_closed = Rc::clone(&save_running);
    let finish_for_closed = Rc::clone(&finish_management_dialog);
    let ui_for_closed = Rc::clone(ui_handles);
    management_dialog.connect_closed(move |_| {
        dialog_closed_for_closed.set(true);
        show_verbose_status(ui_for_closed.as_ref(), "management dialog closed");
        if !save_running_for_closed.get() {
            finish_for_closed();
        }
    });
    filter_search_bar.set_key_capture_widget(Some(&management_dialog));
    add_responsive_switcher_for_dialog(&management_dialog, &switcher, &switcher_bar);

    connect_management_dialog_actions(ManagementDialogActions {
        management_dialog: &management_dialog,
        add_button: &add_button,
        add_rule_button: &add_rule_button,
        group_rules_button: &group_rules_button,
        combine_rules_button: &combine_rules_button,
        add_budget_button: &add_budget_button,
        move_budget_code_button: &move_budget_code_button,
        use_real_income_button: &use_real_income_button,
        use_planned_income_button: &use_planned_income_button,
        use_monthly_values_button: &use_monthly_values_button,
        use_yearly_values_button: &use_yearly_values_button,
        add_alias_button: &add_alias_button,
        cancel_button: &cancel_button,
        save_button: &save_button,
        page_actions_button: &status_bar.page_actions_button,
        stack: &stack,
        filter_entry: &filter_entry,
        filter_search_bar: &filter_search_bar,
        rules_list: &rules_list,
        rules_forms: &rules_forms,
        rules_scroll: &rules_scroll,
        budgets_list: &budgets_list,
        budgets_forms: &budgets_forms,
        budgets_scroll: &budgets_scroll,
        aliases_list: &aliases_list,
        aliases_forms: &aliases_forms,
        aliases_scroll: &aliases_scroll,
        status: &status,
        dialog_closed: Rc::clone(&dialog_closed),
        save_running: Rc::clone(&save_running),
        finish_management_dialog: Rc::clone(&finish_management_dialog),
        state,
        ui_handles,
    });
    let management_form_action_buttons = vec![
        add_button.clone(),
        add_rule_button.clone(),
        group_rules_button.clone(),
        combine_rules_button.clone(),
        add_budget_button.clone(),
        move_budget_code_button.clone(),
        use_real_income_button.clone(),
        use_planned_income_button.clone(),
        use_monthly_values_button.clone(),
        use_yearly_values_button.clone(),
        add_alias_button.clone(),
        save_button.clone(),
    ];
    set_management_form_action_buttons_sensitive(&management_form_action_buttons, false);
    status_bar.page_actions_button.set_sensitive(false);
    if let Some(filter) = initial_filter {
        filter_search_bar.set_search_mode(true);
        filter_entry.set_text(filter);
    }
    management_dialog.present(Some(window));
    schedule_management_forms_load(ManagementFormsLoad {
        advanced_features,
        rules_list,
        rules_forms,
        budgets_list,
        budgets_forms,
        aliases_list,
        aliases_forms,
        filter_entry,
        status,
        dialog_closed,
        advanced_autofill: Rc::clone(&ui_handles.advanced_autofill),
        ui_handles: Rc::clone(ui_handles),
        buttons: management_form_action_buttons,
        page_actions_button: status_bar.page_actions_button.clone(),
        status_handle,
    });
    true
}

struct ManagementFormsLoad {
    advanced_features: bool,
    rules_list: gtk::Box,
    rules_forms: Rc<RefCell<Vec<RuleForm>>>,
    budgets_list: gtk::Box,
    budgets_forms: Rc<RefCell<Vec<BudgetForm>>>,
    aliases_list: gtk::Box,
    aliases_forms: Rc<RefCell<Vec<AliasForm>>>,
    filter_entry: gtk::SearchEntry,
    status: gtk::Label,
    dialog_closed: Rc<Cell<bool>>,
    advanced_autofill: Rc<Cell<bool>>,
    ui_handles: Rc<UiHandles>,
    buttons: Vec<gtk::Button>,
    page_actions_button: gtk::MenuButton,
    status_handle: StatusHandle,
}

struct ManagementLoadedForms {
    rules: Result<std::collections::VecDeque<EditableRule>, String>,
    budgets: Result<std::collections::VecDeque<EditableBudget>, String>,
    aliases: Result<std::collections::VecDeque<EditableAlias>, String>,
}

struct ManagementFormsRender {
    load: ManagementFormsLoad,
    loaded: ManagementLoadedForms,
    stage: ManagementFormsRenderStage,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ManagementFormsRenderStage {
    Rules,
    Budgets,
    Aliases,
    Done,
}

fn append_management_loading(container: &gtk::Box, message: &str) {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.add_css_class("dim-label");
    row.set_margin_top(8);
    row.set_margin_bottom(8);
    row.set_margin_start(4);
    row.set_margin_end(4);
    row.set_valign(gtk::Align::Center);

    let spinner = ui::loading_spinner();
    spinner.set_size_request(18, 18);
    row.append(&spinner);
    row.append(&ui::wrapped_label(&tr(message)));
    container.append(&row);
}

fn schedule_management_forms_load(load: ManagementFormsLoad) {
    show_verbose_status(load.ui_handles.as_ref(), "management forms load started");
    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(load_management_forms_data);
        match task.await {
            Ok(loaded) => {
                if load.dialog_closed.get() {
                    show_verbose_status(
                        load.ui_handles.as_ref(),
                        "management forms load finished after dialog closed",
                    );
                    return;
                }
                show_verbose_status(
                    load.ui_handles.as_ref(),
                    format!(
                        "management forms loaded; {}",
                        management_loaded_forms_summary(&loaded)
                    ),
                );
                start_management_forms_render(load, loaded);
            }
            Err(_) => {
                show_verbose_status(
                    load.ui_handles.as_ref(),
                    "management forms load task canceled",
                );
                load.status_handle.set_loading(false);
                load.status.set_text(&tr(
                    "Management loading canceled: the background task stopped unexpectedly.",
                ));
            }
        }
    });
}

fn load_management_forms_data() -> ManagementLoadedForms {
    ManagementLoadedForms {
        rules: data::load_editable_rules()
            .map(std::collections::VecDeque::from)
            .map_err(|err| format!("{err:#}")),
        budgets: data::load_editable_budgets()
            .map(ordered_management_budgets)
            .map(std::collections::VecDeque::from)
            .map_err(|err| format!("{err:#}")),
        aliases: data::load_editable_aliases()
            .map(std::collections::VecDeque::from)
            .map_err(|err| format!("{err:#}")),
    }
}

fn management_loaded_forms_summary(loaded: &ManagementLoadedForms) -> String {
    format!(
        "rules={}; budgets={}; aliases={}",
        management_loaded_count(&loaded.rules),
        management_loaded_count(&loaded.budgets),
        management_loaded_count(&loaded.aliases)
    )
}

fn management_loaded_count<T>(result: &Result<std::collections::VecDeque<T>, String>) -> String {
    match result {
        Ok(items) => items.len().to_string(),
        Err(_) => "error".to_string(),
    }
}

fn ordered_management_budgets(budgets: Vec<EditableBudget>) -> Vec<EditableBudget> {
    let (planned_income_budget, mut regular_budgets) = partition_planned_income_budget(budgets);
    let mut ordered =
        Vec::with_capacity(regular_budgets.len() + usize::from(planned_income_budget.is_some()));
    if let Some(planned_income_budget) = planned_income_budget {
        ordered.push(planned_income_budget);
    }
    ordered.append(&mut regular_budgets);
    ordered
}

fn start_management_forms_render(load: ManagementFormsLoad, loaded: ManagementLoadedForms) {
    show_verbose_status(
        load.ui_handles.as_ref(),
        format!(
            "management forms render started; batch_size={MANAGEMENT_FORM_RENDER_BATCH_SIZE}; {}",
            management_loaded_forms_summary(&loaded)
        ),
    );
    ui::clear_box(&load.rules_list);
    load.rules_forms.borrow_mut().clear();
    ui::clear_box(&load.budgets_list);
    load.budgets_forms.borrow_mut().clear();
    ui::clear_box(&load.aliases_list);
    load.aliases_forms.borrow_mut().clear();
    load.status_handle
        .set_text(&tr("Loading management data..."));
    let render = Rc::new(RefCell::new(ManagementFormsRender {
        load,
        loaded,
        stage: ManagementFormsRenderStage::Rules,
    }));
    schedule_management_forms_render(render);
}

fn schedule_management_forms_render(render: Rc<RefCell<ManagementFormsRender>>) {
    gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(1), move || {
        if render.borrow().load.dialog_closed.get() {
            return;
        }
        if render_management_forms_batch(&render) {
            schedule_management_forms_render(render);
        }
    });
}

fn render_management_forms_batch(render: &Rc<RefCell<ManagementFormsRender>>) -> bool {
    let mut render = render.borrow_mut();
    let mut remaining = MANAGEMENT_FORM_RENDER_BATCH_SIZE;
    while remaining > 0 {
        match render.stage {
            ManagementFormsRenderStage::Rules => {
                if render_rule_forms_batch(&mut render, &mut remaining) {
                    continue;
                }
            }
            ManagementFormsRenderStage::Budgets => {
                if render_budget_forms_batch(&mut render, &mut remaining) {
                    continue;
                }
            }
            ManagementFormsRenderStage::Aliases => {
                if render_alias_forms_batch(&mut render, &mut remaining) {
                    continue;
                }
            }
            ManagementFormsRenderStage::Done => {
                finish_management_forms_render(&render.load);
                return false;
            }
        }
    }
    true
}

fn render_rule_forms_batch(render: &mut ManagementFormsRender, remaining: &mut usize) -> bool {
    match &mut render.loaded.rules {
        Ok(rules) => {
            while *remaining > 0 {
                let Some(rule) = rules.pop_front() else {
                    show_verbose_status(
                        render.load.ui_handles.as_ref(),
                        format!(
                            "management rules render finished; forms={}",
                            render.load.rules_forms.borrow().len()
                        ),
                    );
                    render.stage = ManagementFormsRenderStage::Budgets;
                    return true;
                };
                append_rule_form(
                    &render.load.rules_list,
                    &render.load.rules_forms,
                    rule,
                    true,
                    &render.load.advanced_autofill,
                );
                *remaining -= 1;
            }
            false
        }
        Err(err) => {
            show_verbose_status(
                render.load.ui_handles.as_ref(),
                format!("management rules render failed; error={err}"),
            );
            render
                .load
                .rules_list
                .append(&ui::selectable_wrapped_label(&trf(
                    "Could not read rules: {error}",
                    &[("error", err.clone())],
                )));
            render.stage = ManagementFormsRenderStage::Budgets;
            true
        }
    }
}

fn render_budget_forms_batch(render: &mut ManagementFormsRender, remaining: &mut usize) -> bool {
    match &mut render.loaded.budgets {
        Ok(budgets) => {
            while *remaining > 0 {
                let Some(budget) = budgets.pop_front() else {
                    show_verbose_status(
                        render.load.ui_handles.as_ref(),
                        format!(
                            "management budgets render finished; forms={}",
                            render.load.budgets_forms.borrow().len()
                        ),
                    );
                    render.stage = ManagementFormsRenderStage::Aliases;
                    return true;
                };
                if planned_income::is_budget_code(&budget.code) {
                    append_planned_income_budget_form(
                        &render.load.budgets_list,
                        &render.load.budgets_forms,
                        budget,
                        render.load.advanced_features,
                    );
                } else {
                    append_budget_form(
                        &render.load.budgets_list,
                        &render.load.budgets_forms,
                        budget,
                        true,
                        &render.load.advanced_autofill,
                        render.load.advanced_features,
                    );
                }
                *remaining -= 1;
            }
            false
        }
        Err(err) => {
            show_verbose_status(
                render.load.ui_handles.as_ref(),
                format!("management budgets render failed; error={err}"),
            );
            let message = if render.load.advanced_features {
                "Could not read budget codes: {error}"
            } else {
                "Could not read budgets: {error}"
            };
            render
                .load
                .budgets_list
                .append(&ui::selectable_wrapped_label(&trf(
                    message,
                    &[("error", err.clone())],
                )));
            render.stage = ManagementFormsRenderStage::Aliases;
            true
        }
    }
}

fn render_alias_forms_batch(render: &mut ManagementFormsRender, remaining: &mut usize) -> bool {
    match &mut render.loaded.aliases {
        Ok(aliases) => {
            while *remaining > 0 {
                let Some(alias) = aliases.pop_front() else {
                    show_verbose_status(
                        render.load.ui_handles.as_ref(),
                        format!(
                            "management aliases render finished; forms={}",
                            render.load.aliases_forms.borrow().len()
                        ),
                    );
                    render.stage = ManagementFormsRenderStage::Done;
                    return true;
                };
                append_alias_form(&render.load.aliases_list, &render.load.aliases_forms, alias);
                *remaining -= 1;
            }
            false
        }
        Err(err) => {
            show_verbose_status(
                render.load.ui_handles.as_ref(),
                format!("management aliases render failed; error={err}"),
            );
            render
                .load
                .aliases_list
                .append(&ui::selectable_wrapped_label(&trf(
                    "Could not read field names: {error}",
                    &[("error", err.clone())],
                )));
            render.stage = ManagementFormsRenderStage::Done;
            true
        }
    }
}

fn finish_management_forms_render(load: &ManagementFormsLoad) {
    show_verbose_status(
        load.ui_handles.as_ref(),
        format!(
            "management forms render finished; rules={}; budgets={}; aliases={}",
            load.rules_forms.borrow().len(),
            load.budgets_forms.borrow().len(),
            load.aliases_forms.borrow().len()
        ),
    );
    apply_management_filter(
        &load.filter_entry.text(),
        &load.rules_forms.borrow(),
        &load.budgets_forms.borrow(),
        &load.aliases_forms.borrow(),
        &load.status,
    );
    set_management_form_action_buttons_sensitive(&load.buttons, true);
    for button in &load.buttons {
        register_loading_sensitive_widget(&load.ui_handles, button);
    }
    load.page_actions_button.set_sensitive(true);
    register_loading_sensitive_widget(&load.ui_handles, &load.page_actions_button);
    load.status_handle.set_loading(false);
}

fn set_management_form_action_buttons_sensitive(buttons: &[gtk::Button], sensitive: bool) {
    for button in buttons {
        button.set_sensitive(sensitive);
    }
}

fn management_dialog_content_size(window: &adw::ApplicationWindow) -> (i32, i32) {
    management_dialog_content_dimensions(
        effective_parent_dimension(window.width(), window.default_width()),
        effective_parent_dimension(window.height(), window.default_height()),
    )
}

fn effective_parent_dimension(current: i32, default: i32) -> i32 {
    if current > 0 {
        current
    } else {
        default
    }
}

fn management_dialog_content_dimensions(parent_width: i32, parent_height: i32) -> (i32, i32) {
    (
        management_dialog_content_dimension(
            parent_width,
            MANAGEMENT_DIALOG_MIN_WIDTH,
            MANAGEMENT_DIALOG_FALLBACK_WIDTH,
        ),
        management_dialog_content_dimension(
            parent_height,
            MANAGEMENT_DIALOG_MIN_HEIGHT,
            MANAGEMENT_DIALOG_FALLBACK_HEIGHT,
        ),
    )
}

fn management_dialog_content_dimension(parent: i32, minimum: i32, fallback: i32) -> i32 {
    if parent > 0 {
        (parent - MANAGEMENT_DIALOG_PARENT_INSET).max(minimum)
    } else {
        fallback
    }
}

fn partition_planned_income_budget(
    budgets: Vec<EditableBudget>,
) -> (Option<EditableBudget>, Vec<EditableBudget>) {
    let mut planned_income_budget = None;
    let mut regular_budgets = Vec::new();
    for budget in budgets {
        if planned_income::is_budget_code(&budget.code) {
            planned_income_budget.get_or_insert(budget);
        } else {
            regular_budgets.push(budget);
        }
    }
    (planned_income_budget, regular_budgets)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget(code: &str) -> EditableBudget {
        EditableBudget {
            code: code.to_string(),
            category: code.to_string(),
            monthly_budget: "0".to_string(),
            yearly_budget: String::new(),
            direction: "expense".to_string(),
            income_basis: "real".to_string(),
            notes: String::new(),
        }
    }

    #[test]
    fn planned_income_budget_is_only_split_when_configured() {
        let (planned, regular) = partition_planned_income_budget(vec![budget("FOOD")]);

        assert!(planned.is_none());
        assert_eq!(regular.len(), 1);
        assert_eq!(regular[0].code, "FOOD");
    }

    #[test]
    fn configured_inc_budget_is_split_into_special_form_source() {
        let (planned, regular) =
            partition_planned_income_budget(vec![budget("FOOD"), budget("inc")]);

        assert_eq!(planned.map(|budget| budget.code), Some("inc".to_string()));
        assert_eq!(regular.len(), 1);
        assert_eq!(regular[0].code, "FOOD");
    }

    #[test]
    fn management_dialog_size_tracks_parent_with_inset() {
        assert_eq!(management_dialog_content_dimensions(1250, 900), (1202, 852));
    }

    #[test]
    fn management_dialog_size_uses_minimum_for_small_parent() {
        assert_eq!(management_dialog_content_dimensions(350, 380), (320, 360));
    }

    #[test]
    fn management_dialog_size_uses_fallback_before_parent_is_allocated() {
        assert_eq!(
            management_dialog_content_dimensions(0, 0),
            (
                MANAGEMENT_DIALOG_FALLBACK_WIDTH,
                MANAGEMENT_DIALOG_FALLBACK_HEIGHT
            )
        );
    }
}
