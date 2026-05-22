use super::*;

const STATUS_AUTOHIDE_SECONDS: u32 = 6;
const COPY_FEEDBACK_SECONDS: u32 = 3;
const COPY_ICON: &str = "edit-copy-symbolic";
const COPIED_ICON: &str = "object-select-symbolic";
const STATUS_HISTORY_TITLE_PAGE: &str = "title";
const STATUS_HISTORY_SEARCH_PAGE: &str = "search";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) struct StatusLogEntry {
    pub(in crate::app) timestamp: String,
    pub(in crate::app) message: String,
}

pub(in crate::app) struct StatusBar {
    pub(in crate::app) container: gtk::Box,
    pub(in crate::app) icon: gtk::Image,
    pub(in crate::app) spinner: adw::Spinner,
    pub(in crate::app) label: gtk::Label,
    pub(in crate::app) action_group: gtk::Box,
    pub(in crate::app) copy_button: gtk::Button,
    pub(in crate::app) history_button: gtk::Button,
    pub(in crate::app) hide_button: gtk::Button,
}

pub(in crate::app) fn build_status_bar() -> StatusBar {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    container.add_css_class("toolbar");
    container.set_margin_top(2);
    container.set_margin_bottom(2);
    container.set_margin_start(6);
    container.set_margin_end(6);

    let status_icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    status_icon.add_css_class("dim-label");
    status_icon.set_margin_top(2);
    status_icon.set_margin_bottom(2);
    status_icon.set_margin_start(4);

    let spinner = ui::loading_spinner();
    spinner.add_css_class("dim-label");
    spinner.set_visible(false);
    spinner.set_margin_top(2);
    spinner.set_margin_bottom(2);

    let label = ui::wrapped_label("");
    label.set_selectable(false);
    label.set_hexpand(true);
    label.set_margin_top(2);
    label.set_margin_bottom(2);

    let copy_button = status_button(COPY_ICON, "Copy message");
    let history_button = status_button("document-open-recent-symbolic", "Show message history");
    let hide_button = status_button("window-close-symbolic", "Hide message");
    let action_group = ui::linked_button_group();
    action_group.set_margin_top(2);
    action_group.set_margin_bottom(2);
    action_group.set_margin_end(2);
    action_group.append(&copy_button);
    action_group.append(&history_button);
    action_group.append(&hide_button);

    container.append(&status_icon);
    container.append(&spinner);
    container.append(&label);
    container.append(&action_group);

    StatusBar {
        container,
        icon: status_icon,
        spinner,
        label,
        action_group,
        copy_button,
        history_button,
        hide_button,
    }
}

pub(in crate::app) fn build_page_actions_menu_button(action_namespace: &str) -> gtk::MenuButton {
    let menu_button = gtk::MenuButton::builder()
        .icon_name("view-more-symbolic")
        .tooltip_text(tr("Page actions"))
        .build();
    menu_button.add_css_class("flat");

    let menu = gtk::gio::Menu::new();
    menu.append(
        Some(&tr("Copy Page")),
        Some(&format!("{action_namespace}.copy-page")),
    );
    menu.append(
        Some(&tr("Print Page")),
        Some(&format!("{action_namespace}.print-page")),
    );
    menu.append(
        Some(&tr("Export CSV")),
        Some(&format!("{action_namespace}.export-csv")),
    );
    menu_button.set_menu_model(Some(&menu));
    menu_button
}

