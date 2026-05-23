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
    combo_text_value(
        combo.active_text().map(|text| text.to_string()),
        combo
            .child()
            .and_then(|child| child.downcast::<gtk::Entry>().ok())
            .map(|entry| entry.text().to_string()),
    )
}

fn combo_text_value(active_text: Option<String>, entry_text: Option<String>) -> String {
    normalized_combo_text(active_text)
        .or_else(|| normalized_combo_text(entry_text))
        .unwrap_or_default()
}

fn normalized_combo_text(text: Option<String>) -> Option<String> {
    text.map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
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

pub fn focus_button_after_combo_selection(combo: &gtk::ComboBoxText, button: &gtk::Button) {
    if !combo.has_entry() {
        combo.set_focus_on_click(false);
    }

    let button_for_change = button.clone();
    let combo_for_change = combo.clone();
    combo.connect_changed(move |_| {
        finish_combo_selection(&combo_for_change, &button_for_change);
    });

    let button_for_popdown = button.clone();
    combo.connect_popdown(move |_| {
        focus_button_after_selection_commit(&button_for_popdown);
        false
    });
}

fn finish_combo_selection(combo: &gtk::ComboBoxText, button: &gtk::Button) {
    close_combo_popup(combo);

    let combo_for_idle = combo.clone();
    let button_for_idle = button.clone();
    gtk::glib::idle_add_local_once(move || {
        close_combo_popup(&combo_for_idle);
        focus_button_if_available(&button_for_idle);
    });

    let combo_for_timeout = combo.clone();
    let button_for_timeout = button.clone();
    gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(80), move || {
        close_combo_popup(&combo_for_timeout);
        focus_button_if_available(&button_for_timeout);
    });
}

fn focus_button_after_selection_commit(button: &gtk::Button) {
    let button_for_idle = button.clone();
    gtk::glib::idle_add_local_once(move || focus_button_if_available(&button_for_idle));

    let button_for_timeout = button.clone();
    gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(80), move || {
        focus_button_if_available(&button_for_timeout);
    });
}

fn focus_button_if_available(button: &gtk::Button) {
    if button.is_visible() && button.is_sensitive() {
        button.grab_focus();
    }
}

fn close_combo_popup(combo: &gtk::ComboBoxText) {
    if combo.is_popup_shown() {
        combo.popdown();
    }
}

pub fn focus_button_after_combo_selections(button: &gtk::Button, combos: &[&gtk::ComboBoxText]) {
    for combo in combos {
        focus_button_after_combo_selection(combo, button);
    }
}

pub fn connect_button_activation<F>(button: &gtk::Button, action: F)
where
    F: Fn(&gtk::Button) + 'static,
{
    button.set_receives_default(true);

    let action: Rc<dyn Fn(&gtk::Button)> = Rc::new(action);
    let activating = Rc::new(Cell::new(false));

    let clicked_action = Rc::clone(&action);
    let clicked_activating = Rc::clone(&activating);
    button.connect_clicked(move |button| {
        run_button_activation(button, &clicked_action, &clicked_activating);
    });

    let click = gtk::GestureClick::new();
    click.set_button(0);
    click.set_propagation_phase(gtk::PropagationPhase::Capture);
    let press_button = button.clone();
    let press_action = Rc::clone(&action);
    let press_activating = Rc::clone(&activating);
    click.connect_pressed(move |_, _, x, y| {
        if point_is_inside_widget(&press_button, x, y) {
            run_button_activation(&press_button, &press_action, &press_activating);
        }
    });
    button.add_controller(click);
}

fn run_button_activation(
    button: &gtk::Button,
    action: &Rc<dyn Fn(&gtk::Button)>,
    activating: &Rc<Cell<bool>>,
) {
    if activating.get() || !button.is_sensitive() || !button.is_visible() {
        return;
    }

    activating.set(true);
    action(button);

    let activating = Rc::clone(activating);
    gtk::glib::idle_add_local_once(move || activating.set(false));
}

fn point_is_inside_widget(widget: &impl IsA<gtk::Widget>, x: f64, y: f64) -> bool {
    x >= 0.0
        && y >= 0.0
        && x < f64::from(widget.as_ref().allocated_width())
        && y < f64::from(widget.as_ref().allocated_height())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combo_text_value_prefers_selected_row_over_stale_entry() {
        assert_eq!(
            combo_text_value(Some("Selected".to_string()), Some("Old entry".to_string())),
            "Selected"
        );
    }

    #[test]
    fn combo_text_value_falls_back_to_entry_text() {
        assert_eq!(
            combo_text_value(None, Some(" Custom value ".to_string())),
            "Custom value"
        );
        assert_eq!(
            combo_text_value(Some("   ".to_string()), Some(" Custom value ".to_string())),
            "Custom value"
        );
    }
}
