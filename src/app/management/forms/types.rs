use super::*;

pub(in crate::app) struct RuleForm {
    pub(in crate::app) form_box: gtk::Box,
    pub(in crate::app) deleted: Rc<Cell<bool>>,
    pub(in crate::app) active: gtk::Switch,
    pub(in crate::app) priority: gtk::SpinButton,
    pub(in crate::app) field: gtk::ComboBoxText,
    pub(in crate::app) search: gtk::TextView,
    pub(in crate::app) is_regex: gtk::Switch,
    pub(in crate::app) category: gtk::ComboBoxText,
    pub(in crate::app) budget_code: gtk::ComboBoxText,
    pub(in crate::app) direction: gtk::ComboBoxText,
    pub(in crate::app) amount_min: gtk::Entry,
    pub(in crate::app) amount_max: gtk::Entry,
    pub(in crate::app) notes: gtk::Entry,
    pub(in crate::app) original_direction: Rc<RefCell<Option<BudgetDirection>>>,
}

pub(in crate::app) struct BudgetForm {
    pub(in crate::app) form_box: gtk::Box,
    pub(in crate::app) deleted: Rc<Cell<bool>>,
    pub(in crate::app) original_code: Rc<RefCell<String>>,
    pub(in crate::app) original_direction: Rc<RefCell<Option<BudgetDirection>>>,
    pub(in crate::app) auto_code: Rc<Cell<bool>>,
    pub(in crate::app) code: gtk::ComboBoxText,
    pub(in crate::app) category: gtk::ComboBoxText,
    pub(in crate::app) monthly_budget: gtk::Entry,
    pub(in crate::app) yearly_budget: gtk::Entry,
    pub(in crate::app) direction: gtk::ComboBoxText,
    pub(in crate::app) income_basis: gtk::ComboBoxText,
    pub(in crate::app) notes: gtk::Entry,
    pub(in crate::app) delete_button: gtk::Button,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(in crate::app) struct BudgetCodeRename {
    pub(in crate::app) from: String,
    pub(in crate::app) to: String,
}

pub(in crate::app) struct AliasForm {
    pub(in crate::app) form_box: gtk::Box,
    pub(in crate::app) deleted: Rc<Cell<bool>>,
    pub(in crate::app) canonical: gtk::ComboBoxText,
    pub(in crate::app) alias: gtk::Entry,
}

pub(in crate::app) fn apply_management_filter(
    query: &str,
    rules: &[RuleForm],
    budgets: &[BudgetForm],
    aliases: &[AliasForm],
    status: &gtk::Label,
) {
    let rules_count = filter_rule_forms(query, rules);
    let budgets_count = filter_budget_forms(query, budgets);
    let aliases_count = filter_alias_forms(query, aliases);
    if query.trim().is_empty() {
        status.set_text(&tr("Changes are only applied after Save."));
    } else {
        status.set_text(&trf(
            "Filter active: {rules_count} rule(s), {budgets_count} budget(s), {aliases_count} field name(s) visible.",
            &[
                ("rules_count", rules_count.to_string()),
                ("budgets_count", budgets_count.to_string()),
                ("aliases_count", aliases_count.to_string()),
            ],
        ));
    }
}

pub(in crate::app) fn filter_rule_forms(query: &str, forms: &[RuleForm]) -> usize {
    let active_only = query.trim().eq_ignore_ascii_case("active");
    let filter = SearchFilter::from_text(query);
    let mut visible_count = 0;
    for form in forms {
        let visible = !form.deleted.get()
            && if active_only {
                form.active.is_active()
            } else {
                filter
                    .as_ref()
                    .map(|filter| rule_form_matches(form, filter))
                    .unwrap_or(true)
            };
        form.form_box.set_visible(visible);
        if visible {
            visible_count += 1;
        }
    }
    visible_count
}

pub(in crate::app) fn filter_budget_forms(query: &str, forms: &[BudgetForm]) -> usize {
    let filter = SearchFilter::from_text(query);
    let mut visible_count = 0;
    for form in forms {
        let visible = filter
            .as_ref()
            .map(|filter| budget_form_matches(form, filter))
            .unwrap_or(true);
        form.form_box.set_visible(visible);
        if visible {
            visible_count += 1;
        }
    }
    visible_count
}

pub(in crate::app) fn filter_alias_forms(query: &str, forms: &[AliasForm]) -> usize {
    let filter = SearchFilter::from_text(query);
    let mut visible_count = 0;
    for form in forms {
        let visible = !form.deleted.get()
            && filter
                .as_ref()
                .map(|filter| alias_form_matches(form, filter))
                .unwrap_or(true);
        form.form_box.set_visible(visible);
        if visible {
            visible_count += 1;
        }
    }
    visible_count
}

pub(in crate::app) fn rule_form_matches(form: &RuleForm, filter: &SearchFilter) -> bool {
    filter.matches(&format!(
        "{} {} {} {} {} {} {} {} {} {} {} {}",
        if form.active.is_active() {
            "active"
        } else {
            "inactive"
        },
        form.priority.value_as_int(),
        combo_active_id(&form.field),
        rule_search_text(&form.search),
        if form.is_regex.is_active() {
            "regex"
        } else {
            "text"
        },
        ui::combo_text(&form.category),
        ui::combo_text(&form.budget_code),
        combo_active_id(&form.direction),
        form.amount_min.text(),
        form.amount_max.text(),
        form.notes.text(),
        if form.deleted.get() { "deleted" } else { "" },
    ))
}

pub(in crate::app) fn budget_form_matches(form: &BudgetForm, filter: &SearchFilter) -> bool {
    filter.matches(&format!(
        "{} {} {} {} {} {} {} {}",
        ui::combo_text(&form.code),
        ui::combo_text(&form.category),
        form.monthly_budget.text(),
        form.yearly_budget.text(),
        combo_active_id(&form.direction),
        combo_active_id(&form.income_basis),
        form.notes.text(),
        if form.deleted.get() { "deleted" } else { "" },
    ))
}

pub(in crate::app) fn alias_form_matches(form: &AliasForm, filter: &SearchFilter) -> bool {
    filter.matches(&format!(
        "{} {}",
        combo_active_id(&form.canonical),
        form.alias.text(),
    ))
}
