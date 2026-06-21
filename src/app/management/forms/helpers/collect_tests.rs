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
fn refund_budget_codes_are_reserved_for_canonical_fields() {
    let budget = refund_budget::normalize_editable_budget(EditableBudget {
        code: " refunded ".to_string(),
        parent_code: String::new(),
        special: String::new(),
        category: "Returned".to_string(),
        monthly_budget: "10%".to_string(),
        yearly_budget: String::new(),
        direction: "expense".to_string(),
        income_basis: "planned".to_string(),
        notes: String::new(),
    });

    assert_eq!(budget.code, crate::model::REFUNDED_BUDGET_CODE);
    assert_eq!(budget.direction, "income");
    assert_eq!(budget.income_basis, "real");
}

#[test]
fn transfer_budget_code_is_reserved_for_canonical_fields() {
    let budget = transfer_budget::normalize_editable_budget(EditableBudget {
        code: " transfer ".to_string(),
        parent_code: String::new(),
        special: String::new(),
        category: "Internal".to_string(),
        monthly_budget: "10%".to_string(),
        yearly_budget: String::new(),
        direction: "expense".to_string(),
        income_basis: "planned".to_string(),
        notes: String::new(),
    });

    assert_eq!(budget.code, transfer_budget::BUDGET_CODE);
    assert_eq!(budget.direction, "transfer");
    assert_eq!(budget.income_basis, "real");
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
