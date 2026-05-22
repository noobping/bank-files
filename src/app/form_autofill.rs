use super::*;

#[derive(Debug, Clone)]
pub(in crate::app) struct BudgetAutofillEntry {
    code: String,
    category: String,
    direction: String,
}

#[derive(Debug, Clone, Copy)]
enum BudgetAutofillSource {
    Initial,
    Category,
    Code,
    Direction,
}

#[derive(Debug, Default)]
struct BudgetAutofillState {
    applying: Cell<bool>,
}

pub(in crate::app) fn app_category_values(data: &AppData) -> Vec<String> {
    unique_texts(
        data.budgets
            .iter()
            .map(|budget| budget.category.trim().to_string()),
    )
}

pub(in crate::app) fn app_budget_code_values(data: &AppData) -> Vec<String> {
    unique_texts(
        data.budgets
            .iter()
            .map(|budget| budget.code.trim().to_string()),
    )
}

pub(in crate::app) fn app_budget_autofill_entries(data: &AppData) -> Vec<BudgetAutofillEntry> {
    budget_autofill_entries(data.budgets.iter().map(|budget| {
        (
            budget.code.clone(),
            budget.category.clone(),
            budget.direction.as_str().to_string(),
        )
    }))
}

pub(in crate::app) fn transaction_rule_search_values(tx: &Transaction) -> Vec<String> {
    unique_texts([
        tx.counterparty.trim().to_string(),
        tx.tags.trim().to_string(),
        tx.description.trim().to_string(),
        tx.account.trim().to_string(),
        tx.transaction_id.trim().to_string(),
    ])
}

pub(in crate::app) fn pattern_rule_search_values(
    pattern: &analytics::TransactionPattern,
) -> Vec<String> {
    unique_texts(
        std::iter::once(pattern.label.trim().to_string()).chain(
            pattern
                .match_labels
                .iter()
                .map(|label| label.trim().to_string()),
        ),
    )
}

pub(in crate::app) fn editable_category_values() -> Vec<String> {
    unique_texts(
        data::load_editable_budgets()
            .unwrap_or_default()
            .into_iter()
            .map(|budget| budget.category),
    )
}

pub(in crate::app) fn editable_budget_code_values() -> Vec<String> {
    unique_texts(
        data::load_editable_budgets()
            .unwrap_or_default()
            .into_iter()
            .map(|budget| budget.code),
    )
}

pub(in crate::app) fn editable_budget_autofill_entries() -> Vec<BudgetAutofillEntry> {
    budget_autofill_entries(
        data::load_editable_budgets()
            .unwrap_or_default()
            .into_iter()
            .map(|budget| (budget.code, budget.category, budget.direction)),
    )
}

pub(in crate::app) fn editable_rule_search_values() -> Vec<String> {
    unique_texts(
        data::load_editable_rules()
            .unwrap_or_default()
            .into_iter()
            .map(|rule| rule.search),
    )
}

fn unique_texts(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut values = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort_by_key(|value| value.to_ascii_uppercase());
    values.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    values
}

pub(in crate::app) fn connect_budget_fields_autofill(
    category: &gtk::ComboBoxText,
    budget_code: &gtk::ComboBoxText,
    direction: &gtk::ComboBoxText,
    entries: Vec<BudgetAutofillEntry>,
    advanced_autofill: &Rc<Cell<bool>>,
) {
    let state = Rc::new(BudgetAutofillState::default());
    let entries = Rc::new(entries);
    let advanced_autofill = Rc::clone(advanced_autofill);

    let category_for_apply = category.clone();
    let budget_code_for_apply = budget_code.clone();
    let direction_for_apply = direction.clone();
    let state_for_apply = Rc::clone(&state);
    let entries_for_apply = Rc::clone(&entries);
    let apply: Rc<dyn Fn(BudgetAutofillSource)> = Rc::new(move |source| {
        apply_budget_fields_autofill(
            &category_for_apply,
            &budget_code_for_apply,
            &direction_for_apply,
            &entries_for_apply,
            &advanced_autofill,
            &state_for_apply,
            source,
        );
    });

    connect_text_combo_changed(category, &state, &apply, BudgetAutofillSource::Category);
    connect_text_combo_changed(budget_code, &state, &apply, BudgetAutofillSource::Code);
    connect_direction_changed(direction, &state, &apply);
    apply(BudgetAutofillSource::Initial);
}

fn budget_autofill_entries(
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

fn connect_text_combo_changed(
    combo: &gtk::ComboBoxText,
    state: &Rc<BudgetAutofillState>,
    apply: &Rc<dyn Fn(BudgetAutofillSource)>,
    source: BudgetAutofillSource,
) {
    let state_for_combo = Rc::clone(state);
    let apply_for_combo = Rc::clone(apply);
    combo.connect_changed(move |_| {
        if !state_for_combo.applying.get() {
            apply_for_combo(source);
        }
    });

    if let Some(entry) = combo
        .child()
        .and_then(|child| child.downcast::<gtk::Entry>().ok())
    {
        let state_for_entry = Rc::clone(state);
        let apply_for_entry = Rc::clone(apply);
        entry.connect_changed(move |_| {
            if !state_for_entry.applying.get() {
                apply_for_entry(source);
            }
        });
    }
}

fn connect_direction_changed(
    direction: &gtk::ComboBoxText,
    state: &Rc<BudgetAutofillState>,
    apply: &Rc<dyn Fn(BudgetAutofillSource)>,
) {
    let state_for_direction = Rc::clone(state);
    let apply_for_direction = Rc::clone(apply);
    direction.connect_changed(move |_| {
        if !state_for_direction.applying.get() {
            apply_for_direction(BudgetAutofillSource::Direction);
        }
    });
}

fn apply_budget_fields_autofill(
    category: &gtk::ComboBoxText,
    budget_code: &gtk::ComboBoxText,
    direction: &gtk::ComboBoxText,
    entries: &[BudgetAutofillEntry],
    advanced_autofill: &Rc<Cell<bool>>,
    state: &BudgetAutofillState,
    source: BudgetAutofillSource,
) {
    if !advanced_autofill.get() || state.applying.get() {
        return;
    }

    let Some(entry) =
        infer_budget_autofill_entry(category, budget_code, direction, entries, source)
    else {
        return;
    };

    state.applying.set(true);
    set_text_combo_value(category, &entry.category);
    set_text_combo_value(budget_code, &entry.code);
    set_direction_value(direction, &entry.direction);
    state.applying.set(false);
}

fn infer_budget_autofill_entry<'a>(
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

fn same_budget_autofill_entry(left: &BudgetAutofillEntry, right: &BudgetAutofillEntry) -> bool {
    left.code.eq_ignore_ascii_case(&right.code)
        && left.category.eq_ignore_ascii_case(&right.category)
        && left.direction == right.direction
}

fn set_text_combo_value(combo: &gtk::ComboBoxText, value: &str) {
    if value.is_empty() || ui::combo_text(combo).eq_ignore_ascii_case(value) {
        return;
    }

    combo.set_active_id(Some(value));
    if ui::combo_text(combo).eq_ignore_ascii_case(value) {
        return;
    }

    if let Some(entry) = combo
        .child()
        .and_then(|child| child.downcast::<gtk::Entry>().ok())
    {
        entry.set_text(value);
    }
}

fn set_direction_value(direction: &gtk::ComboBoxText, value: &str) {
    if !value.is_empty() && ui::combo_active_id(direction) != value {
        direction.set_active_id(Some(value));
    }
}
