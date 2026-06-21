use super::budget_bulk::{budget_values_for_period, BudgetValuePeriod, BudgetValueUpdate};
use super::budget_move::{
    budget_move_changes_direction, budget_move_code_is_eligible, BudgetMoveTarget,
};

#[test]
fn budget_values_convert_monthly_fixed_values_to_yearly() {
    assert_eq!(
        budget_values_for_period("100", "", BudgetValuePeriod::Yearly),
        BudgetValueUpdate::Changed {
            monthly: String::new(),
            yearly: "1200".to_string(),
        }
    );
}

#[test]
fn budget_values_convert_yearly_fixed_values_to_monthly() {
    assert_eq!(
        budget_values_for_period("", "1200", BudgetValuePeriod::Monthly),
        BudgetValueUpdate::Changed {
            monthly: "100".to_string(),
            yearly: String::new(),
        }
    );
}

#[test]
fn budget_values_keep_percentages_when_switching_period() {
    assert_eq!(
        budget_values_for_period("10%", "", BudgetValuePeriod::Yearly),
        BudgetValueUpdate::Changed {
            monthly: String::new(),
            yearly: "10%".to_string(),
        }
    );
}

#[test]
fn budget_values_skip_invalid_conversion_sources() {
    assert_eq!(
        budget_values_for_period("", "not money", BudgetValuePeriod::Monthly),
        BudgetValueUpdate::Skipped
    );
}

#[test]
fn simple_budget_move_detects_direction_changes() {
    let options = vec![
        BudgetMoveTarget {
            code: "FOOD".to_string(),
            category: "Groceries".to_string(),
            direction: "expense".to_string(),
        },
        BudgetMoveTarget {
            code: "OTHER".to_string(),
            category: "Other".to_string(),
            direction: "expense".to_string(),
        },
        BudgetMoveTarget {
            code: "SALARY".to_string(),
            category: "Salary".to_string(),
            direction: "income".to_string(),
        },
    ];

    assert!(!budget_move_changes_direction(&options, "FOOD", "OTHER"));
    assert!(budget_move_changes_direction(&options, "FOOD", "SALARY"));
}

#[test]
fn budget_move_keeps_planned_income_available() {
    assert!(budget_move_code_is_eligible("INC", &[]));
    assert!(!budget_move_code_is_eligible(
        "INC",
        &[BudgetMoveTarget {
            code: "inc".to_string(),
            category: "Income".to_string(),
            direction: "income".to_string(),
        }]
    ));
}