pub(in crate::app) fn connect_embedded_status_bar(
    window: &adw::ApplicationWindow,
    status_bar: &StatusBar,
    status_autohide: Rc<Cell<bool>>,
) {
    let generation = Rc::new(Cell::new(0u64));
    let copy_feedback_generation = Rc::new(Cell::new(0u64));
    status_bar.history_button.set_visible(false);

    let label_for_copy = status_bar.label.clone();
    let window_for_copy = window.clone();
    let copy_button_for_copy = status_bar.copy_button.clone();
    let copy_feedback_for_copy = Rc::clone(&copy_feedback_generation);
    status_bar.copy_button.connect_clicked(move |_| {
        window_for_copy.clipboard().set_text(&label_for_copy.text());
        show_copy_feedback(&copy_button_for_copy, &copy_feedback_for_copy);
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
    status_bar.label.connect_label_notify(move |label| {
        if label.text().is_empty() {
            container_for_label.set_visible(false);
            return;
        }

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
    copy_button: gtk::Button,
    history_button: gtk::Button,
    hide_button: gtk::Button,
) {
    let copy_feedback_generation = Rc::new(Cell::new(0u64));
    let ui_for_status_copy = Rc::clone(ui);
    let copy_button_for_status_copy = copy_button.clone();
    let copy_feedback_for_status_copy = Rc::clone(&copy_feedback_generation);
    let copy_status_action = gtk::gio::SimpleAction::new("copy-status", None);
    copy_status_action.connect_activate(move |_, _| {
        ui_for_status_copy
            .window
            .clipboard()
            .set_text(&ui_for_status_copy.status.text());
        show_copy_feedback(&copy_button_for_status_copy, &copy_feedback_for_status_copy);
    });
    app.add_action(&copy_status_action);
    copy_button.set_action_name(Some("app.copy-status"));

    let ui_for_history = Rc::clone(ui);
    let history_popover = Rc::new(RefCell::new(None::<gtk::Popover>));
    history_button.connect_clicked(move |button| {
        show_status_history_popover(&ui_for_history, button, &history_popover);
    });

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
    let message = message.trim();
    if message.is_empty() {
        return;
    }

    ui.status_history.borrow_mut().push(StatusLogEntry {
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        message: message.to_string(),
    });
}

fn show_status_history_popover(
    ui: &Rc<UiHandles>,
    button: &gtk::Button,
    active_popover: &Rc<RefCell<Option<gtk::Popover>>>,
) {
    let visible_popover = active_popover
        .borrow()
        .as_ref()
        .filter(|popover| popover.is_visible())
        .cloned();
    if let Some(popover) = visible_popover {
        popover.popdown();
        return;
    }

    let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
    root.set_margin_top(10);
    root.set_margin_bottom(10);
    root.set_margin_start(10);
    root.set_margin_end(10);

    let header = build_status_history_header();
    root.append(&header.stack);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    list.set_selection_mode(gtk::SelectionMode::None);
    list.set_hexpand(true);
    content.append(&list);
    let empty_label = ui::wrapped_label(&tr("No messages found."));
    empty_label.add_css_class("dim-label");
    empty_label.set_visible(false);
    empty_label.set_margin_top(10);
    empty_label.set_margin_bottom(10);
    empty_label.set_margin_start(10);
    empty_label.set_margin_end(10);
    content.append(&empty_label);

    let entries = ui.status_history.borrow().clone();
    let rows = append_status_history_rows(&list, &entries);
    if rows.is_empty() {
        empty_label.set_text(&tr("No messages yet."));
        empty_label.set_visible(true);
    }
    root.append(&ui::compact_popover_scroll(&content));

    let popover = gtk::Popover::builder().autohide(true).build();
    popover.set_child(Some(&root));
    popover.set_parent(button);

    let stack_for_search = header.stack.clone();
    let search_entry_for_button = header.search_entry.clone();
    header.search_button.connect_clicked(move |_| {
        stack_for_search.set_visible_child_name(STATUS_HISTORY_SEARCH_PAGE);
        search_entry_for_button.grab_focus();
    });
    let stack_for_back = header.stack.clone();
    let search_entry_for_back = header.search_entry.clone();
    header.back_button.connect_clicked(move |_| {
        search_entry_for_back.set_text("");
        stack_for_back.set_visible_child_name(STATUS_HISTORY_TITLE_PAGE);
    });
    connect_status_history_search(&header.search_entry, rows, empty_label);

    let active_popover_for_closed = Rc::clone(active_popover);
    popover.connect_closed(move |_| {
        active_popover_for_closed.borrow_mut().take();
    });

    *active_popover.borrow_mut() = Some(popover.clone());
    popover.popup();
}

struct StatusHistoryHeader {
    stack: gtk::Stack,
    search_button: gtk::Button,
    back_button: gtk::Button,
    search_entry: gtk::SearchEntry,
}

fn build_status_history_header() -> StatusHistoryHeader {
    let stack = gtk::Stack::builder()
        .hhomogeneous(false)
        .vhomogeneous(false)
        .build();
    stack.set_hexpand(true);

    let title_header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    title_header.set_hexpand(true);
    let title_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    title_box.set_hexpand(true);
    let title = gtk::Label::new(Some(&tr("Message History")));
    title.add_css_class("heading");
    title.set_selectable(false);
    title.set_xalign(0.0);
    title.set_width_chars(1);
    title.set_max_width_chars(28);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let subtitle = gtk::Label::new(Some(&tr("Recent status messages")));
    subtitle.add_css_class("dim-label");
    subtitle.set_selectable(false);
    subtitle.set_xalign(0.0);
    subtitle.set_width_chars(1);
    subtitle.set_max_width_chars(34);
    subtitle.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title_box.append(&title);
    title_box.append(&subtitle);

    let search_button = ui::icon_button("edit-find-symbolic", "Search messages");
    let title_actions = ui::linked_button_group();
    title_actions.append(&search_button);
    title_header.append(&title_box);
    title_header.append(&title_actions);

    let back_button = ui::icon_button("go-previous-symbolic", "Back");
    let search_entry = gtk::SearchEntry::builder()
        .placeholder_text(tr("Search messages"))
        .hexpand(true)
        .build();
    let search_header = ui::linked_button_group();
    search_header.set_hexpand(true);
    search_header.append(&back_button);
    search_header.append(&search_entry);

    stack.add_named(&title_header, Some(STATUS_HISTORY_TITLE_PAGE));
    stack.add_named(&search_header, Some(STATUS_HISTORY_SEARCH_PAGE));
    stack.set_visible_child_name(STATUS_HISTORY_TITLE_PAGE);

    StatusHistoryHeader {
        stack,
        search_button,
        back_button,
        search_entry,
    }
}

#[derive(Clone)]
struct StatusHistoryRow {
    widget: gtk::Widget,
    keywords: String,
}

fn append_status_history_rows(
    list: &gtk::ListBox,
    entries: &[StatusLogEntry],
) -> Vec<StatusHistoryRow> {
    let mut rows = Vec::with_capacity(entries.len());
    for entry in entries.iter().rev() {
        let row = status_history_row(entry);
        list.append(&row);
        rows.push(StatusHistoryRow {
            widget: row.upcast::<gtk::Widget>(),
            keywords: status_log_keywords(entry),
        });
    }
    rows
}

fn status_history_row(entry: &StatusLogEntry) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::builder()
        .activatable(false)
        .selectable(false)
        .build();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 2);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(10);
    content.set_margin_end(10);

    let message = gtk::Label::new(Some(&entry.message));
    message.set_selectable(false);
    message.set_xalign(0.0);
    message.set_width_chars(1);
    message.set_max_width_chars(34);
    message.set_ellipsize(gtk::pango::EllipsizeMode::End);
    content.append(&message);

    let timestamp = gtk::Label::new(Some(&entry.timestamp));
    timestamp.add_css_class("dim-label");
    timestamp.set_selectable(false);
    timestamp.set_xalign(0.0);
    content.append(&timestamp);

    row.set_child(Some(&content));
    row
}

fn connect_status_history_search(
    search_entry: &gtk::SearchEntry,
    rows: Vec<StatusHistoryRow>,
    empty_label: gtk::Label,
) {
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().trim().to_lowercase();
        let mut visible_count = 0;
        for row in &rows {
            let visible = status_log_matches_keywords(&row.keywords, &query);
            row.widget.set_visible(visible);
            visible_count += usize::from(visible);
        }
        empty_label.set_text(&tr(if query.is_empty() {
            "No messages yet."
        } else {
            "No messages found."
        }));
        empty_label.set_visible(visible_count == 0);
    });
}

