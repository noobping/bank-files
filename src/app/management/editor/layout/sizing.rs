use super::super::*;
use super::*;

pub(super) fn management_dialog_content_size(window: &adw::ApplicationWindow) -> (i32, i32) {
    management_dialog_content_dimensions(
        effective_parent_dimension(window.width(), window.default_width()),
        effective_parent_dimension(window.height(), window.default_height()),
    )
}

fn effective_parent_dimension(current: i32, default: i32) -> i32 {
    if current > 0 {
        current
    } else {
        default
    }
}

pub(super) fn management_dialog_content_dimensions(
    parent_width: i32,
    parent_height: i32,
) -> (i32, i32) {
    (
        management_dialog_content_dimension(
            parent_width,
            MANAGEMENT_DIALOG_MIN_WIDTH,
            MANAGEMENT_DIALOG_FALLBACK_WIDTH,
        ),
        management_dialog_content_dimension(
            parent_height,
            MANAGEMENT_DIALOG_MIN_HEIGHT,
            MANAGEMENT_DIALOG_FALLBACK_HEIGHT,
        ),
    )
}

fn management_dialog_content_dimension(parent: i32, minimum: i32, fallback: i32) -> i32 {
    if parent > 0 {
        (parent - MANAGEMENT_DIALOG_PARENT_INSET).max(minimum)
    } else {
        fallback
    }
}

pub(super) fn partition_planned_income_budget(
    budgets: Vec<EditableBudget>,
) -> (Option<EditableBudget>, Vec<EditableBudget>) {
    let mut planned_income_budget = None;
    let mut regular_budgets = Vec::new();
    for budget in budgets {
        if crate::model::budget_special_kind_for_config(&budget.special, &budget.code)
            .is_planned_income()
            || planned_income::is_budget_code(&budget.code)
        {
            planned_income_budget.get_or_insert(budget);
        } else {
            regular_budgets.push(budget);
        }
    }
    (planned_income_budget, regular_budgets)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget(code: &str) -> EditableBudget {
        EditableBudget {
            code: code.to_string(),
            special: String::new(),
            category: code.to_string(),
            monthly_budget: "0".to_string(),
            yearly_budget: String::new(),
            direction: "expense".to_string(),
            income_basis: "real".to_string(),
            notes: String::new(),
        }
    }

    #[test]
    fn planned_income_budget_is_only_split_when_configured() {
        let (planned, regular) = partition_planned_income_budget(vec![budget("FOOD")]);

        assert!(planned.is_none());
        assert_eq!(regular.len(), 1);
        assert_eq!(regular[0].code, "FOOD");
    }

    #[test]
    fn configured_inc_budget_is_split_into_special_form_source() {
        let (planned, regular) =
            partition_planned_income_budget(vec![budget("FOOD"), budget("inc")]);

        assert_eq!(planned.map(|budget| budget.code), Some("inc".to_string()));
        assert_eq!(regular.len(), 1);
        assert_eq!(regular[0].code, "FOOD");
    }

    #[test]
    fn management_dialog_size_tracks_parent_with_inset() {
        assert_eq!(management_dialog_content_dimensions(1250, 900), (1202, 852));
    }

    #[test]
    fn management_dialog_size_uses_minimum_for_small_parent() {
        assert_eq!(management_dialog_content_dimensions(350, 380), (320, 360));
    }

    #[test]
    fn management_dialog_size_uses_fallback_before_parent_is_allocated() {
        assert_eq!(
            management_dialog_content_dimensions(0, 0),
            (
                MANAGEMENT_DIALOG_FALLBACK_WIDTH,
                MANAGEMENT_DIALOG_FALLBACK_HEIGHT
            )
        );
    }
}
