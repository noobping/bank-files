use super::history_rows::{append_status_history_rows, connect_status_history_search};
use super::page_actions::compact_status_cell;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) struct StatusLogEntry {
    pub(in crate::app) timestamp: String,
    pub(in crate::app) message: String,
}

pub(super) fn show_status_history_dialog(
    window: &adw::ApplicationWindow,
    history: &Rc<RefCell<Vec<StatusLogEntry>>>,
    active_dialog: &Rc<RefCell<Option<adw::Dialog>>>,
    print_ui: Option<Rc<UiHandles>>,
) {
    let visible_dialog = active_dialog
        .borrow()
        .as_ref()
        .filter(|dialog| dialog.is_visible())
        .cloned();
    if let Some(dialog) = visible_dialog {
        dialog.close();
        return;
    }

    let shell = build_settings_dialog_shell("Message History", "Search messages");
    let root = shell.root;
    let search_bar = shell.search_bar;
    let search_entry = shell.search_entry;

    let actions_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text(tr("Menu"))
        .build();
    actions_button.set_focus_on_click(false);
    shell.header.pack_end(&actions_button);

    let builder = ui::builder_from_resource("status-history-dialog.ui");
    let content = status_history_object::<gtk::Box>(&builder, "status_history_content");
    let list = status_history_object::<gtk::ListBox>(&builder, "status_history_list");
    let empty_label = status_history_object::<gtk::Label>(&builder, "status_history_empty_label");
    empty_label.set_text(&tr("No messages found."));

    let entries = history.borrow().clone();
    let has_entries = !entries.is_empty();
    actions_button.set_sensitive(has_entries);
    let rows = append_status_history_rows(window, &list, &entries);
    if rows.is_empty() {
        empty_label.set_text(&tr("No messages yet."));
        empty_label.set_visible(true);
    }

    connect_status_history_actions(&actions_button, window, entries, print_ui);
    connect_status_history_search(&search_entry, rows, empty_label);

    root.append(&ui::action_dialog_scroll_with_min(&content, 360));
    let dialog = ui::content_dialog(tr("Message History"), &root)
        .content_width(620)
        .content_height(560)
        .build();
    ui::bind_search_bar(&dialog, &dialog, &search_bar, &search_entry);

    let active_dialog_for_closed = Rc::clone(active_dialog);
    dialog.connect_closed(move |_| {
        active_dialog_for_closed.borrow_mut().take();
    });

    *active_dialog.borrow_mut() = Some(dialog.clone());
    dialog.present(Some(window));
}

fn status_history_object<T: IsA<gtk::glib::Object>>(builder: &gtk::Builder, id: &str) -> T {
    ui::builder_object(builder, id, "status-history-dialog.ui")
}

fn connect_status_history_actions(
    actions_button: &gtk::MenuButton,
    window: &adw::ApplicationWindow,
    entries: Vec<StatusLogEntry>,
    print_ui: Option<Rc<UiHandles>>,
) {
    let menu = gtk::gio::Menu::new();
    menu.append(Some(&tr("Print")), Some("status-history.print"));
    menu.append(Some(&tr("Copy")), Some("status-history.copy"));
    menu.append(Some(&tr("Save")), Some("status-history.save"));
    actions_button.set_menu_model(Some(&menu));

    let action_group = gtk::gio::SimpleActionGroup::new();
    let has_entries = !entries.is_empty();

    let entries_for_print = entries.clone();
    let print_action = gtk::gio::SimpleAction::new("print", None);
    print_action.set_enabled(has_entries && print_ui.is_some());
    print_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        let Some(ui) = print_ui.as_ref() else {
            return;
        };
        let report = table_print_report(
            "Message History",
            "Recent status messages",
            &status_log_columns(),
            &status_log_rows(&entries_for_print),
        );
        print_report(ui, report);
    });
    action_group.add_action(&print_action);

    let window_for_copy = window.clone();
    let entries_for_copy = entries.clone();
    let copy_action = gtk::gio::SimpleAction::new("copy", None);
    copy_action.set_enabled(has_entries);
    copy_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        window_for_copy
            .clipboard()
            .set_text(&status_log_text(&entries_for_copy));
    });
    action_group.add_action(&copy_action);

    let entries_for_save = entries;
    let save_action = gtk::gio::SimpleAction::new("save", None);
    save_action.set_enabled(has_entries);
    save_action.connect_activate(move |action, _| {
        save_status_history_entries(action, entries_for_save.clone());
    });
    action_group.add_action(&save_action);

    actions_button.insert_action_group("status-history", Some(&action_group));
}

pub(super) fn status_log_text(entries: &[StatusLogEntry]) -> String {
    entries
        .iter()
        .map(|entry| {
            format!(
                "{}\t{}",
                entry.timestamp,
                compact_status_cell(&entry.message)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn status_log_columns() -> Vec<String> {
    vec![tr("Time"), tr("Message")]
}

pub(super) fn status_log_rows(entries: &[StatusLogEntry]) -> Vec<Vec<String>> {
    entries
        .iter()
        .map(|entry| vec![entry.timestamp.clone(), compact_status_cell(&entry.message)])
        .collect()
}

fn status_log_file_name() -> String {
    format!(
        "bank_files_messages_{}.log",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    )
}

fn save_status_history_entries(action: &gtk::gio::SimpleAction, entries: Vec<StatusLogEntry>) {
    if entries.is_empty() || !action.is_enabled() {
        return;
    }

    action.set_enabled(false);

    let action = action.clone();
    gtk::glib::MainContext::default().spawn_local(async move {
        let handle = rfd::AsyncFileDialog::new()
            .set_title(tr("Save Message History"))
            .add_filter(tr("Log files"), &["log", "txt"])
            .set_file_name(status_log_file_name())
            .save_file()
            .await;

        let Some(handle) = handle else {
            action.set_enabled(true);
            return;
        };

        let path = handle.path().to_path_buf();
        let contents = status_log_text(&entries);
        let task = gtk::gio::spawn_blocking(move || {
            std::fs::write(&path, contents)?;
            anyhow::Ok(())
        });
        let _ = task.await;
        action.set_enabled(true);
    });
}
