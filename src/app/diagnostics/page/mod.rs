use super::*;

mod metrics;
mod orphaned;
mod patterns;
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

    let patterns_section_search_matches = search
        .as_ref()
        .map(|filter| patterns::transaction_patterns_section_matches(Some(filter)))
        .unwrap_or(false);
    if patterns::transaction_patterns_section_visible(
        search.as_ref(),
        smart_pattern_detection_enabled(
            ui_handles.advanced_features.get(),
            ui_handles.show_predictions.get(),
        ),
    ) {
        has_search_results = true;
        patterns::append_transaction_patterns_section_async(
            data,
            search.clone(),
            selected_year(data, ui_handles.as_ref()),
            state,
            ui_handles,
        );
    } else if patterns_section_search_matches {
        has_search_results = true;
        patterns::append_transaction_patterns_disabled_section(ui_handles);
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
