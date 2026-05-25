use super::history::{show_status_history_dialog, StatusLogEntry};
use super::*;

pub(in crate::app) fn connect_embedded_status_bar(
    window: &adw::ApplicationWindow,
    status_bar: &StatusBar,
    status_autohide: Rc<Cell<bool>>,
) {
    let generation = Rc::new(Cell::new(0u64));
    let history = Rc::new(RefCell::new(Vec::<StatusLogEntry>::new()));
    let history_dialog = Rc::new(RefCell::new(None::<adw::Dialog>));

    let window_for_history = window.clone();
    let history_for_button = Rc::clone(&history);
    let history_dialog_for_button = Rc::clone(&history_dialog);
    status_bar.history_button.connect_clicked(move |_| {
        show_status_history_dialog(
            &window_for_history,
            &history_for_button,
            &history_dialog_for_button,
            None,
        );
    });

    let container_for_hide = status_bar.container.clone();
    let generation_for_hide = Rc::clone(&generation);
    status_bar.hide_button.connect_clicked(move |_| {
        generation_for_hide.set(generation_for_hide.get().wrapping_add(1));
        container_for_hide.set_visible(false);
    });

    let container_for_label = status_bar.container.clone();
    let generation_for_label = Rc::clone(&generation);
    let status_autohide_for_label = Rc::clone(&status_autohide);
    let history_for_label = Rc::clone(&history);
    status_bar.label.connect_label_notify(move |label| {
        let message = label.text().trim().to_string();
        if message.is_empty() {
            container_for_label.set_visible(false);
            return;
        }

        push_status_history_entry(&history_for_label, &message);
        let current = generation_for_label.get().wrapping_add(1);
        generation_for_label.set(current);
        container_for_label.set_visible(true);
        schedule_embedded_status_autohide(
            &container_for_label,
            &status_autohide_for_label,
            &generation_for_label,
            current,
        );
    });

    schedule_embedded_status_watchdog(&status_bar.container, &status_bar.label, &status_autohide);
}

pub(in crate::app) fn connect_status_actions(
    app: &adw::Application,
    ui: &Rc<UiHandles>,
    history_button: gtk::Button,
    hide_button: gtk::Button,
) {
    let history_dialog = Rc::new(RefCell::new(None::<adw::Dialog>));

    let window_for_history = ui.window.clone();
    let history_for_history = Rc::clone(&ui.status_history);
    let history_dialog_for_button = Rc::clone(&history_dialog);
    let ui_for_history = Rc::clone(ui);
    history_button.connect_clicked(move |_| {
        show_status_history_dialog(
            &window_for_history,
            &history_for_history,
            &history_dialog_for_button,
            Some(Rc::clone(&ui_for_history)),
        );
    });

    let window_for_history_action = ui.window.clone();
    let history_for_history_action = Rc::clone(&ui.status_history);
    let history_dialog_for_action = Rc::clone(&history_dialog);
    let ui_for_history_action = Rc::clone(ui);
    let history_action = gtk::gio::SimpleAction::new("status-history", None);
    history_action.connect_activate(move |_, _| {
        show_status_history_dialog(
            &window_for_history_action,
            &history_for_history_action,
            &history_dialog_for_action,
            Some(Rc::clone(&ui_for_history_action)),
        );
    });
    app.add_action(&history_action);

    let ui_for_hide = Rc::clone(ui);
    hide_button.connect_clicked(move |_| {
        hide_status(&ui_for_hide);
    });

    let autohide_action = gtk::gio::SimpleAction::new_stateful(
        "autohide-status",
        None,
        &ui.status_autohide.get().to_variant(),
    );
    autohide_action.set_enabled(ui.preferences.action_is_writable("autohide-status"));
    autohide_action.connect_activate(move |action, _| {
        let enabled = action
            .state()
            .and_then(|state| state.get::<bool>())
            .unwrap_or(false);
        action.change_state(&(!enabled).to_variant());
    });
    let ui_for_autohide = Rc::clone(ui);
    autohide_action.connect_change_state(move |action, value| {
        let Some(enabled) = value.and_then(|value| value.get::<bool>()) else {
            return;
        };

        ui_for_autohide.status_autohide.set(enabled);
        ui_for_autohide.preferences.set_autohide_status_bar(enabled);
        action.set_state(&enabled.to_variant());
        if enabled {
            schedule_status_autohide(&ui_for_autohide, ui_for_autohide.status_generation.get());
        } else if !ui_for_autohide.status.text().is_empty() {
            ui_for_autohide.status_bar.set_visible(true);
        }
    });
    app.add_action(&autohide_action);
}

