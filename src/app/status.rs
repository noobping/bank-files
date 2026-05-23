use super::*;

const STATUS_AUTOHIDE_SECONDS: u32 = 6;
const COPY_FEEDBACK_SECONDS: u32 = 3;
const COPY_ICON: &str = "edit-copy-symbolic";
const COPIED_ICON: &str = "object-select-symbolic";
const SAVE_ICON: &str = "document-save-symbolic";
const SAVE_ERROR_ICON: &str = "dialog-error-symbolic";
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
    pub(in crate::app) history_button: gtk::Button,
    pub(in crate::app) search_preset_button: gtk::MenuButton,
    pub(in crate::app) page_actions_button: gtk::MenuButton,
    pub(in crate::app) hide_button: gtk::Button,
}

#[derive(Clone)]
pub(in crate::app) struct StatusHandle {
    icon: gtk::Image,
    spinner: adw::Spinner,
    label: gtk::Label,
}

impl StatusHandle {
    pub(in crate::app) fn from_status_bar(status_bar: &StatusBar) -> Self {
        Self {
            icon: status_bar.icon.clone(),
            spinner: status_bar.spinner.clone(),
            label: status_bar.label.clone(),
        }
    }

    pub(in crate::app) fn set_text(&self, message: &str) {
        self.label.set_text(message);
    }

    pub(in crate::app) fn set_loading(&self, loading: bool) {
        self.icon.set_visible(!loading);
        self.spinner.set_visible(loading);
    }
}

pub(in crate::app) fn build_status_bar() -> StatusBar {
    let builder = ui::builder_from_resource("status-bar.ui");
    let container = builder
        .object::<gtk::Box>("status_bar")
        .expect("status-bar.ui should define status_bar");
    let status_icon = builder
        .object::<gtk::Image>("status_icon")
        .expect("status-bar.ui should define status_icon");
    let spinner = builder
        .object::<adw::Spinner>("status_spinner")
        .expect("status-bar.ui should define status_spinner");
    let label = builder
        .object::<gtk::Label>("status_label")
        .expect("status-bar.ui should define status_label");
    let action_group = builder
        .object::<gtk::Box>("status_action_group")
        .expect("status-bar.ui should define status_action_group");
    let history_button = builder
        .object::<gtk::Button>("status_history_button")
        .expect("status-bar.ui should define status_history_button");
    history_button.set_tooltip_text(Some(&tr("Show message history")));
    let search_preset_button = builder
        .object::<gtk::MenuButton>("status_search_preset_button")
        .expect("status-bar.ui should define status_search_preset_button");
    set_search_preset_menu_model(&search_preset_button);
    let page_actions_button = builder
        .object::<gtk::MenuButton>("status_page_actions_button")
        .expect("status-bar.ui should define status_page_actions_button");
    set_page_actions_menu_namespace(&page_actions_button, "app");
    let hide_button = builder
        .object::<gtk::Button>("status_hide_button")
        .expect("status-bar.ui should define status_hide_button");
    hide_button.set_tooltip_text(Some(&tr("Hide message")));

    StatusBar {
        container,
        icon: status_icon,
        spinner,
        label,
        action_group,
        history_button,
        search_preset_button,
        page_actions_button,
        hide_button,
    }
}

fn set_search_preset_menu_model(menu_button: &gtk::MenuButton) {
    menu_button.set_tooltip_text(Some(&tr("Search filters")));

    let menu = gtk::gio::Menu::new();
    for section in [
        SearchPresetSection::General,
        SearchPresetSection::Transactions,
        SearchPresetSection::Diagnostics,
    ] {
        let section_menu = gtk::gio::Menu::new();
        for preset in search_preset_specs()
            .iter()
            .filter(|preset| preset.section == section)
        {
            append_search_preset(&section_menu, preset.label, preset.id);
        }
        let label = section.label().map(tr);
        menu.append_section(label.as_deref(), &section_menu);
    }

    menu_button.set_menu_model(Some(&menu));
}

fn append_search_preset(menu: &gtk::gio::Menu, label: &str, preset: &str) {
    let item = gtk::gio::MenuItem::new(Some(&tr(label)), Some(SEARCH_PRESET_DETAILED_ACTION));
    item.set_attribute_value("target", Some(&preset.to_variant()));
    menu.append_item(&item);
}

