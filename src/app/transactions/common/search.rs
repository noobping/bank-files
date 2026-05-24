use super::*;

pub(super) fn similar_transaction_query(tx: &Transaction) -> String {
    [
        tx.counterparty.trim(),
        tx.tags.trim(),
        tx.description.trim(),
        tx.budget_code.trim(),
    ]
    .into_iter()
    .find(|value| !value.is_empty())
    .unwrap_or_else(|| tx.transaction_id.trim())
    .to_string()
}

pub(super) fn show_transactions_text_search(
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    query: &str,
    status: &str,
) {
    ui_handles.stack.set_visible_child_name("transactions");
    *ui_handles.active_transaction_filter.borrow_mut() = None;
    *ui_handles.search_query.borrow_mut() = query.to_string();
    ui_handles.search_bar.set_search_mode(true);
    if ui_handles.search_entry.text().as_str() != query {
        ui_handles.search_entry.set_text(query);
    }
    render_views(&state.borrow(), ui_handles, state);
    show_status(ui_handles, status);
}

pub(super) fn show_diagnostics_text_search(
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    query: &str,
) {
    if !smart_pattern_detection_enabled(
        ui_handles.advanced_features.get(),
        ui_handles.show_predictions.get(),
    ) {
        show_status(
            ui_handles,
            "Smart Insights are disabled. Enable Smart Insights to search detected transaction patterns.",
        );
        return;
    }
    ui_handles.stack.set_visible_child_name("debug");
    *ui_handles.active_transaction_filter.borrow_mut() = None;
    *ui_handles.search_query.borrow_mut() = query.to_string();
    ui_handles.search_bar.set_search_mode(true);
    if ui_handles.search_entry.text().as_str() != query {
        ui_handles.search_entry.set_text(query);
    }
    render_views(&state.borrow(), ui_handles, state);
    show_status(ui_handles, "Searching Diagnostics for related patterns.");
}
