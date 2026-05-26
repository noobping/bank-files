use super::*;

pub(in crate::app) const BUDGET_CODE: &str = crate::model::TRANSFER_BUDGET_CODE;

pub(in crate::app) fn is_budget_code(code: &str) -> bool {
    crate::model::is_transfer_budget_code(code)
}

pub(in crate::app) fn editable_budget(notes: String) -> EditableBudget {
    EditableBudget {
        code: BUDGET_CODE.to_string(),
        special: crate::model::BudgetSpecialKind::Transfer
            .as_config()
            .to_string(),
        category: tr("Transfers"),
        monthly_budget: "0".to_string(),
        yearly_budget: String::new(),
        direction: "transfer".to_string(),
        income_basis: "real".to_string(),
        notes: notes.trim().to_string(),
    }
}

pub(in crate::app) fn normalize_editable_budget(mut budget: EditableBudget) -> EditableBudget {
    let special = crate::model::budget_special_kind_for_config(&budget.special, &budget.code);
    if special.is_transfer() || is_budget_code(&budget.code) {
        if is_budget_code(&budget.code) {
            budget.code = BUDGET_CODE.to_string();
        }
        budget.special = crate::model::BudgetSpecialKind::Transfer
            .as_config()
            .to_string();
        budget.direction = "transfer".to_string();
        budget.income_basis = "real".to_string();
    }
    budget
}

pub(in crate::app) fn code_for_new_budget(
    category: &str,
    direction: &str,
    existing_codes: &[String],
) -> String {
    if BudgetDirection::parse(direction, "", category).is_transfer()
        && !existing_codes.iter().any(|code| is_budget_code(code))
    {
        BUDGET_CODE.to_string()
    } else {
        data::generated_budget_code_for_category(category, existing_codes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_budget_code_is_special_case_insensitively() {
        assert!(is_budget_code(" transfer "));
        assert!(!is_budget_code("TRANSFER-OTHER"));
    }

    #[test]
    fn transfer_budget_is_canonical() {
        let budget = editable_budget(" Created ".to_string());

        assert_eq!(budget.code, BUDGET_CODE);
        assert_eq!(budget.direction, "transfer");
        assert_eq!(budget.income_basis, "real");
        assert_eq!(budget.monthly_budget, "0");
        assert_eq!(budget.yearly_budget, "");
        assert_eq!(budget.notes, "Created");
    }

    #[test]
    fn transfer_budget_normalization_keeps_user_text_but_forces_canonical_fields() {
        let budget = normalize_editable_budget(EditableBudget {
            code: " transfer ".to_string(),
            special: crate::model::BudgetSpecialKind::Transfer
                .as_config()
                .to_string(),
            category: "Internal".to_string(),
            monthly_budget: "10%".to_string(),
            yearly_budget: String::new(),
            direction: "expense".to_string(),
            income_basis: "planned".to_string(),
            notes: "Keep".to_string(),
        });

        assert_eq!(budget.code, BUDGET_CODE);
        assert_eq!(budget.category, "Internal");
        assert_eq!(budget.monthly_budget, "10%");
        assert_eq!(budget.direction, "transfer");
        assert_eq!(budget.income_basis, "real");
    }

    #[test]
    fn transfer_budget_normalization_keeps_alias_code() {
        let budget = normalize_editable_budget(EditableBudget {
            code: "INTERNAL".to_string(),
            special: crate::model::BudgetSpecialKind::Transfer
                .as_config()
                .to_string(),
            category: "Internal".to_string(),
            monthly_budget: "0".to_string(),
            yearly_budget: String::new(),
            direction: "expense".to_string(),
            income_basis: "planned".to_string(),
            notes: String::new(),
        });

        assert_eq!(budget.code, "INTERNAL");
        assert_eq!(budget.direction, "transfer");
        assert_eq!(budget.income_basis, "real");
    }

    #[test]
    fn new_transfer_budget_uses_special_code_until_it_exists() {
        assert_eq!(
            code_for_new_budget("Internal", "transfer", &[]),
            BUDGET_CODE
        );
        assert_eq!(
            code_for_new_budget("Internal", "transfer", &[BUDGET_CODE.to_string()]),
            "INTERNAL"
        );
    }
}
