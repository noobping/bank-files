use super::super::*;

pub(super) struct RuleValueWidgets<'a> {
    pub(super) active: &'a gtk::Switch,
    pub(super) priority: &'a gtk::SpinButton,
    pub(super) field: &'a gtk::ComboBoxText,
    pub(super) search: &'a gtk::TextView,
    pub(super) is_regex: &'a gtk::Switch,
    pub(super) category: &'a gtk::ComboBoxText,
    pub(super) budget_code: &'a gtk::ComboBoxText,
    pub(super) direction: &'a gtk::ComboBoxText,
    pub(super) amount_min: &'a gtk::Entry,
    pub(super) amount_max: &'a gtk::Entry,
    pub(super) notes: &'a gtk::Entry,
}

pub(super) fn rule_value(widgets: RuleValueWidgets<'_>) -> EditableRule {
    EditableRule {
        priority: widgets.priority.value_as_int(),
        active: widgets.active.is_active(),
        field: combo_active_id(widgets.field),
        search: rule_search_text(widgets.search),
        is_regex: widgets.is_regex.is_active(),
        category: ui::combo_text(widgets.category),
        budget_code: ui::combo_text(widgets.budget_code),
        direction: combo_active_id(widgets.direction),
        amount_min: widgets.amount_min.text().trim().to_string(),
        amount_max: widgets.amount_max.text().trim().to_string(),
        notes: widgets.notes.text().trim().to_string(),
    }
}

pub(super) struct BudgetValueWidgets<'a> {
    pub(super) code: &'a gtk::ComboBoxText,
    pub(super) parent_code: &'a gtk::ComboBoxText,
    pub(super) special: &'a str,
    pub(super) category: &'a gtk::ComboBoxText,
    pub(super) monthly_budget: &'a gtk::Entry,
    pub(super) yearly_budget: &'a gtk::Entry,
    pub(super) direction: &'a gtk::ComboBoxText,
    pub(super) income_basis: &'a gtk::ComboBoxText,
    pub(super) notes: &'a gtk::Entry,
}

pub(super) fn budget_value(widgets: BudgetValueWidgets<'_>) -> EditableBudget {
    let code = ui::combo_text(widgets.code);
    let special = crate::model::budget_special_kind_for_config(widgets.special, &code);
    let planned_income = special.is_planned_income() || planned_income::is_budget_code(&code);
    EditableBudget {
        code,
        parent_code: combo_active_id(widgets.parent_code),
        special: special.as_config().to_string(),
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

pub(super) fn alias_value(canonical: &gtk::ComboBoxText, alias: &gtk::Entry) -> EditableAlias {
    EditableAlias {
        canonical: combo_active_id(canonical),
        alias: alias.text().trim().to_string(),
    }
}
