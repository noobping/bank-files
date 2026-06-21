use super::super::*;
use super::render::start_management_forms_render;

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
    pub(super) action_widgets: Vec<gtk::Widget>,
    pub(super) menu_actions: Vec<gtk::gio::SimpleAction>,
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

fn budget_forms_queue(budgets: Vec<EditableBudget>) -> std::collections::VecDeque<EditableBudget> {
    std::collections::VecDeque::from(budgets)
}

fn alias_forms_queue(mut aliases: Vec<EditableAlias>) -> std::collections::VecDeque<EditableAlias> {
    aliases.sort_by(|left, right| {
        alias_sort_text(left)
            .cmp(&alias_sort_text(right))
            .then_with(|| left.canonical.cmp(&right.canonical))
            .then_with(|| left.alias.cmp(&right.alias))
    });
    std::collections::VecDeque::from(aliases)
}

fn alias_sort_text(alias: &EditableAlias) -> String {
    crate::util::normalize_key(&alias.alias)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum ManagementFormsRenderStage {
    Rules,
    Budgets,
    Aliases,
    Done,
}

pub(super) fn schedule_management_forms_load(load: ManagementFormsLoad) {
    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(load_management_forms_data);
        match task.await {
            Ok(loaded) => {
                if load.dialog_closed.get() {
                    return;
                }
                start_management_forms_render(load, loaded);
            }
            Err(_) => {
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
            .map(budget_forms_queue)
            .map_err(|err| format!("{err:#}")),
        aliases: data::load_editable_aliases()
            .map(alias_forms_queue)
            .map_err(|err| format!("{err:#}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget(code: &str, special: &str) -> EditableBudget {
        EditableBudget {
            code: code.to_string(),
            parent_code: String::new(),
            special: special.to_string(),
            category: code.to_string(),
            monthly_budget: String::new(),
            yearly_budget: String::new(),
            direction: "expense".to_string(),
            income_basis: "real".to_string(),
            notes: String::new(),
        }
    }

    #[test]
    fn budget_forms_queue_preserves_saved_order() {
        let queue = budget_forms_queue(vec![
            budget("FOOD", ""),
            budget("INC", "planned-income"),
            budget("OTHER", ""),
        ]);
        let codes = queue
            .iter()
            .map(|budget| budget.code.as_str())
            .collect::<Vec<_>>();

        assert_eq!(codes, vec!["FOOD", "INC", "OTHER"]);
    }

    fn alias(canonical: &str, alias: &str) -> EditableAlias {
        EditableAlias {
            canonical: canonical.to_string(),
            alias: alias.to_string(),
        }
    }

    #[test]
    fn alias_forms_queue_sorts_by_bank_column_name() {
        let queue = alias_forms_queue(vec![
            alias("amount", "Bedrag"),
            alias("date", "Datum"),
            alias("description", "af bij"),
        ]);
        let aliases = queue
            .iter()
            .map(|alias| alias.alias.as_str())
            .collect::<Vec<_>>();

        assert_eq!(aliases, vec!["af bij", "Bedrag", "Datum"]);
    }
}