fn set_page_actions_menu_namespace(menu_button: &gtk::MenuButton, action_namespace: &str) {
    menu_button.set_tooltip_text(Some(&tr("Page actions")));

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) struct StaticPageSnapshot {
    key: String,
    title: String,
    subtitle: String,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl StaticPageSnapshot {
    pub(in crate::app) fn new(
        key: &str,
        title: &str,
        subtitle: &str,
        columns: &[&str],
        rows: Vec<Vec<String>>,
    ) -> Self {
        Self {
            key: key.to_string(),
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            columns: columns.iter().map(|column| (*column).to_string()).collect(),
            rows,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) struct PageActionSnapshot {
    key: String,
    title: String,
    subtitle: String,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    text: String,
    csv: String,
}

impl PageActionSnapshot {
    pub(in crate::app) fn from_rows(
        key: &str,
        title: &str,
        subtitle: &str,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    ) -> anyhow::Result<Self> {
        let text = page_action_text(title, subtitle, &columns, &rows);
        let csv = page_action_csv(&columns, &rows)?;
        Ok(Self {
            key: key.to_string(),
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            columns,
            rows,
            text,
            csv,
        })
    }

    pub(in crate::app) fn from_csv(
        key: &str,
        title: &str,
        subtitle: &str,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
        csv: String,
    ) -> Self {
        let text = page_action_text(title, subtitle, &columns, &rows);
        Self {
            key: key.to_string(),
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            columns,
            rows,
            text,
            csv,
        }
    }

    fn from_static(snapshot: StaticPageSnapshot) -> anyhow::Result<Self> {
        Self::from_rows(
            &snapshot.key,
            &snapshot.title,
            &snapshot.subtitle,
            snapshot.columns,
            snapshot.rows,
        )
    }
}

pub(in crate::app) fn connect_static_page_actions(
    page_actions_button: &gtk::MenuButton,
    action_namespace: &str,
    status: &gtk::Label,
    ui_handles: &Rc<UiHandles>,
    snapshot: StaticPageSnapshot,
) {
    connect_page_actions(
        page_actions_button,
        action_namespace,
        status,
        ui_handles,
        move || PageActionSnapshot::from_static(snapshot.clone()),
    );
}

pub(in crate::app) fn connect_page_actions<F>(
    page_actions_button: &gtk::MenuButton,
    action_namespace: &str,
    status: &gtk::Label,
    ui_handles: &Rc<UiHandles>,
    snapshot_provider: F,
) where
    F: Fn() -> anyhow::Result<PageActionSnapshot> + 'static,
{
    set_page_actions_menu_namespace(page_actions_button, action_namespace);
    register_loading_sensitive_widget(ui_handles, page_actions_button);
    let snapshot_provider: Rc<dyn Fn() -> anyhow::Result<PageActionSnapshot>> =
        Rc::new(snapshot_provider);
    let action_group = gtk::gio::SimpleActionGroup::new();

    let snapshot_for_copy = Rc::clone(&snapshot_provider);
    let status_for_copy = status.clone();
    let ui_for_copy = Rc::clone(ui_handles);
    let copy_action = gtk::gio::SimpleAction::new("copy-page", None);
    copy_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        match snapshot_for_copy() {
            Ok(snapshot) => {
                show_verbose_status(
                    ui_for_copy.as_ref(),
                    format!(
                        "page copied; page={}; rows={}",
                        snapshot.key,
                        snapshot.rows.len()
                    ),
                );
                ui_for_copy.window.clipboard().set_text(&snapshot.text);
                status_for_copy.set_text(&trf("Copied {page}.", &[("page", tr(&snapshot.title))]));
            }
            Err(err) => status_for_copy.set_text(&trf(
                "Copy failed: {error}",
                &[("error", format!("{err:#}"))],
            )),
        }
    });
    action_group.add_action(&copy_action);

    let snapshot_for_print = Rc::clone(&snapshot_provider);
    let status_for_print = status.clone();
    let ui_for_print = Rc::clone(ui_handles);
    let print_action = gtk::gio::SimpleAction::new("print-page", None);
    print_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        match snapshot_for_print() {
            Ok(snapshot) => {
                show_verbose_status(
                    ui_for_print.as_ref(),
                    format!(
                        "page print started; page={}; rows={}",
                        snapshot.key,
                        snapshot.rows.len()
                    ),
                );
                status_for_print
                    .set_text(&trf("Printing {page}...", &[("page", tr(&snapshot.title))]));
                let report = table_print_report(
                    &snapshot.title,
                    &snapshot.subtitle,
                    &snapshot.columns,
                    &snapshot.rows,
                );
                print_report(&ui_for_print, report);
            }
            Err(err) => status_for_print.set_text(&trf(
                "Printing failed: {error}",
                &[("error", format!("{err:#}"))],
            )),
        }
    });
    action_group.add_action(&print_action);

    let snapshot_for_export = Rc::clone(&snapshot_provider);
    let status_for_export = status.clone();
    let export_action = gtk::gio::SimpleAction::new("export-csv", None);
    export_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        match snapshot_for_export() {
            Ok(snapshot) => export_page_action_snapshot(action, &status_for_export, snapshot),
            Err(err) => status_for_export.set_text(&trf(
                "Export error: {error}",
                &[("error", format!("{err:#}"))],
            )),
        }
    });
    action_group.add_action(&export_action);

    page_actions_button.insert_action_group(action_namespace, Some(&action_group));
}

