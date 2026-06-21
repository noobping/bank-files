use super::codes::{budget_code_key, renamed_budget_code, set_text_combo};
use super::*;

pub(in crate::app) fn collect_rule_forms(forms: &[RuleForm]) -> Vec<EditableRule> {
    forms
        .iter()
        .filter(|form| !form.deleted.get())
        .map(|form| EditableRule {
            priority: form.priority.value_as_int(),
            active: form.active.is_active(),
            field: ui::combo_active_id(&form.field),
            search: rule_search_text(&form.search),
            is_regex: form.is_regex.is_active(),
            category: ui::combo_text(&form.category),
            budget_code: ui::combo_text(&form.budget_code),
            direction: ui::combo_active_id(&form.direction),
            amount_min: form.amount_min.text().trim().to_string(),
            amount_max: form.amount_max.text().trim().to_string(),
            notes: form.notes.text().trim().to_string(),
        })
        .collect()
}

pub(in crate::app) fn collect_budget_forms(forms: &[BudgetForm]) -> Vec<EditableBudget> {
    let mut reserved = Vec::new();
    forms
        .iter()
        .filter(|form| !form.deleted.get())
        .map(|form| {
            let mut budget = editable_budget_from_form(form);
            if form.auto_code.get() || budget.code.trim().is_empty() {
                budget.code = data::generated_budget_code_for_category(&budget.category, &reserved);
                set_text_combo(&form.code, &budget.code);
            }
            budget = transfer_budget::normalize_editable_budget(budget);
            budget = refund_budget::normalize_editable_budget(budget);
            let key = budget_code_key(&budget.code);
            if !key.is_empty() {
                reserved.push(budget.code.clone());
            }
            budget
        })
        .collect()
}

fn editable_budget_from_form(form: &BudgetForm) -> EditableBudget {
    let code = ui::combo_text(&form.code);
    let special = crate::model::budget_special_kind_for_config(&form.special, &code);
    let planned_income = special.is_planned_income() || budget_code_is_planned_income(&code);
    EditableBudget {
        code,
        special: special.as_config().to_string(),
        category: ui::combo_text(&form.category),
        parent_code: ui::combo_active_id(&form.parent_code),
        monthly_budget: budget_amount_text_for_save(&form.monthly_budget.text(), planned_income),
        yearly_budget: budget_amount_text_for_save(&form.yearly_budget.text(), planned_income),
        direction: if planned_income {
            "income".to_string()
        } else {
            ui::combo_active_id(&form.direction)
        },
        income_basis: if planned_income {
            "real".to_string()
        } else {
            ui::combo_active_id(&form.income_basis)
        },
        notes: form.notes.text().trim().to_string(),
    }
}

fn budget_code_is_planned_income(code: &str) -> bool {
    planned_income::is_budget_code(code)
}

fn budget_amount_text_for_save(input: &str, fixed_only: bool) -> String {
    if fixed_only {
        planned_income::fixed_budget_amount_text(input)
    } else {
        input.trim().to_string()
    }
}

pub(in crate::app) fn collect_budget_code_renames(forms: &[BudgetForm]) -> Vec<BudgetCodeRename> {
    forms
        .iter()
        .filter(|form| !form.deleted.get())
        .filter_map(|form| {
            let from = form.original_code.borrow().trim().to_string();
            let to = ui::combo_text(&form.code).trim().to_string();
            if from.is_empty() || to.is_empty() || from == to {
                None
            } else {
                Some(BudgetCodeRename { from, to })
            }
        })
        .collect()
}

pub(in crate::app) fn collect_rule_direction_changes(
    forms: &[RuleForm],
) -> Vec<BudgetDirectionChange> {
    forms
        .iter()
        .filter(|form| !form.deleted.get())
        .filter_map(|form| {
            let from = form.original_direction.borrow().as_ref().copied()?;
            let to = BudgetDirection::from_config(&ui::combo_active_id(&form.direction))?;
            budget_direction_change(&rule_direction_change_label(form), from, to)
        })
        .collect()
}

pub(in crate::app) fn collect_budget_direction_changes(
    forms: &[BudgetForm],
) -> Vec<BudgetDirectionChange> {
    forms
        .iter()
        .filter(|form| !form.deleted.get())
        .filter_map(|form| {
            let from = form.original_direction.borrow().as_ref().copied()?;
            let to = budget_form_direction(form);
            budget_direction_change(&ui::combo_text(&form.code), from, to)
        })
        .collect()
}

