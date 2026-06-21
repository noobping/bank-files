use super::super::*;
use super::card::CollapsibleFormCard;

pub(super) fn set_option_combo(combo: &gtk::ComboBoxText, value: &str) {
    combo.set_active_id(Some(value.trim()));
    if combo.active_id().is_none() {
        combo.set_active(Some(0));
    }
}

pub(super) fn connect_delete_button(
    button: &gtk::Button,
    deleted: &Rc<Cell<bool>>,
    form_box: &gtk::Box,
) {
    let deleted_for_button = Rc::clone(deleted);
    let form_box_for_delete = form_box.clone();
    button.connect_clicked(move |_| {
        deleted_for_button.set(true);
        form_box_for_delete.set_visible(false);
    });
}

pub(super) fn connect_budget_delete_button(
    button: &gtk::Button,
    deleted: &Rc<Cell<bool>>,
    form_box: &gtk::Box,
) {
    let button_for_delete = button.clone();
    let deleted_for_button = Rc::clone(deleted);
    let form_box_for_delete = form_box.clone();
    button.connect_clicked(move |_| {
        let should_delete = !deleted_for_button.get();
        set_budget_delete_state(
            &form_box_for_delete,
            &button_for_delete,
            &deleted_for_button,
            should_delete,
        );
    });
}

pub(super) fn attach_details_grid(card: &CollapsibleFormCard, grid: &gtk::Grid) {
    grid.set_margin_start(12);
    grid.set_margin_end(12);
    grid.set_margin_bottom(12);
    card.details.append(grid);
}

pub(super) fn connect_entry_summary(entry: &gtk::Entry, update: &Rc<dyn Fn()>) {
    let update_for_entry = Rc::clone(update);
    entry.connect_changed(move |_| update_for_entry());
}

pub(super) fn connect_combo_summary(combo: &gtk::ComboBoxText, update: &Rc<dyn Fn()>) {
    let update_for_combo = Rc::clone(update);
    combo.connect_changed(move |_| update_for_combo());

    if let Some(entry) = combo
        .child()
        .and_then(|child| child.downcast::<gtk::Entry>().ok())
    {
        let update_for_entry = Rc::clone(update);
        entry.connect_changed(move |_| update_for_entry());
    }
}

pub(super) fn connect_switch_summary(switch: &gtk::Switch, update: &Rc<dyn Fn()>) {
    let update_for_switch = Rc::clone(update);
    switch.connect_active_notify(move |_| update_for_switch());
}

pub(super) fn connect_spin_summary(spin: &gtk::SpinButton, update: &Rc<dyn Fn()>) {
    let update_for_spin = Rc::clone(update);
    spin.connect_value_changed(move |_| update_for_spin());
}

pub(super) fn set_summary(row: &adw::ExpanderRow, values: (String, String)) {
    row.set_title(&values.0);
    row.set_subtitle(&values.1);
}

pub(super) fn combo_display_text(combo: &gtk::ComboBoxText) -> String {
    combo
        .active_text()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| combo_active_id(combo))
}

pub(super) fn entry_summary(entry: &gtk::Entry, fallback: &str) -> String {
    let text = entry.text().trim().to_string();
    if text.is_empty() {
        fallback.to_string()
    } else {
        text
    }
}

pub(super) fn entry_summary_fixed_budget(entry: &gtk::Entry, fallback: &str) -> String {
    entry_summary_text(
        &planned_income::fixed_budget_amount_text(&entry.text()),
        fallback,
    )
}

fn entry_summary_text(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.trim().to_string()
    }
}
