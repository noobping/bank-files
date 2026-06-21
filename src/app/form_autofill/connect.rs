use super::inference::infer_budget_autofill_entry;
use super::*;

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

fn set_text_combo_value(combo: &gtk::ComboBoxText, value: &str) {
    let value = value.trim();
    combo.set_active_id(if value.is_empty() { None } else { Some(value) });
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