fn page_action_text(
    title: &str,
    subtitle: &str,
    columns: &[String],
    rows: &[Vec<String>],
) -> String {
    let mut lines = vec![tr(title), tr(subtitle), String::new()];
    lines.push(
        columns
            .iter()
            .map(|column| tr(column))
            .collect::<Vec<_>>()
            .join("\t"),
    );
    lines.extend(rows.iter().map(|row| {
        row.iter()
            .map(|value| compact_status_cell(value))
            .collect::<Vec<_>>()
            .join("\t")
    }));
    lines.join("\n")
}

fn page_action_csv(columns: &[String], rows: &[Vec<String>]) -> anyhow::Result<String> {
    let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
    let columns = columns.iter().map(|column| tr(column)).collect::<Vec<_>>();
    writer.write_record(columns.iter().map(String::as_str))?;
    for row in rows {
        writer.write_record(row.iter().map(String::as_str))?;
    }
    let bytes = writer.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}

fn compact_status_cell(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn export_page_action_snapshot(
    action: &gtk::gio::SimpleAction,
    status: &gtk::Label,
    snapshot: PageActionSnapshot,
) {
    action.set_enabled(false);
    status.set_text(&tr("Opening the file portal to save the CSV export..."));

    let action = action.clone();
    let status = status.clone();
    gtk::glib::MainContext::default().spawn_local(async move {
        let handle = rfd::AsyncFileDialog::new()
            .set_title(tr("Save CSV export"))
            .add_filter(tr("CSV files"), &["csv"])
            .set_file_name(page_action_export_file_name(&snapshot.key))
            .save_file()
            .await;

        let Some(handle) = handle else {
            action.set_enabled(true);
            status.set_text(&tr("CSV export canceled."));
            return;
        };

        let path = handle.path().to_path_buf();
        let contents = snapshot.csv;
        status.set_text(&tr("Saving CSV export..."));
        let task = gtk::gio::spawn_blocking(move || {
            std::fs::write(&path, contents)?;
            anyhow::Ok(path)
        });
        match task.await {
            Ok(Ok(path)) => status.set_text(&trf(
                "Export saved: {path}",
                &[("path", path.display().to_string())],
            )),
            Ok(Err(err)) => status.set_text(&trf(
                "Export error: {error}",
                &[("error", format!("{err:#}"))],
            )),
            Err(_) => status.set_text(&tr(
                "CSV export canceled: the background task stopped unexpectedly.",
            )),
        }
        action.set_enabled(true);
    });
}

fn page_action_export_file_name(key: &str) -> String {
    format!(
        "bank_files_{key}_{}.csv",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    )
}

pub(in crate::app) fn connect_embedded_status_bar(
    window: &adw::ApplicationWindow,
    status_bar: &StatusBar,
    status_autohide: Rc<Cell<bool>>,
) {
    let generation = Rc::new(Cell::new(0u64));
    let history = Rc::new(RefCell::new(Vec::<StatusLogEntry>::new()));
    let history_popover = Rc::new(RefCell::new(None::<gtk::Popover>));

    let window_for_history = window.clone();
    let history_for_button = Rc::clone(&history);
    let history_popover_for_button = Rc::clone(&history_popover);
    status_bar.history_button.connect_clicked(move |button| {
        show_status_history_popover(
            &window_for_history,
            &history_for_button,
            button,
            &history_popover_for_button,
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
    let window_for_history = ui.window.clone();
    let history_for_history = Rc::clone(&ui.status_history);
    let history_popover = Rc::new(RefCell::new(None::<gtk::Popover>));
    history_button.connect_clicked(move |button| {
        show_status_history_popover(
            &window_for_history,
            &history_for_history,
            button,
            &history_popover,
        );
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

pub(in crate::app) fn show_verbose_status(ui: &UiHandles, message: impl AsRef<str>) {
    #[cfg(debug_assertions)]
    {
        let message = message.as_ref().trim();
        if !message.is_empty() {
            show_status(ui, &format!("[debug] {message}"));
        }
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = ui;
        let _ = message;
    }
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

fn show_status_history_popover(
    window: &adw::ApplicationWindow,
    history: &Rc<RefCell<Vec<StatusLogEntry>>>,
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

    let root = ui::compact_popover_root();

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

    let entries = history.borrow().clone();
    let has_entries = !entries.is_empty();
    header.copy_button.set_sensitive(has_entries);
    header.save_button.set_sensitive(has_entries);
    let rows = append_status_history_rows(window, &list, &entries);
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
        show_status_history_search(&stack_for_search, &search_entry_for_button);
    });

    let window_for_copy = window.clone();
    let entries_for_copy = entries.clone();
    let copy_button_for_copy = header.copy_button.clone();
    let copy_feedback_generation = Rc::new(Cell::new(0u64));
    header.copy_button.connect_clicked(move |_| {
        window_for_copy
            .clipboard()
            .set_text(&status_log_text(&entries_for_copy));
        show_copy_feedback(&copy_button_for_copy, &copy_feedback_generation);
    });
    connect_status_history_save(&header.save_button, entries);
    let stack_for_back = header.stack.clone();
    let search_entry_for_back = header.search_entry.clone();
    header.back_button.connect_clicked(move |_| {
        hide_status_history_search(&stack_for_back, &search_entry_for_back);
    });
    let stack_for_stop = header.stack.clone();
    let search_entry_for_stop = header.search_entry.clone();
    header.search_entry.connect_stop_search(move |_| {
        hide_status_history_search(&stack_for_stop, &search_entry_for_stop);
    });
    let stack_for_shortcut = header.stack.clone();
    let search_entry_for_shortcut = header.search_entry.clone();
    ui::connect_primary_f_shortcut(&root, move || {
        toggle_status_history_search(&stack_for_shortcut, &search_entry_for_shortcut);
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
    copy_button: gtk::Button,
    save_button: gtk::Button,
    back_button: gtk::Button,
    search_entry: gtk::SearchEntry,
}

fn build_status_history_header() -> StatusHistoryHeader {
    let builder = ui::builder_from_resource("status-history-popover.ui");
    let stack = gtk::Stack::builder()
        .hhomogeneous(false)
        .vhomogeneous(false)
        .hexpand(true)
        .build();
    let title_header = builder
        .object::<gtk::Box>("status_history_title_header")
        .expect("status-history-popover.ui should define status_history_title_header");
    let search_header = builder
        .object::<gtk::Box>("status_history_search_header")
        .expect("status-history-popover.ui should define status_history_search_header");
    let title = builder
        .object::<gtk::Label>("status_history_title")
        .expect("status-history-popover.ui should define status_history_title");
    title.set_text(&tr("Message History"));
    let subtitle = builder
        .object::<gtk::Label>("status_history_subtitle")
        .expect("status-history-popover.ui should define status_history_subtitle");
    subtitle.set_text(&tr("Recent status messages"));

    let search_button = builder
        .object::<gtk::Button>("status_history_search_button")
        .expect("status-history-popover.ui should define status_history_search_button");
    search_button.set_tooltip_text(Some(&tr("Search messages")));
    let copy_button = builder
        .object::<gtk::Button>("status_history_copy_button")
        .expect("status-history-popover.ui should define status_history_copy_button");
    copy_button.set_tooltip_text(Some(&tr("Copy message history")));
    let save_button = builder
        .object::<gtk::Button>("status_history_save_button")
        .expect("status-history-popover.ui should define status_history_save_button");
    save_button.set_tooltip_text(Some(&tr("Save message history")));
    let back_button = builder
        .object::<gtk::Button>("status_history_back_button")
        .expect("status-history-popover.ui should define status_history_back_button");
    back_button.set_tooltip_text(Some(&tr("Back")));
    let search_entry = builder
        .object::<gtk::SearchEntry>("status_history_search_entry")
        .expect("status-history-popover.ui should define status_history_search_entry");
    search_entry.set_placeholder_text(Some(&tr("Search messages")));
    stack.add_named(&title_header, Some(STATUS_HISTORY_TITLE_PAGE));
    stack.add_named(&search_header, Some(STATUS_HISTORY_SEARCH_PAGE));
    stack.set_visible_child_name(STATUS_HISTORY_TITLE_PAGE);

    StatusHistoryHeader {
        stack,
        search_button,
        copy_button,
        save_button,
        back_button,
        search_entry,
    }
}

fn show_status_history_search(stack: &gtk::Stack, search_entry: &gtk::SearchEntry) {
    stack.set_visible_child_name(STATUS_HISTORY_SEARCH_PAGE);
    search_entry.grab_focus();
    search_entry.select_region(0, -1);
}

fn hide_status_history_search(stack: &gtk::Stack, search_entry: &gtk::SearchEntry) {
    search_entry.set_text("");
    stack.set_visible_child_name(STATUS_HISTORY_TITLE_PAGE);
}

fn toggle_status_history_search(stack: &gtk::Stack, search_entry: &gtk::SearchEntry) {
    if stack.visible_child_name().as_deref() == Some(STATUS_HISTORY_SEARCH_PAGE) {
        hide_status_history_search(stack, search_entry);
    } else {
        show_status_history_search(stack, search_entry);
    }
}

fn status_log_text(entries: &[StatusLogEntry]) -> String {
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

fn status_log_file_name() -> String {
    format!(
        "bank_files_messages_{}.log",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    )
}

fn connect_status_history_save(save_button: &gtk::Button, entries: Vec<StatusLogEntry>) {
    let save_button_for_save = save_button.clone();
    let save_feedback_generation = Rc::new(Cell::new(0u64));
    save_button.connect_clicked(move |_| {
        if entries.is_empty() || !save_button_for_save.is_sensitive() {
            return;
        }

        save_button_for_save.set_sensitive(false);
        let entries = entries.clone();
        let save_button = save_button_for_save.clone();
        let feedback_generation = Rc::clone(&save_feedback_generation);
        gtk::glib::MainContext::default().spawn_local(async move {
            let handle = rfd::AsyncFileDialog::new()
                .set_title(tr("Save Message History"))
                .add_filter(tr("Log files"), &["log", "txt"])
                .set_file_name(status_log_file_name())
                .save_file()
                .await;

            let Some(handle) = handle else {
                save_button.set_sensitive(true);
                return;
            };

            let path = handle.path().to_path_buf();
            let contents = status_log_text(&entries);
            let task = gtk::gio::spawn_blocking(move || {
                std::fs::write(&path, contents)?;
                anyhow::Ok(())
            });
            match task.await {
                Ok(Ok(())) => {
                    show_icon_feedback(&save_button, &feedback_generation, COPIED_ICON, SAVE_ICON);
                }
                Ok(Err(_)) | Err(_) => {
                    show_icon_feedback(
                        &save_button,
                        &feedback_generation,
                        SAVE_ERROR_ICON,
                        SAVE_ICON,
                    );
                }
            }
            save_button.set_sensitive(true);
        });
    });
}

#[derive(Clone)]
struct StatusHistoryRow {
    widget: gtk::Widget,
    keywords: String,
}

fn append_status_history_rows(
    window: &adw::ApplicationWindow,
    list: &gtk::ListBox,
    entries: &[StatusLogEntry],
) -> Vec<StatusHistoryRow> {
    let mut rows = Vec::with_capacity(entries.len());
    for entry in entries.iter().rev() {
        let row = status_history_row(window, entry);
        list.append(&row);
        rows.push(StatusHistoryRow {
            widget: row.upcast::<gtk::Widget>(),
            keywords: status_log_keywords(entry),
        });
    }
    rows
}

fn status_history_row(window: &adw::ApplicationWindow, entry: &StatusLogEntry) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::builder()
        .activatable(false)
        .selectable(false)
        .build();
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(10);
    content.set_margin_end(10);

    let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text_box.set_hexpand(true);
    let message = gtk::Label::new(Some(&entry.message));
    message.set_selectable(false);
    message.set_xalign(0.0);
    message.set_width_chars(1);
    message.set_max_width_chars(34);
    message.set_ellipsize(gtk::pango::EllipsizeMode::End);
    text_box.append(&message);

    let timestamp = gtk::Label::new(Some(&entry.timestamp));
    timestamp.add_css_class("dim-label");
    timestamp.set_selectable(false);
    timestamp.set_xalign(0.0);
    text_box.append(&timestamp);
    content.append(&text_box);

    let copy_button = status_button(COPY_ICON, "Copy message");
    copy_button.set_valign(gtk::Align::Center);
    let window_for_copy = window.clone();
    let message_for_copy = entry.message.clone();
    let copy_button_for_copy = copy_button.clone();
    let copy_feedback_generation = Rc::new(Cell::new(0u64));
    copy_button.connect_clicked(move |_| {
        window_for_copy.clipboard().set_text(&message_for_copy);
        show_copy_feedback(&copy_button_for_copy, &copy_feedback_generation);
    });
    content.append(&copy_button);

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
    query.split_whitespace().all(|term| {
        if let Some(excluded) = term.strip_prefix('!') {
            excluded.is_empty() || !keywords.contains(excluded)
        } else {
            keywords.contains(term)
        }
    })
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
    show_icon_feedback(button, generation, COPIED_ICON, COPY_ICON);
}

fn show_icon_feedback(
    button: &gtk::Button,
    generation: &Rc<Cell<u64>>,
    feedback_icon: &str,
    restore_icon: &str,
) {
    let current = generation.get().wrapping_add(1);
    generation.set(current);
    ui::set_button_icon(button, feedback_icon);

    let button = button.clone();
    let generation = Rc::clone(generation);
    let restore_icon = restore_icon.to_string();
    gtk::glib::timeout_add_seconds_local(COPY_FEEDBACK_SECONDS, move || {
        if generation.get() == current {
            ui::set_button_icon(&button, &restore_icon);
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

    #[test]
    fn status_log_search_supports_negative_terms() {
        let debug_entry = StatusLogEntry {
            timestamp: "12:34:56".to_string(),
            message: "[debug] render started".to_string(),
        };
        let normal_entry = StatusLogEntry {
            timestamp: "12:35:00".to_string(),
            message: "CSV import finished".to_string(),
        };
        let debug_keywords = status_log_keywords(&debug_entry);
        let normal_keywords = status_log_keywords(&normal_entry);

        assert!(!status_log_matches_keywords(&debug_keywords, "!debug"));
        assert!(status_log_matches_keywords(&normal_keywords, "!debug"));
        assert!(status_log_matches_keywords(&normal_keywords, "csv !debug"));
        assert!(!status_log_matches_keywords(
            &debug_keywords,
            "render !debug"
        ));
        assert!(status_log_matches_keywords(&debug_keywords, "!"));
    }

    #[test]
    fn status_log_text_includes_all_entries_and_sanitizes_messages() {
        let entries = vec![
            StatusLogEntry {
                timestamp: "12:34:56".to_string(),
                message: "CSV import finished".to_string(),
            },
            StatusLogEntry {
                timestamp: "12:35:00".to_string(),
                message: "Line one\nline two".to_string(),
            },
        ];

        assert_eq!(
            status_log_text(&entries),
            "12:34:56\tCSV import finished\n12:35:00\tLine one line two"
        );
    }

    #[test]
    fn page_action_snapshot_builds_copy_text_and_csv() {
        let snapshot = PageActionSnapshot::from_rows(
            "sample",
            "Sample Page",
            "Rows visible on the sample page.",
            vec!["Name".to_string(), "Notes".to_string()],
            vec![vec![
                "Groceries".to_string(),
                "Line one
line two"
                    .to_string(),
            ]],
        )
        .expect("snapshot should serialize to CSV");

        assert_eq!(
            snapshot.text,
            [
                tr("Sample Page"),
                tr("Rows visible on the sample page."),
                String::new(),
                format!("{}	{}", tr("Name"), tr("Notes")),
                "Groceries	Line one line two".to_string(),
            ]
            .join("\n")
        );
        assert!(snapshot.csv.contains("Name"));
        assert!(snapshot.csv.contains("Groceries"));
    }
}
