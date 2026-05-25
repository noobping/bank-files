pub const PLANNED_INCOME_BUDGET_CODE: &str = "INC";
pub const TRANSFER_BUDGET_CODE: &str = "TRANSFER";
pub const REFUNDING_BUDGET_CODE: &str = "REFUNDING";
pub const REFUNDED_BUDGET_CODE: &str = "REFUNDED";

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
    if is_planned_income_budget_code(code) {
        Some(PLANNED_INCOME_BUDGET_CODE)
    } else if is_transfer_budget_code(code) {
        Some(TRANSFER_BUDGET_CODE)
    } else if is_refunding_budget_code(code) {
        Some(REFUNDING_BUDGET_CODE)
    } else if is_refunded_budget_code(code) {
        Some(REFUNDED_BUDGET_CODE)
    } else {
        None
    }
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
}
