use super::*;

mod budget;
mod diagnostics;
mod overview;
mod table;
mod transactions;

pub(in crate::app) use table::table_print_report;

pub(in crate::app) fn current_print_report(data: &AppData, ui: &UiHandles) -> PrintReport {
    let runtime_data = data_with_fake_transactions(data.clone(), ui.fake_transactions.list());
    let filtered = filtered_app_data(&runtime_data, ui);
    let print_data = filtered.as_ref().unwrap_or(&runtime_data);
    let search = active_search(ui);
    match ui.stack.visible_child_name().as_deref() {
        Some("overview") => overview::overview_print_report(print_data, ui, search.as_ref()),
        Some("transactions") => {
            transactions::transactions_print_report(print_data, search.as_ref())
        }
        Some("debug") => diagnostics::diagnostics_print_report(print_data, search.as_ref()),
        _ => budget::budget_print_report(print_data, ui, search.as_ref()),
    }
}

pub(super) fn print_subtitle(default: &str, search: Option<&SearchFilter>) -> String {
    search
        .map(|filter| {
            trf(
                "Filtered by “{query}”. {description}",
                &[
                    ("query", filter.raw.clone()),
                    ("description", default.to_string()),
                ],
            )
        })
        .unwrap_or_else(|| default.to_string())
}