pub(in crate::app) fn show_status(ui: &UiHandles, message: &str) {
    let generation = ui.status_generation.get().wrapping_add(1);
    ui.status_generation.set(generation);
    let message = tr(message);
    push_status_history(ui, &message);
    ui.status.set_text(&message);
    ui.status_bar.set_visible(true);
    schedule_status_autohide(ui, generation);
}

fn push_status_history(ui: &UiHandles, message: &str) {
    push_status_history_entry(&ui.status_history, message);
}

fn push_status_history_entry(history: &Rc<RefCell<Vec<StatusLogEntry>>>, message: &str) {
    let message = message.trim();
    if message.is_empty() {
        return;
    }

    let mut history = history.borrow_mut();
    if history
        .last()
        .map(|entry| entry.message.as_str())
        .is_some_and(|last_message| last_message == message)
    {
        return;
    }

    history.push(StatusLogEntry {
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        message: message.to_string(),
    });
}

pub(in crate::app) fn schedule_status_autohide_after_loading(ui: &UiHandles) {
    if ui.status.text().is_empty() || !ui.status_bar.is_visible() {
        return;
    }

    let generation = ui.status_generation.get().wrapping_add(1);
    ui.status_generation.set(generation);
    schedule_status_autohide(ui, generation);
}

fn hide_status(ui: &UiHandles) {
    ui.status_generation
        .set(ui.status_generation.get().wrapping_add(1));
    ui.status_bar.set_visible(false);
}

fn schedule_embedded_status_watchdog(
    status_bar: &gtk::Box,
    label: &gtk::Label,
    status_autohide: &Rc<Cell<bool>>,
) {
    let status_bar = status_bar.clone();
    let label = label.clone();
    let status_autohide = Rc::clone(status_autohide);
    gtk::glib::timeout_add_seconds_local(STATUS_AUTOHIDE_SECONDS, move || {
        if status_bar.root().is_none() {
            return gtk::glib::ControlFlow::Break;
        }
        if status_autohide.get() && status_bar.is_visible() && !label.text().is_empty() {
            status_bar.set_visible(false);
        }
        gtk::glib::ControlFlow::Continue
    });
}

fn schedule_embedded_status_autohide(
    status_bar: &gtk::Box,
    status_autohide: &Rc<Cell<bool>>,
    status_generation: &Rc<Cell<u64>>,
    generation: u64,
) {
    if !status_autohide.get() {
        return;
    }

    let status_bar = status_bar.clone();
    let status_autohide = Rc::clone(status_autohide);
    let status_generation = Rc::clone(status_generation);
    gtk::glib::timeout_add_seconds_local(STATUS_AUTOHIDE_SECONDS, move || {
        if status_autohide.get() && status_generation.get() == generation {
            status_bar.set_visible(false);
        }
        gtk::glib::ControlFlow::Break
    });
}

fn schedule_status_autohide(ui: &UiHandles, generation: u64) {
    if !ui.status_autohide.get() {
        return;
    }

    let status_bar = ui.status_bar.clone();
    let status_autohide = Rc::clone(&ui.status_autohide);
    let status_generation = Rc::clone(&ui.status_generation);
    let loading_count = Rc::clone(&ui.loading_count);
    gtk::glib::timeout_add_seconds_local(STATUS_AUTOHIDE_SECONDS, move || {
        if status_autohide.get()
            && status_generation.get() == generation
            && loading_count.get() == 0
        {
            status_bar.set_visible(false);
        }
        gtk::glib::ControlFlow::Break
    });
}
