use super::*;

pub fn toggle_search_bar(search_bar: &gtk::SearchBar, search_entry: &gtk::SearchEntry) {
    let enabled = !search_bar.is_search_mode();
    search_bar.set_search_mode(enabled);
    if enabled {
        search_entry.grab_focus();
        search_entry.select_region(0, -1);
    }
}

pub fn connect_search_button(
    search_button: &gtk::Button,
    search_bar: &gtk::SearchBar,
    search_entry: &gtk::SearchEntry,
) {
    let search_bar_for_button = search_bar.clone();
    let search_entry_for_button = search_entry.clone();
    search_button.connect_clicked(move |_| {
        toggle_search_bar(&search_bar_for_button, &search_entry_for_button);
    });
}

pub fn connect_primary_f_shortcut(widget: &impl IsA<gtk::Widget>, on_find: impl Fn() + 'static) {
    let key_controller = gtk::EventControllerKey::new();
    key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
    key_controller.connect_key_pressed(move |_, key, _, modifier| {
        if !is_primary_f_shortcut(key.to_unicode(), modifier) {
            return gtk::glib::Propagation::Proceed;
        }

        on_find();
        gtk::glib::Propagation::Stop
    });
    widget.add_controller(key_controller);
}

pub fn connect_search_shortcut(
    widget: &impl IsA<gtk::Widget>,
    search_bar: &gtk::SearchBar,
    search_entry: &gtk::SearchEntry,
) {
    let search_bar_for_shortcut = search_bar.clone();
    let search_entry_for_shortcut = search_entry.clone();
    connect_primary_f_shortcut(widget, move || {
        toggle_search_bar(&search_bar_for_shortcut, &search_entry_for_shortcut);
    });
}

pub fn setup_search_bar(
    capture_widget: &impl IsA<gtk::Widget>,
    search_bar: &gtk::SearchBar,
    search_entry: &gtk::SearchEntry,
) {
    search_bar.connect_entry(search_entry);
    search_bar.set_key_capture_widget(Some(capture_widget));
}

pub fn bind_search_bar(
    shortcut_widget: &impl IsA<gtk::Widget>,
    capture_widget: &impl IsA<gtk::Widget>,
    search_bar: &gtk::SearchBar,
    search_entry: &gtk::SearchEntry,
) {
    setup_search_bar(capture_widget, search_bar, search_entry);
    connect_search_shortcut(shortcut_widget, search_bar, search_entry);
}

fn is_primary_f_shortcut(key: Option<char>, modifier: gtk::gdk::ModifierType) -> bool {
    modifier.contains(gtk::gdk::ModifierType::CONTROL_MASK)
        && !modifier.contains(gtk::gdk::ModifierType::ALT_MASK)
        && matches!(key, Some('f') | Some('F'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_f_shortcut_requires_control_f_without_alt() {
        let control = gtk::gdk::ModifierType::CONTROL_MASK;
        let control_alt = control | gtk::gdk::ModifierType::ALT_MASK;

        assert!(is_primary_f_shortcut(Some('f'), control));
        assert!(is_primary_f_shortcut(Some('F'), control));
        assert!(!is_primary_f_shortcut(Some('x'), control));
        assert!(!is_primary_f_shortcut(Some('f'), control_alt));
        assert!(!is_primary_f_shortcut(
            Some('f'),
            gtk::gdk::ModifierType::empty()
        ));
    }
}
