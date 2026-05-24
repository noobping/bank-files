use super::super::*;
use super::loading::append_management_loading;
use super::sections::{build_budget_action_section, build_rule_action_section};
use super::setup::{finish_management_dialog_setup, ManagementDialogSetup};
use super::sizing::management_dialog_content_size;
use super::*;

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

    let filter_placeholder = if advanced_features {
        "Filter rules, budgets, and field names"
    } else {
        "Filter budgets and field names"
    };
    let ManagementDialogShell {
        root,
        add_button,
        save_button,
        filter_entry,
        filter_search_bar,
        stack,
        switcher,
        switcher_bar,
    } = build_management_dialog_shell(filter_placeholder);

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
    let rule_actions = build_rule_action_section();
    let add_rule_button = rule_actions.add_rule_button.clone();
    let group_rules_action = rule_actions.group_rules_action.clone();
    let combine_rules_action = rule_actions.combine_rules_action.clone();
    let rule_bulk_menu_button = rule_actions.bulk_menu_button.clone();
    rules_box.append(&rule_actions.container);

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
    let budget_actions = build_budget_action_section(advanced_features);
    let add_budget_button = budget_actions.add_budget_button.clone();
    let move_budget_code_action = budget_actions.move_budget_code_action.clone();
    let use_real_income_action = budget_actions.use_real_income_action.clone();
    let use_planned_income_action = budget_actions.use_planned_income_action.clone();
    let use_monthly_values_action = budget_actions.use_monthly_values_action.clone();
    let use_yearly_values_action = budget_actions.use_yearly_values_action.clone();
    let budget_bulk_menu_button = budget_actions.bulk_menu_button.clone();
    budgets_box.append(&budget_actions.container);

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

    let status_bar = build_status_bar();
    connect_embedded_status_bar(window, &status_bar, Rc::clone(&ui_handles.status_autohide));
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
    let management_dialog = ui::content_dialog(tr(management_title), &root)
        .width_request(MANAGEMENT_DIALOG_MIN_WIDTH)
        .height_request(MANAGEMENT_DIALOG_MIN_HEIGHT)
        .content_width(content_width)
        .content_height(content_height)
        .build();
    let dialog_closed_for_closed = Rc::clone(&dialog_closed);
    let save_running_for_closed = Rc::clone(&save_running);
    let finish_for_closed = Rc::clone(&finish_management_dialog);
    let ui_for_closed = Rc::clone(ui_handles);
    management_dialog.connect_closed(move |_| {
        dialog_closed_for_closed.set(true);
        ui_for_closed.management_search.borrow_mut().take();
        show_verbose_status(ui_for_closed.as_ref(), "management dialog closed");
        if !save_running_for_closed.get() {
            finish_for_closed();
        }
    });
    *ui_handles.management_search.borrow_mut() = Some(SearchToggleHandle {
        search_bar: filter_search_bar.clone(),
        search_entry: filter_entry.clone(),
    });
    filter_search_bar.set_key_capture_widget(Some(&management_dialog));
    add_responsive_switcher_for_dialog(&management_dialog, &switcher, &switcher_bar);

    finish_management_dialog_setup(ManagementDialogSetup {
        window,
        management_dialog: &management_dialog,
        add_button: &add_button,
        add_rule_button: &add_rule_button,
        group_rules_action: &group_rules_action,
        combine_rules_action: &combine_rules_action,
        add_budget_button: &add_budget_button,
        move_budget_code_action: &move_budget_code_action,
        use_real_income_action: &use_real_income_action,
        use_planned_income_action: &use_planned_income_action,
        use_monthly_values_action: &use_monthly_values_action,
        use_yearly_values_action: &use_yearly_values_action,
        add_alias_button: &add_alias_button,
        rule_bulk_menu_button: &rule_bulk_menu_button,
        budget_bulk_menu_button: &budget_bulk_menu_button,
        save_button: &save_button,
        page_actions_button: &status_bar.page_actions_button,
        stack: &stack,
        filter_entry: &filter_entry,
        filter_search_bar: &filter_search_bar,
        rules_list,
        rules_forms,
        rules_scroll: &rules_scroll,
        budgets_list,
        budgets_forms,
        budgets_scroll: &budgets_scroll,
        aliases_list,
        aliases_forms,
        aliases_scroll: &aliases_scroll,
        status,
        dialog_closed,
        save_running,
        finish_management_dialog,
        initial_filter,
        advanced_features,
        state,
        ui_handles,
        status_handle,
    });
    true
}

struct ManagementDialogShell {
    root: gtk::Box,
    add_button: gtk::Button,
    save_button: gtk::Button,
    filter_entry: gtk::SearchEntry,
    filter_search_bar: gtk::SearchBar,
    stack: adw::ViewStack,
    switcher: adw::ViewSwitcher,
    switcher_bar: adw::ViewSwitcherBar,
}

fn build_management_dialog_shell(filter_placeholder: &str) -> ManagementDialogShell {
    const RESOURCE: &str = "management-dialog.ui";

    let builder = ui::builder_from_resource(RESOURCE);
    let root = ui::builder_object::<gtk::Box>(&builder, "management_root", RESOURCE);
    let header = ui::builder_object::<adw::HeaderBar>(&builder, "management_header", RESOURCE);
    let switcher =
        ui::builder_object::<adw::ViewSwitcher>(&builder, "management_switcher", RESOURCE);
    let switcher_bar =
        ui::builder_object::<adw::ViewSwitcherBar>(&builder, "management_switcher_bar", RESOURCE);
    let stack = ui::builder_object::<adw::ViewStack>(&builder, "management_stack", RESOURCE);
    let add_button = ui::builder_object::<gtk::Button>(&builder, "management_add_button", RESOURCE);
    let save_button =
        ui::builder_object::<gtk::Button>(&builder, "management_save_button", RESOURCE);
    let save_content =
        ui::builder_object::<adw::ButtonContent>(&builder, "management_save_content", RESOURCE);
    let filter_search_bar =
        ui::builder_object::<gtk::SearchBar>(&builder, "management_filter_search_bar", RESOURCE);
    let filter_entry =
        ui::builder_object::<gtk::SearchEntry>(&builder, "management_filter_entry", RESOURCE);

    switcher.set_stack(Some(&stack));
    switcher_bar.set_stack(Some(&stack));
    header.set_title_widget(Some(&switcher));

    add_button.set_tooltip_text(Some(&tr("New item")));

    save_content.set_label(&tr("Save"));
    save_content.set_icon_name("document-save-symbolic");
    save_button.set_tooltip_text(Some(&tr("Save rules, budgets, and field names")));

    filter_entry.set_placeholder_text(Some(&tr(filter_placeholder)));
    filter_search_bar.connect_entry(&filter_entry);

    ManagementDialogShell {
        root,
        add_button,
        save_button,
        filter_entry,
        filter_search_bar,
        stack,
        switcher,
        switcher_bar,
    }
}
