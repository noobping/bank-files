use super::*;

#[test]
fn localized_default_budget_files_include_special_budget_kinds() {
    for contents in [
        include_str!("../../../data/defaults/budgetcodes.csv"),
        include_str!("../../../data/defaults/budgetcodes.nl.csv"),
        include_str!("../../../data/defaults/budgetcodes.de.csv"),
    ] {
        let budgets = parse_editable_budgets(contents).unwrap();
        assert_default_special_budget(&budgets, "INC", "planned-income");
        assert_default_special_budget(&budgets, "TRANSFER", "transfer");
        assert_default_special_budget(&budgets, "REFUNDING", "refunding");
        assert_default_special_budget(&budgets, "REFUNDED", "refunded");
    }
}

fn assert_default_special_budget(budgets: &[EditableBudget], code: &str, special: &str) {
    assert!(budgets
        .iter()
        .any(|budget| budget.code == code && budget.special == special));
}
