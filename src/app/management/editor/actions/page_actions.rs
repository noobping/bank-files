use super::*;

pub(super) fn connect_management_page_actions(
    page_actions_button: &gtk::MenuButton,
    stack: &adw::ViewStack,
    rules_forms: &Rc<RefCell<Vec<RuleForm>>>,
    budgets_forms: &Rc<RefCell<Vec<BudgetForm>>>,
    aliases_forms: &Rc<RefCell<Vec<AliasForm>>>,
    status: &gtk::Label,
    ui_handles: &Rc<UiHandles>,
) {
    let stack_for_snapshot = stack.clone();
    let rules_forms_for_snapshot = Rc::clone(rules_forms);
    let budgets_forms_for_snapshot = Rc::clone(budgets_forms);
    let aliases_forms_for_snapshot = Rc::clone(aliases_forms);
    connect_page_actions(
        page_actions_button,
        "management",
        status,
        ui_handles,
        move || {
            current_management_page_snapshot(
                &stack_for_snapshot,
                &rules_forms_for_snapshot.borrow(),
                &budgets_forms_for_snapshot.borrow(),
                &aliases_forms_for_snapshot.borrow(),
            )
        },
    );
}

fn current_management_page_snapshot(
    stack: &adw::ViewStack,
    rules_forms: &[RuleForm],
    budgets_forms: &[BudgetForm],
    aliases_forms: &[AliasForm],
) -> anyhow::Result<PageActionSnapshot> {
    match stack.visible_child_name().as_deref() {
        Some("rules") => rules_management_snapshot(rules_forms),
        Some("aliases") => aliases_management_snapshot(aliases_forms),
        _ => budgets_management_snapshot(budgets_forms),
    }
}

fn rules_management_snapshot(forms: &[RuleForm]) -> anyhow::Result<PageActionSnapshot> {
    let rules = visible_collected_rules(forms);
    let columns = strings(&[
        "Active",
        "Priority",
        "Field",
        "Search",
        "Regex",
        "Category",
        "Budget Code",
        "Direction",
        "Min",
        "Max",
        "Notes",
    ]);
    let rows = rules
        .iter()
        .map(|rule| {
            vec![
                rule.active.to_string(),
                rule.priority.to_string(),
                rule.field.clone(),
                rule.search.clone(),
                rule.is_regex.to_string(),
                rule.category.clone(),
                rule.budget_code.clone(),
                rule.direction.clone(),
                rule.amount_min.clone(),
                rule.amount_max.clone(),
                rule.notes.clone(),
            ]
        })
        .collect::<Vec<_>>();
    management_page_snapshot(
        "management_rules",
        "Rules",
        "Categorization rules visible in the management window.",
        columns,
        rows,
        data::editable_rules_to_csv(&rules)?,
    )
}

fn budgets_management_snapshot(forms: &[BudgetForm]) -> anyhow::Result<PageActionSnapshot> {
    let budgets = visible_collected_budgets(forms);
    let columns = strings(&[
        "Code",
        "Category",
        "Monthly",
        "Yearly",
        "Direction",
        "Income basis",
        "Notes",
    ]);
    let rows = budgets
        .iter()
        .map(|budget| {
            vec![
                budget.code.clone(),
                budget.category.clone(),
                budget.monthly_budget.clone(),
                budget.yearly_budget.clone(),
                budget.direction.clone(),
                budget.income_basis.clone(),
                budget.notes.clone(),
            ]
        })
        .collect::<Vec<_>>();
    management_page_snapshot(
        "management_budgets",
        "Budgets",
        "Budgets visible in the management window.",
        columns,
        rows,
        data::editable_budgets_to_csv(&budgets)?,
    )
}

fn aliases_management_snapshot(forms: &[AliasForm]) -> anyhow::Result<PageActionSnapshot> {
    let aliases = visible_collected_aliases(forms);
    let columns = strings(&["Canonical", "Alias"]);
    let rows = aliases
        .iter()
        .map(|alias| vec![alias.canonical.clone(), alias.alias.clone()])
        .collect::<Vec<_>>();
    management_page_snapshot(
        "management_aliases",
        "Normalize",
        "Field names visible in the management window.",
        columns,
        rows,
        data::editable_aliases_to_csv(&aliases)?,
    )
}

fn management_page_snapshot(
    key: &str,
    title: &str,
    subtitle: &str,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    csv: String,
) -> anyhow::Result<PageActionSnapshot> {
    Ok(PageActionSnapshot::from_csv(
        key, title, subtitle, columns, rows, csv,
    ))
}

fn visible_collected_rules(forms: &[RuleForm]) -> Vec<EditableRule> {
    let rules = collect_rule_forms(forms);
    forms
        .iter()
        .filter(|form| !form.deleted.get())
        .zip(rules)
        .filter(|(form, _)| form.form_box.is_visible())
        .map(|(_, rule)| rule)
        .collect()
}

fn visible_collected_budgets(forms: &[BudgetForm]) -> Vec<EditableBudget> {
    let budgets = collect_budget_forms(forms);
    forms
        .iter()
        .filter(|form| !form.deleted.get())
        .zip(budgets)
        .filter(|(form, _)| form.form_box.is_visible())
        .map(|(_, budget)| budget)
        .collect()
}

fn visible_collected_aliases(forms: &[AliasForm]) -> Vec<EditableAlias> {
    let aliases = collect_alias_forms(forms);
    forms
        .iter()
        .filter(|form| !form.deleted.get())
        .zip(aliases)
        .filter(|(form, _)| form.form_box.is_visible())
        .map(|(_, alias)| alias)
        .collect()
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}
