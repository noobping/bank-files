use super::*;
use crate::model::AppData;

#[test]
fn automatic_configuration_requires_smart_insights_feature() {
    let error = generate_automatic_configuration(&AppData::default(), true).unwrap_err();

    assert!(format!("{error:#}").contains(SMART_INSIGHTS_REQUIRED_MESSAGE));
}

#[test]
fn generated_budget_code_uses_readable_category_slug_without_smart_insights() {
    assert_eq!(
        generated_budget_code_for_category("Dining out & coffee", &[]),
        "DINING-OUT-COFFEE"
    );
    assert_eq!(generated_budget_code_for_category("!!!", &[]), "BUDGET");
}
