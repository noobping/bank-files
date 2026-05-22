use super::*;

pub(in crate::app) const BUDGET_CODE: &str = "INC";
pub(in crate::app) const NET_INCOME_REMINDER: &str =
    "Income can be gross or net. Net income is needed for budgeting and calculations.";

pub(in crate::app) fn is_budget_code(code: &str) -> bool {
    code.trim().eq_ignore_ascii_case(BUDGET_CODE)
}

pub(in crate::app) fn fixed_budget_amount_text(input: &str) -> String {
    input
        .trim()
        .split_once('%')
        .map(|(amount, _)| amount.trim())
        .unwrap_or_else(|| input.trim())
        .to_string()
}

pub(in crate::app) fn connect_fixed_budget_entry(entry: &gtk::Entry) {
    entry.connect_changed(|entry| {
        let fixed_text = fixed_budget_amount_text(&entry.text());
        if entry.text().trim() != fixed_text {
            entry.set_text(&fixed_text);
        }
    });
}

pub(in crate::app) fn editable_budget(
    category: String,
    monthly_budget: String,
    yearly_budget: String,
    notes: String,
) -> EditableBudget {
    EditableBudget {
        code: BUDGET_CODE.to_string(),
        category,
        monthly_budget: fixed_budget_amount_text(&monthly_budget),
        yearly_budget: fixed_budget_amount_text(&yearly_budget),
        direction: "income".to_string(),
        income_basis: "real".to_string(),
        notes: notes.trim().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planned_income_budget_code_is_reserved() {
        assert!(is_budget_code("inc"));
        assert!(is_budget_code(" INC "));
        assert!(!is_budget_code("INC-OTHER"));
    }

    #[test]
    fn planned_income_budget_amounts_are_fixed_values() {
        assert_eq!(fixed_budget_amount_text("10% of income"), "10");
        assert_eq!(fixed_budget_amount_text("20000"), "20000");
    }

    #[test]
    fn planned_income_editable_budget_uses_canonical_fields() {
        let budget = editable_budget(
            "Income".to_string(),
            "500%".to_string(),
            "20000".to_string(),
            " Planned ".to_string(),
        );

        assert_eq!(budget.code, BUDGET_CODE);
        assert_eq!(budget.monthly_budget, "500");
        assert_eq!(budget.yearly_budget, "20000");
        assert_eq!(budget.direction, "income");
        assert_eq!(budget.income_basis, "real");
        assert_eq!(budget.notes, "Planned");
    }
}
