use super::*;

mod metrics;
mod orphaned;
mod reports;
mod warnings;

pub(in crate::app) fn render_diagnostics_page(
    data: &AppData,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    ui::clear_box(&ui_handles.debug);
    let search = active_search(ui_handles.as_ref());
    let subtitle = search
        .as_ref()
        .map(|filter| {
            trf(
                "Filter “{query}” searches import settings, CSV files, and warnings.",
                &[("query", filter.raw.clone())],
            )
        })
        .unwrap_or_else(|| {
            "Import quality, detected fields, and warnings. Everything is local and copyable here."
                .to_string()
        });
    append_page_header(
        &ui_handles.debug,
        ui_handles.as_ref(),
        "Diagnostics",
        &subtitle,
        summary::render_debug(data),
        &data.transactions,
    );

    metrics::append_diagnostics_metrics(data, state, ui_handles);

    let mut has_search_results = false;
    if orphaned::append_orphaned_config_section(search.as_ref(), state, ui_handles) {
        has_search_results = true;
    }

    if reports::append_reports_section(data, search.as_ref(), state, ui_handles) {
        has_search_results = true;
    }
    if warnings::append_warnings_section(data, search.as_ref(), ui_handles) {
        has_search_results = true;
    }

    if search.is_some() && !has_search_results {
        ui_handles.debug.append(&search_empty_page(
            "No diagnostic results",
            "No CSV files or warnings match this search term.",
        ));
    }
}
