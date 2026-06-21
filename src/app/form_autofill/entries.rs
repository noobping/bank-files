use super::*;

pub(super) fn budget_autofill_entries(
    budgets: impl IntoIterator<Item = (String, String, String)>,
) -> Vec<BudgetAutofillEntry> {
    let mut entries = budgets
        .into_iter()
        .filter_map(|(code, category, direction)| {
            normalize_budget_autofill_entry(&code, &category, &direction)
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| {
        (
            entry.code.to_ascii_uppercase(),
            entry.category.to_ascii_uppercase(),
            entry.direction.clone(),
        )
    });
    entries.dedup_by(|left, right| same_budget_autofill_entry(left, right));
    entries
}

fn normalize_budget_autofill_entry(
    code: &str,
    category: &str,
    direction: &str,
) -> Option<BudgetAutofillEntry> {
    let code = code.trim();
    let category = category.trim();
    if code.is_empty() || category.is_empty() {
        return None;
    }

    Some(BudgetAutofillEntry {
        code: code.to_string(),
        category: category.to_string(),
        direction: ui::budget_direction_id(direction).to_string(),
    })
}

pub(super) fn same_budget_autofill_entry(
    left: &BudgetAutofillEntry,
    right: &BudgetAutofillEntry,
) -> bool {
    left.code.eq_ignore_ascii_case(&right.code)
        && left.category.eq_ignore_ascii_case(&right.category)
        && left.direction == right.direction
}
