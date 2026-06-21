use super::*;

pub(in crate::app) fn budget_is_special_neutral(code: &str) -> bool {
    transfer_budget::is_budget_code(code) || refund_budget::is_budget_code(code)
}

pub(in crate::app) fn budget_special_controls_are_hidden(
    advanced_features: bool,
    is_special_neutral_budget: bool,
) -> bool {
    !advanced_features && is_special_neutral_budget
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_mode_hides_transfer_and_refund_locked_controls() {
        assert!(budget_special_controls_are_hidden(false, true));
        assert!(!budget_special_controls_are_hidden(true, true));
        assert!(!budget_special_controls_are_hidden(false, false));
    }
}