fn budget_form_direction(form: &BudgetForm) -> BudgetDirection {
    BudgetDirection::parse(
        &ui::combo_active_id(&form.direction),
        &ui::combo_text(&form.code),
        &ui::combo_text(&form.category),
    )
}

fn rule_direction_change_label(form: &RuleForm) -> String {
    let budget_code = ui::combo_text(&form.budget_code);
    if !budget_code.trim().is_empty() {
        return budget_code;
    }
    rule_search_text(&form.search)
}

pub(in crate::app) fn apply_budget_code_renames_to_rule_forms(
    forms: &[RuleForm],
    renames: &[BudgetCodeRename],
) -> usize {
    let mut updated = 0;
    for form in forms.iter().filter(|form| !form.deleted.get()) {
        let current = ui::combo_text(&form.budget_code);
        let Some(renamed) = renamed_budget_code(&current, renames) else {
            continue;
        };
        if current.trim() == renamed.trim() {
            continue;
        }
        set_text_combo(&form.budget_code, renamed);
        updated += 1;
    }
    updated
}

pub(in crate::app) fn apply_budget_code_renames_to_budget_parent_forms(
    forms: &[BudgetForm],
    renames: &[BudgetCodeRename],
) -> usize {
    let deleted_codes = forms
        .iter()
        .filter(|form| form.deleted.get())
        .flat_map(|form| {
            [
                form.original_code.borrow().trim().to_string(),
                ui::combo_text(&form.code).trim().to_string(),
            ]
        })
        .filter(|code| !code.is_empty())
        .collect::<Vec<_>>();

    let mut updated = 0;
    for form in forms.iter().filter(|form| !form.deleted.get()) {
        let current = ui::combo_active_id(&form.parent_code);
        if current.trim().is_empty() {
            continue;
        }

        if deleted_codes
            .iter()
            .any(|deleted| deleted.trim().eq_ignore_ascii_case(current.trim()))
        {
            set_parent_combo(&form.parent_code, "");
            updated += 1;
            continue;
        }

        let Some(renamed) = renamed_budget_code(&current, renames) else {
            continue;
        };
        if current.trim() == renamed.trim() {
            continue;
        }
        set_parent_combo(&form.parent_code, renamed);
        updated += 1;
    }
    updated
}

fn set_parent_combo(combo: &gtk::ComboBoxText, value: &str) {
    let value = value.trim();
    if value.is_empty() {
        combo.set_active(Some(0));
        return;
    }

    combo.set_active_id(Some(value));
    if combo.active_id().is_none() {
        combo.append(Some(value), value);
        combo.set_active_id(Some(value));
    }
}

pub(in crate::app) fn mark_rule_forms_saved(forms: &[RuleForm]) {
    for form in forms.iter().filter(|form| !form.deleted.get()) {
        *form.original_direction.borrow_mut() =
            BudgetDirection::from_config(&ui::combo_active_id(&form.direction));
    }
}

pub(in crate::app) fn mark_budget_forms_saved(forms: &[BudgetForm]) {
    for form in forms.iter().filter(|form| !form.deleted.get()) {
        *form.original_code.borrow_mut() = ui::combo_text(&form.code).trim().to_string();
        *form.original_direction.borrow_mut() = Some(budget_form_direction(form));
        form.auto_code.set(false);
    }
}

#[cfg(test)]
fn apply_budget_code_renames_to_rules(
    rules: &mut [EditableRule],
    renames: &[BudgetCodeRename],
) -> usize {
    let mut updated = 0;
    for rule in rules {
        let Some(renamed) = renamed_budget_code(&rule.budget_code, renames) else {
            continue;
        };
        if rule.budget_code.trim() == renamed.trim() {
            continue;
        }
        rule.budget_code = renamed.to_string();
        updated += 1;
    }
    updated
}

pub(in crate::app) fn collect_alias_forms(forms: &[AliasForm]) -> Vec<EditableAlias> {
    forms
        .iter()
        .filter(|form| !form.deleted.get())
        .map(|form| EditableAlias {
            canonical: ui::combo_active_id(&form.canonical),
            alias: form.alias.text().trim().to_string(),
        })
        .collect()
}

#[cfg(test)]
#[path = "collect_tests.rs"]
mod tests;
