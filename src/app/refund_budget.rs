use super::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::app) enum RefundBudgetKind {
    Refunding,
    Refunded,
}

impl RefundBudgetKind {
    pub(in crate::app) fn for_amount(amount: Decimal) -> Self {
        if amount > Decimal::ZERO {
            Self::Refunded
        } else {
            Self::Refunding
        }
    }

    pub(in crate::app) fn for_code(code: &str) -> Option<Self> {
        if crate::model::is_refunding_budget_code(code) {
            Some(Self::Refunding)
        } else if crate::model::is_refunded_budget_code(code) {
            Some(Self::Refunded)
        } else {
            None
        }
    }

    pub(in crate::app) fn code(self) -> &'static str {
        match self {
            Self::Refunding => crate::model::REFUNDING_BUDGET_CODE,
            Self::Refunded => crate::model::REFUNDED_BUDGET_CODE,
        }
    }

    pub(in crate::app) fn special(self) -> crate::model::BudgetSpecialKind {
        match self {
            Self::Refunding => crate::model::BudgetSpecialKind::Refunding,
            Self::Refunded => crate::model::BudgetSpecialKind::Refunded,
        }
    }

    pub(in crate::app) fn category(self) -> String {
        match self {
            Self::Refunding => tr("Refunding"),
            Self::Refunded => tr("Refunded"),
        }
    }

    pub(in crate::app) fn direction(self) -> &'static str {
        match self {
            Self::Refunding => "expense",
            Self::Refunded => "income",
        }
    }
}

pub(in crate::app) fn is_budget_code(code: &str) -> bool {
    crate::model::is_refund_budget_code(code)
}

pub(in crate::app) fn editable_budget(kind: RefundBudgetKind, notes: String) -> EditableBudget {
    EditableBudget {
        code: kind.code().to_string(),
        special: kind.special().as_config().to_string(),
        category: kind.category(),
        monthly_budget: "0".to_string(),
        yearly_budget: String::new(),
        direction: kind.direction().to_string(),
        income_basis: "real".to_string(),
        notes: notes.trim().to_string(),
    }
}

pub(in crate::app) fn editable_budget_for_code(
    code: &str,
    notes: String,
) -> Option<EditableBudget> {
    RefundBudgetKind::for_code(code).map(|kind| editable_budget(kind, notes))
}

pub(in crate::app) fn normalize_editable_budget(mut budget: EditableBudget) -> EditableBudget {
    let special = crate::model::budget_special_kind_for_config(&budget.special, &budget.code);
    let kind = match special {
        crate::model::BudgetSpecialKind::Refunding => Some(RefundBudgetKind::Refunding),
        crate::model::BudgetSpecialKind::Refunded => Some(RefundBudgetKind::Refunded),
        _ => RefundBudgetKind::for_code(&budget.code),
    };
    if let Some(kind) = kind {
        if RefundBudgetKind::for_code(&budget.code).is_some() {
            budget.code = kind.code().to_string();
        }
        budget.special = kind.special().as_config().to_string();
        budget.direction = kind.direction().to_string();
        budget.income_basis = "real".to_string();
    }
    budget
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refund_budget_code_is_special_case_insensitively() {
        assert!(is_budget_code(" refunding "));
        assert!(is_budget_code(" refunded "));
        assert!(!is_budget_code("refund"));
    }

    #[test]
    fn refund_budget_kind_follows_transaction_sign() {
        assert_eq!(
            RefundBudgetKind::for_amount(Decimal::new(-10, 0)),
            RefundBudgetKind::Refunding
        );
        assert_eq!(
            RefundBudgetKind::for_amount(Decimal::new(10, 0)),
            RefundBudgetKind::Refunded
        );
    }

    #[test]
    fn refund_budget_is_canonical() {
        let budget = editable_budget(RefundBudgetKind::Refunded, " Created ".to_string());

        assert_eq!(budget.code, crate::model::REFUNDED_BUDGET_CODE);
        assert_eq!(budget.category, tr("Refunded"));
        assert_eq!(budget.direction, "income");
        assert_eq!(budget.income_basis, "real");
        assert_eq!(budget.monthly_budget, "0");
        assert_eq!(budget.notes, "Created");
    }

    #[test]
    fn refund_budget_normalization_keeps_alias_code() {
        let budget = normalize_editable_budget(EditableBudget {
            code: "REFUND_OUT".to_string(),
            special: crate::model::BudgetSpecialKind::Refunding
                .as_config()
                .to_string(),
            category: "Return".to_string(),
            monthly_budget: "0".to_string(),
            yearly_budget: String::new(),
            direction: "income".to_string(),
            income_basis: "planned".to_string(),
            notes: String::new(),
        });

        assert_eq!(budget.code, "REFUND_OUT");
        assert_eq!(budget.direction, "expense");
        assert_eq!(budget.income_basis, "real");
    }

    #[test]
    fn refund_budget_normalization_forces_canonical_fields() {
        let budget = normalize_editable_budget(EditableBudget {
            code: " refunding ".to_string(),
            special: String::new(),
            category: "Return".to_string(),
            monthly_budget: "10%".to_string(),
            yearly_budget: String::new(),
            direction: "income".to_string(),
            income_basis: "planned".to_string(),
            notes: String::new(),
        });

        assert_eq!(budget.code, crate::model::REFUNDING_BUDGET_CODE);
        assert_eq!(budget.category, "Return");
        assert_eq!(budget.direction, "expense");
        assert_eq!(budget.income_basis, "real");
    }
}
