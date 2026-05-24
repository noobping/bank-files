use super::*;

pub(super) fn append_diagnostics_metrics(
    data: &AppData,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let rows_seen: usize = data.reports.iter().map(|report| report.rows_seen).sum();
    let rows_imported: usize = data.reports.iter().map(|report| report.rows_imported).sum();
    let rows_skipped: usize = data.reports.iter().map(|report| report.rows_skipped).sum();
    let unconfigured_budget_count =
        analytics::unconfigured_expense_budget_count(&data.transactions, &data.budgets);
    let other_category_count = analytics::other_category_count(&data.transactions);
    let csv_status = if data.reports.is_empty() {
        tr("No CSV files opened. Choose CSV files or drop bank files onto the window.")
    } else {
        let names = data
            .reports
            .iter()
            .filter_map(|report| report.source.file_name())
            .map(|name| name.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        trf(
            "{count} open CSV file(s): {files}",
            &[
                ("count", data.reports.len().to_string()),
                ("files", truncate(&names, 160)),
            ],
        )
    };
    let rows_status = trf(
        "{seen} rows seen. {imported} imported, {skipped} skipped.",
        &[
            ("seen", rows_seen.to_string()),
            ("imported", rows_imported.to_string()),
            ("skipped", rows_skipped.to_string()),
        ],
    );
    let ui_for_csv = Rc::clone(ui_handles);
    let ui_for_rows = Rc::clone(ui_handles);
    let state_for_imported = Rc::clone(state);
    let ui_for_imported = Rc::clone(ui_handles);
    let state_for_unconfigured = Rc::clone(state);
    let ui_for_unconfigured = Rc::clone(ui_handles);
    let state_for_other = Rc::clone(state);
    let ui_for_other = Rc::clone(ui_handles);
    let app_for_duplicates = ui_handles.window.application();
    ui_handles.debug.append(&ui::metric_grid(
        vec![
            ui::activatable_metric_card(
                "CSV files",
                &data.reports.len().to_string(),
                "Stored",
                move || show_status(&ui_for_csv, &csv_status),
            ),
            ui::activatable_metric_card(
                "Rows seen",
                &rows_seen.to_string(),
                "For import checks",
                move || show_status(&ui_for_rows, &rows_status),
            ),
            ui::activatable_metric_card(
                "Imported",
                &rows_imported.to_string(),
                &trf("{count} skipped", &[("count", rows_skipped.to_string())]),
                move || {
                    show_transactions_filter(
                        &state_for_imported,
                        &ui_for_imported,
                        TransactionFilter::all(),
                    );
                },
            ),
            duplicate_filtering_card(data, move || {
                if let Some(action) = app_for_duplicates
                    .as_ref()
                    .and_then(|app| app.lookup_action("dedupe-enabled"))
                {
                    let enabled = action
                        .state()
                        .and_then(|state| state.get::<bool>())
                        .unwrap_or(true);
                    action.change_state(&(!enabled).to_variant());
                }
            }),
            ui::activatable_metric_card(
                "Unconfigured budgets",
                &unconfigured_budget_count.to_string(),
                TransactionFilter::UnconfiguredBudgets.description(),
                move || {
                    show_transactions_filter(
                        &state_for_unconfigured,
                        &ui_for_unconfigured,
                        TransactionFilter::UnconfiguredBudgets,
                    );
                },
            ),
            ui::activatable_metric_card(
                "Other categories",
                &other_category_count.to_string(),
                TransactionFilter::OtherCategories.description(),
                move || {
                    show_transactions_filter(
                        &state_for_other,
                        &ui_for_other,
                        TransactionFilter::OtherCategories,
                    );
                },
            ),
        ],
        4,
    ));
}

fn duplicate_filtering_card<F>(data: &AppData, on_activate: F) -> gtk::Box
where
    F: Fn() + 'static,
{
    let enabled = data.dedupe_mode.is_enabled();
    let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
    card.add_css_class("card");
    card.set_margin_top(4);
    card.set_margin_bottom(4);
    card.set_margin_start(4);
    card.set_margin_end(4);
    card.set_hexpand(true);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 4);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header.set_hexpand(true);

    let title = gtk::Label::new(Some(&tr("Duplicates")));
    title.add_css_class("caption");
    title.set_xalign(0.0);
    title.set_hexpand(true);
    header.append(&title);

    let badge = gtk::Label::new(Some(&tr(if enabled { "Filtered" } else { "Unfiltered" })));
    badge.add_css_class("caption");
    badge.add_css_class(if enabled { "success" } else { "dim-label" });
    badge.set_tooltip_text(Some(&tr(if enabled {
        "Duplicate filtering is on"
    } else {
        "Duplicate filtering is off"
    })));
    badge.set_valign(gtk::Align::Start);
    header.append(&badge);
    content.append(&header);

    let value = gtk::Label::new(Some(&data.duplicate_count.to_string()));
    value.add_css_class("title-2");
    value.set_xalign(0.0);
    content.append(&value);

    let subtitle = ui::wrapped_label(&tr(data.dedupe_mode.description()));
    subtitle.add_css_class("dim-label");
    subtitle.set_width_chars(1);
    subtitle.set_max_width_chars(32);
    content.append(&subtitle);

    card.append(&content);
    ui::activatable_card(card, on_activate)
}
