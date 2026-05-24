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
    let planned_income = budget_code_is_planned_income(&code);
    EditableBudget {
        code,
        category: ui::combo_text(&form.category),
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
mod tests {
    use super::*;

    fn rule(code: &str) -> EditableRule {
        EditableRule {
            priority: 0,
            active: true,
            field: "any".to_string(),
            search: "test".to_string(),
            is_regex: false,
            category: "Category".to_string(),
            budget_code: code.to_string(),
            direction: "expense".to_string(),
            amount_min: String::new(),
            amount_max: String::new(),
            notes: String::new(),
        }
    }

    #[test]
    fn planned_income_budget_code_is_reserved() {
        assert!(budget_code_is_planned_income("inc"));
        assert!(budget_code_is_planned_income(" INC "));
        assert!(!budget_code_is_planned_income("INC-OTHER"));
    }

    #[test]
    fn planned_income_budget_amounts_save_as_fixed_values() {
        assert_eq!(budget_amount_text_for_save("10% of income", true), "10");
        assert_eq!(budget_amount_text_for_save("20000", true), "20000");
        assert_eq!(
            budget_amount_text_for_save("10% of income", false),
            "10% of income"
        );
    }

    #[test]
    fn budget_code_renames_update_rule_codes_case_insensitively() {
        let renames = vec![BudgetCodeRename {
            from: "FOOD".to_string(),
            to: "GROCERY".to_string(),
        }];
        let mut rules = vec![rule("food"), rule("RENT"), rule("")];

        let updated = apply_budget_code_renames_to_rules(&mut rules, &renames);

        assert_eq!(updated, 1);
        assert_eq!(rules[0].budget_code, "GROCERY");
        assert_eq!(rules[1].budget_code, "RENT");
        assert_eq!(rules[2].budget_code, "");
    }

    #[test]
    fn budget_code_renames_apply_direct_mapping_without_chaining() {
        let renames = vec![
            BudgetCodeRename {
                from: "A".to_string(),
                to: "B".to_string(),
            },
            BudgetCodeRename {
                from: "B".to_string(),
                to: "C".to_string(),
            },
        ];
        let mut rules = vec![rule("A"), rule("B")];

        let updated = apply_budget_code_renames_to_rules(&mut rules, &renames);

        assert_eq!(updated, 2);
        assert_eq!(rules[0].budget_code, "B");
        assert_eq!(rules[1].budget_code, "C");
    }
}
