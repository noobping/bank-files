use super::*;

pub fn form_grid() -> gtk::Grid {
    gtk::Grid::builder()
        .column_spacing(10)
        .row_spacing(8)
        .hexpand(true)
        .build()
}

pub fn form_box() -> gtk::Box {
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 10);
    box_.set_hexpand(true);
    box_
}

pub fn add_labeled_stacked(
    container: &gtk::Box,
    label: &str,
    widget: &impl IsA<gtk::Widget>,
) -> gtk::Label {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 4);
    row.set_hexpand(true);
    let label = gtk::Label::new(Some(&gettext(label)));
    label.set_xalign(0.0);
    label.add_css_class("caption");
    label.add_css_class("dim-label");
    row.append(&label);

    if widget.as_ref().is::<gtk::Switch>() || widget.as_ref().is::<gtk::SpinButton>() {
        widget.set_halign(gtk::Align::Start);
        widget.set_hexpand(false);
    } else {
        widget.set_hexpand(true);
        widget.set_halign(gtk::Align::Fill);
    }
    row.append(widget);
    container.append(&row);
    label
}

pub fn add_labeled(
    grid: &gtk::Grid,
    row: i32,
    label: &str,
    widget: &impl IsA<gtk::Widget>,
) -> gtk::Label {
    let label = gtk::Label::new(Some(&gettext(label)));
    label.set_xalign(0.0);
    label.add_css_class("caption");
    label.set_width_chars(14);
    grid.attach(&label, 0, row, 1, 1);

    if widget.as_ref().is::<gtk::Switch>() || widget.as_ref().is::<gtk::SpinButton>() {
        widget.set_halign(gtk::Align::Start);
        widget.set_hexpand(false);
    } else {
        widget.set_hexpand(true);
        widget.set_halign(gtk::Align::Fill);
    }
    grid.attach(widget, 1, row, 1, 1);
    label
}

pub fn entry(text: &str, placeholder: &str) -> gtk::Entry {
    let entry = gtk::Entry::new();
    entry.set_text(text);
    entry.set_placeholder_text(Some(&gettext(placeholder)));
    entry.set_hexpand(true);
    entry
}

pub fn text_combo(selected: &str, values: impl IntoIterator<Item = String>) -> gtk::ComboBoxText {
    let combo = gtk::ComboBoxText::with_entry();
    let selected = selected.trim();
    let mut values = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    if !selected.is_empty()
        && !values
            .iter()
            .any(|value| value.eq_ignore_ascii_case(selected))
    {
        values.push(selected.to_string());
    }
    values.sort_by_key(|value| value.to_ascii_uppercase());
    values.dedup_by(|left, right| left.eq_ignore_ascii_case(right));

    for value in values {
        combo.append(Some(&value), &value);
    }
    if !selected.is_empty() {
        combo.set_active_id(Some(selected));
    }
    combo
}

pub fn combo_text(combo: &gtk::ComboBoxText) -> String {
    if let Some(text) = combo
        .child()
        .and_then(|child| child.downcast::<gtk::Entry>().ok())
        .map(|entry| entry.text().trim().to_string())
        .filter(|text| !text.is_empty())
    {
        return text;
    }

    combo
        .active_text()
        .map(|text| text.trim().to_string())
        .unwrap_or_default()
}

pub fn combo_from_options(options: &[(&str, &str)], active: &str) -> gtk::ComboBoxText {
    let combo = gtk::ComboBoxText::new();
    for (id, label) in options {
        combo.append(Some(id), &gettext(label));
    }
    combo.set_active_id(Some(active));
    if combo.active_id().is_none() {
        combo.set_active(Some(0));
    }
    combo
}

pub fn combo_active_id(combo: &gtk::ComboBoxText) -> String {
    combo
        .active_id()
        .map(|id| id.to_string())
        .unwrap_or_default()
}

pub fn budget_direction_id(input: &str) -> &'static str {
    match crate::model::BudgetDirection::parse(input, "", "") {
        crate::model::BudgetDirection::Income => "income",
        crate::model::BudgetDirection::Transfer => "transfer",
        crate::model::BudgetDirection::Expense => "expense",
    }
}

pub fn budget_income_basis_id(input: &str) -> &'static str {
    crate::model::BudgetIncomeBasis::parse(input).as_str()
}
