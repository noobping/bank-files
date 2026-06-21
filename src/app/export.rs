use super::*;

pub(in crate::app) fn export_transactions_from_action(
    ui: &UiHandles,
    export_action: &gtk::gio::SimpleAction,
    transactions: &[Transaction],
) {
    if transactions.is_empty() {
        show_status(ui, "No transactions to export.");
        return;
    }

    export_action.set_enabled(false);
    show_status(ui, "Opening the file portal to save the CSV export...");

    let ui = ui.clone();
    let export_action = export_action.clone();
    let transactions = transactions.to_vec();
    let default_name = format!(
        "bank_files_export_{}.csv",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );
    gtk::glib::MainContext::default().spawn_local(async move {
        let handle = rfd::AsyncFileDialog::new()
            .set_title(tr("Save CSV export"))
            .add_filter(tr("CSV files"), &["csv"])
            .set_file_name(default_name)
            .save_file()
            .await;

        let Some(handle) = handle else {
            export_action.set_enabled(true);
            show_status(&ui, "CSV export canceled.");
            return;
        };
        let path = handle.path().to_path_buf();
        show_status(&ui, "Saving CSV export...");
        begin_background_operation(&ui);

        let task = gtk::gio::spawn_blocking(move || {
            data::export_transactions_to_path(&transactions, &path)
        });
        match task.await {
            Ok(Ok(path)) => {
                show_status(
                    &ui,
                    &trf(
                        "Export saved: {path}",
                        &[("path", path.display().to_string())],
                    ),
                );
            }
            Ok(Err(err)) => show_status(
                &ui,
                &trf("Export error: {error}", &[("error", format!("{err:#}"))]),
            ),
            Err(_) => show_status(
                &ui,
                "CSV export canceled: the background task stopped unexpectedly.",
            ),
        }
        finish_background_operation(&ui);
        export_action.set_enabled(true);
    });
}
