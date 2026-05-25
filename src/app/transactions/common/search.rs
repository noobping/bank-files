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
