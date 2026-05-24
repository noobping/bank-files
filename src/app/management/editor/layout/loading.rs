use super::super::*;
use super::render::start_management_forms_render;
use super::sizing::partition_planned_income_budget;

pub(super) struct ManagementFormsLoad {
    pub(super) advanced_features: bool,
    pub(super) rules_list: gtk::Box,
    pub(super) rules_forms: Rc<RefCell<Vec<RuleForm>>>,
    pub(super) budgets_list: gtk::Box,
    pub(super) budgets_forms: Rc<RefCell<Vec<BudgetForm>>>,
    pub(super) aliases_list: gtk::Box,
    pub(super) aliases_forms: Rc<RefCell<Vec<AliasForm>>>,
    pub(super) filter_entry: gtk::SearchEntry,
    pub(super) status: gtk::Label,
    pub(super) dialog_closed: Rc<Cell<bool>>,
    pub(super) advanced_autofill: Rc<Cell<bool>>,
    pub(super) ui_handles: Rc<UiHandles>,
    pub(super) buttons: Vec<gtk::Button>,
    pub(super) page_actions_button: gtk::MenuButton,
    pub(super) status_handle: StatusHandle,
}

pub(super) struct ManagementLoadedForms {
    pub(super) rules: Result<std::collections::VecDeque<EditableRule>, String>,
    pub(super) budgets: Result<std::collections::VecDeque<EditableBudget>, String>,
    pub(super) aliases: Result<std::collections::VecDeque<EditableAlias>, String>,
}

pub(super) struct ManagementFormsRender {
    pub(super) load: ManagementFormsLoad,
    pub(super) loaded: ManagementLoadedForms,
    pub(super) stage: ManagementFormsRenderStage,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum ManagementFormsRenderStage {
    Rules,
    Budgets,
    Aliases,
    Done,
}

pub(super) fn append_management_loading(container: &gtk::Box, message: &str) {
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

pub(super) fn schedule_management_forms_load(load: ManagementFormsLoad) {
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

pub(super) fn management_loaded_forms_summary(loaded: &ManagementLoadedForms) -> String {
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

pub(super) fn ordered_management_budgets(budgets: Vec<EditableBudget>) -> Vec<EditableBudget> {
    let (planned_income_budget, mut regular_budgets) = partition_planned_income_budget(budgets);
    let mut ordered =
        Vec::with_capacity(regular_budgets.len() + usize::from(planned_income_budget.is_some()));
    if let Some(planned_income_budget) = planned_income_budget {
        ordered.push(planned_income_budget);
    }
    ordered.append(&mut regular_budgets);
    ordered
}
