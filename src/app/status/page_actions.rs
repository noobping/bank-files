use super::bar::set_page_actions_menu_namespace;
use super::snapshot::{PageActionSnapshot, StaticPageSnapshot};
use super::*;

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

pub(super) fn page_action_text(
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

pub(super) fn page_action_csv(columns: &[String], rows: &[Vec<String>]) -> anyhow::Result<String> {
    let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
    let columns = columns.iter().map(|column| tr(column)).collect::<Vec<_>>();
    writer.write_record(columns.iter().map(String::as_str))?;
    for row in rows {
        writer.write_record(row.iter().map(String::as_str))?;
    }
    let bytes = writer.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}

pub(super) fn compact_status_cell(value: &str) -> String {
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
