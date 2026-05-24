use super::entries::same_budget_autofill_entry;
use super::*;

pub(super) fn infer_budget_autofill_entry<'a>(
    category: &gtk::ComboBoxText,
    budget_code: &gtk::ComboBoxText,
    direction: &gtk::ComboBoxText,
    entries: &'a [BudgetAutofillEntry],
    source: BudgetAutofillSource,
) -> Option<&'a BudgetAutofillEntry> {
    let category_text = ui::combo_text(category);
    let code_text = ui::combo_text(budget_code);
    let direction_id = ui::combo_active_id(direction);

    match source {
        BudgetAutofillSource::Category => infer_from_category(&category_text, entries),
        BudgetAutofillSource::Code => infer_from_code(&code_text, entries),
        BudgetAutofillSource::Direction => infer_from_direction(&direction_id, &code_text, entries),
        BudgetAutofillSource::Initial => infer_from_code(&code_text, entries)
            .or_else(|| infer_from_category(&category_text, entries))
            .or_else(|| infer_from_direction(&direction_id, &code_text, entries)),
    }
}

fn infer_from_category<'a>(
    category: &str,
    entries: &'a [BudgetAutofillEntry],
) -> Option<&'a BudgetAutofillEntry> {
    if category_implies_transfer(category) {
        return preferred_transfer_entry(entries);
    }

    if category.trim().is_empty() {
        return None;
    }

    unique_matching_entry(entries, |entry| {
        entry.category.eq_ignore_ascii_case(category.trim())
    })
}

fn infer_from_code<'a>(
    code: &str,
    entries: &'a [BudgetAutofillEntry],
) -> Option<&'a BudgetAutofillEntry> {
    if code.trim().is_empty() {
        return None;
    }

    unique_matching_entry(entries, |entry| {
        entry.code.eq_ignore_ascii_case(code.trim())
    })
}

fn infer_from_direction<'a>(
    direction: &str,
    code: &str,
    entries: &'a [BudgetAutofillEntry],
) -> Option<&'a BudgetAutofillEntry> {
    if direction == "transfer" {
        return preferred_transfer_entry(entries);
    }

    infer_from_code(code, entries)
}

fn preferred_transfer_entry(entries: &[BudgetAutofillEntry]) -> Option<&BudgetAutofillEntry> {
    unique_matching_entry(entries, |entry| {
        entry.direction == "transfer" && entry.code.eq_ignore_ascii_case("TRANSFER")
    })
    .or_else(|| unique_matching_entry(entries, |entry| entry.direction == "transfer"))
}

fn category_implies_transfer(category: &str) -> bool {
    let category = category.trim().to_lowercase();
    !category.is_empty()
        && (category.contains("transfer")
            || category.contains("overboeking")
            || category.contains("overboekingen")
            || category.contains("ueberweisung")
            || category.contains("uberweisung")
            || category.contains("überweisung")
            || category.contains("umbuchung"))
}

fn unique_matching_entry(
    entries: &[BudgetAutofillEntry],
    mut matches: impl FnMut(&BudgetAutofillEntry) -> bool,
) -> Option<&BudgetAutofillEntry> {
    let mut found = None;
    for entry in entries.iter().filter(|entry| matches(entry)) {
        match found {
            Some(previous) if !same_budget_autofill_entry(previous, entry) => return None,
            Some(_) => {}
            None => found = Some(entry),
        }
    }
    found
}
