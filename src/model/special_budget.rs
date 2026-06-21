use crate::util::normalize_key;
use serde::{Deserialize, Serialize};

pub const PLANNED_INCOME_BUDGET_CODE: &str = "INC";
pub const TRANSFER_BUDGET_CODE: &str = "TRANSFER";
pub const REFUNDING_BUDGET_CODE: &str = "REFUNDING";
pub const REFUNDED_BUDGET_CODE: &str = "REFUNDED";

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum BudgetSpecialKind {
    #[default]
    None,
    PlannedIncome,
    Transfer,
    Refunding,
    Refunded,
}

impl BudgetSpecialKind {
    pub fn from_config(input: &str) -> Option<Self> {
        match normalize_key(input).as_str() {
            "" | "none" | "normal" => Some(Self::None),
            "plannedincome" | "planned-income" | "planned income" | "planned" | "inc"
            | "income-plan" | "income plan" => Some(Self::PlannedIncome),
            "transfer" | "transfers" => Some(Self::Transfer),
            "refunding" | "refund-out" | "refund out" | "outgoing-refund" | "outgoing refund" => {
                Some(Self::Refunding)
            }
            "refunded" | "refund-in" | "refund in" | "incoming-refund" | "incoming refund" => {
                Some(Self::Refunded)
            }
            _ => None,
        }
    }

    pub fn from_code(code: &str) -> Self {
        if is_planned_income_budget_code(code) {
            Self::PlannedIncome
        } else if is_transfer_budget_code(code) {
            Self::Transfer
        } else if is_refunding_budget_code(code) {
            Self::Refunding
        } else if is_refunded_budget_code(code) {
            Self::Refunded
        } else {
            Self::None
        }
    }

    pub fn from_config_or_code(input: &str, code: &str) -> Self {
        if input.trim().is_empty() {
            return Self::from_code(code);
        }

        Self::from_config(input)
            .filter(|kind| !matches!(kind, Self::None))
            .unwrap_or_else(|| Self::from_code(code))
    }

    pub fn as_config(self) -> &'static str {
        match self {
            Self::None => "",
            Self::PlannedIncome => "planned-income",
            Self::Transfer => "transfer",
            Self::Refunding => "refunding",
            Self::Refunded => "refunded",
        }
    }

    pub fn canonical_code(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::PlannedIncome => Some(PLANNED_INCOME_BUDGET_CODE),
            Self::Transfer => Some(TRANSFER_BUDGET_CODE),
            Self::Refunding => Some(REFUNDING_BUDGET_CODE),
            Self::Refunded => Some(REFUNDED_BUDGET_CODE),
        }
    }

    pub fn is_planned_income(self) -> bool {
        matches!(self, Self::PlannedIncome)
    }

    pub fn is_transfer(self) -> bool {
        matches!(self, Self::Transfer)
    }

    pub fn is_refund(self) -> bool {
        matches!(self, Self::Refunding | Self::Refunded)
    }

    pub fn is_neutral(self) -> bool {
        self.is_transfer() || self.is_refund()
    }
}

pub fn budget_special_kind_for_config(input: &str, code: &str) -> BudgetSpecialKind {
    BudgetSpecialKind::from_config_or_code(input, code)
}

pub fn budget_special_kind_is_valid_config(input: &str) -> bool {
    BudgetSpecialKind::from_config(input).is_some()
}

pub fn is_planned_income_budget_code(code: &str) -> bool {
    code_eq(code, PLANNED_INCOME_BUDGET_CODE)
}

pub fn is_transfer_budget_code(code: &str) -> bool {
    code_eq(code, TRANSFER_BUDGET_CODE)
}

pub fn is_refunding_budget_code(code: &str) -> bool {
    code_eq(code, REFUNDING_BUDGET_CODE)
}

pub fn is_refunded_budget_code(code: &str) -> bool {
    code_eq(code, REFUNDED_BUDGET_CODE)
}

pub fn is_refund_budget_code(code: &str) -> bool {
    is_refunding_budget_code(code) || is_refunded_budget_code(code)
}

pub fn is_reserved_budget_code(code: &str) -> bool {
    canonical_special_budget_code(code).is_some()
}

pub fn canonical_special_budget_code(code: &str) -> Option<&'static str> {
    BudgetSpecialKind::from_code(code).canonical_code()
}

fn code_eq(code: &str, expected: &str) -> bool {
    code.trim().eq_ignore_ascii_case(expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_budget_codes_are_reserved_case_insensitively() {
        assert!(is_reserved_budget_code(" inc "));
        assert!(is_reserved_budget_code("transfer"));
        assert!(is_reserved_budget_code("refunding"));
        assert!(is_reserved_budget_code("refunded"));
        assert!(!is_reserved_budget_code("INC-OTHER"));
    }

    #[test]
    fn refund_codes_are_grouped() {
        assert!(is_refund_budget_code("REFUNDING"));
        assert!(is_refund_budget_code("refunded"));
        assert!(!is_refund_budget_code("refund"));
    }

    #[test]
    fn special_kind_can_be_loaded_from_alias_column() {
        assert_eq!(
            budget_special_kind_for_config("planned-income", "SALARY"),
            BudgetSpecialKind::PlannedIncome
        );
        assert_eq!(
            budget_special_kind_for_config("", "transfer"),
            BudgetSpecialKind::Transfer
        );
        assert_eq!(
            budget_special_kind_for_config("", "SALARY"),
            BudgetSpecialKind::None
        );
    }
}