fn status_log_keywords(entry: &StatusLogEntry) -> String {
    format!("{} {}", entry.timestamp, entry.message).to_lowercase()
}

fn status_log_matches_keywords(keywords: &str, query: &str) -> bool {
    query.is_empty() || keywords.contains(query)
}

pub(in crate::app) fn schedule_status_autohide_after_loading(ui: &UiHandles) {
    if ui.status.text().is_empty() || !ui.status_bar.is_visible() {
        return;
    }

    let generation = ui.status_generation.get().wrapping_add(1);
    ui.status_generation.set(generation);
    schedule_status_autohide(ui, generation);
}

pub(in crate::app) fn register_page_copy_feedback_button(ui: &UiHandles, button: &gtk::Button) {
    ui.page_copy_buttons.borrow_mut().push(button.clone());
}

pub(in crate::app) fn show_page_copy_feedback(ui: &UiHandles) {
    show_copy_feedback_for_buttons(&ui.page_copy_buttons, &ui.page_copy_feedback_generation);
}

fn show_copy_feedback_for_buttons(
    buttons: &Rc<RefCell<Vec<gtk::Button>>>,
    generation: &Rc<Cell<u64>>,
) {
    let current = generation.get().wrapping_add(1);
    generation.set(current);
    set_copy_feedback_icons(buttons, COPIED_ICON);

    let buttons = Rc::clone(buttons);
    let generation = Rc::clone(generation);
    gtk::glib::timeout_add_seconds_local(COPY_FEEDBACK_SECONDS, move || {
        if generation.get() == current {
            set_copy_feedback_icons(&buttons, COPY_ICON);
        }
        gtk::glib::ControlFlow::Break
    });
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

fn show_copy_feedback(button: &gtk::Button, generation: &Rc<Cell<u64>>) {
    let current = generation.get().wrapping_add(1);
    generation.set(current);
    ui::set_button_icon(button, COPIED_ICON);

    let button = button.clone();
    let generation = Rc::clone(generation);
    gtk::glib::timeout_add_seconds_local(COPY_FEEDBACK_SECONDS, move || {
        if generation.get() == current {
            ui::set_button_icon(&button, COPY_ICON);
        }
        gtk::glib::ControlFlow::Break
    });
}

fn set_copy_feedback_icons(buttons: &Rc<RefCell<Vec<gtk::Button>>>, icon_name: &str) {
    let mut buttons = buttons.borrow_mut();
    buttons.retain(|button| button.root().is_some());
    for button in buttons.iter() {
        ui::set_button_icon(button, icon_name);
    }
}

fn status_button(icon_name: &str, tooltip: &str) -> gtk::Button {
    let button = ui::overlay_icon_button(icon_name, tooltip);
    button.add_css_class("flat");
    button
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_log_search_matches_timestamp_and_message() {
        let entry = StatusLogEntry {
            timestamp: "12:34:56".to_string(),
            message: "CSV import finished".to_string(),
        };
        let keywords = status_log_keywords(&entry);

        assert!(status_log_matches_keywords(&keywords, "12:34"));
        assert!(status_log_matches_keywords(&keywords, "import"));
        assert!(status_log_matches_keywords(&keywords, ""));
        assert!(!status_log_matches_keywords(&keywords, "backup"));
    }
}
