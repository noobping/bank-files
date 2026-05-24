use super::*;

pub(in crate::app) fn search_empty_page(title: &str, description: &str) -> adw::StatusPage {
    empty_page("edit-find-symbolic", title, description)
}

pub(in crate::app) fn append_page_header(
    container: &gtk::Box,
    ui_handles: &UiHandles,
    title: &str,
    subtitle: &str,
    _page_text: String,
    transactions: &[Transaction],
) {
    ui_handles.mobile_header_title.set_title(&tr(title));
    ui_handles.mobile_header_title.set_subtitle(&tr(subtitle));
    let copy_button = ui::icon_button("edit-copy-symbolic", "Copy this page");
    copy_button.set_action_name(Some("app.copy-page"));
    register_page_copy_feedback_button(ui_handles, &copy_button);

    let print_button = ui::icon_button("document-print-symbolic", "Print this page");
    print_button.set_action_name(Some("app.print-page"));

    let export_button = ui::icon_button("document-save-symbolic", "Export CSV");
    export_button.set_sensitive(transactions.iter().any(|tx| !transaction_is_fake(tx)));
    export_button.set_action_name(Some("app.export-csv"));

    let actions = ui::linked_button_group();
    actions.append(&copy_button);
    actions.append(&print_button);
    actions.append(&export_button);

    let page_header = ui::section_title_with_action(title, subtitle, &actions);
    ui_handles
        .mobile_header_title
        .bind_property("visible", &page_header, "visible")
        .sync_create()
        .invert_boolean()
        .build();
    container.append(&page_header);
}

#[derive(Clone)]
pub(in crate::app) struct PageSnapshot {
    pub(in crate::app) text: String,
    pub(in crate::app) transactions: Vec<Transaction>,
}

pub(in crate::app) fn current_page_snapshot(data: &AppData, ui: &UiHandles) -> PageSnapshot {
    page_snapshot(data, ui, true)
}

pub(in crate::app) fn current_real_page_snapshot(data: &AppData, ui: &UiHandles) -> PageSnapshot {
    page_snapshot(data, ui, false)
}

fn page_snapshot(data: &AppData, ui: &UiHandles, include_fake: bool) -> PageSnapshot {
    let runtime_data;
    let data = if include_fake {
        runtime_data = data_with_fake_transactions(data.clone(), ui.fake_transactions.list());
        &runtime_data
    } else {
        data
    };
    let visible = filtered_app_data(data, ui);
    let visible_data = visible.as_ref().unwrap_or(data);
    let transactions = if include_fake {
        visible_data.transactions.clone()
    } else {
        real_transactions(&visible_data.transactions)
    };
    match ui.stack.visible_child_name().as_deref() {
        Some("overview") => PageSnapshot {
            text: summary::render_overview(visible_data),
            transactions: transactions.clone(),
        },
        Some("transactions") => PageSnapshot {
            text: summary::render_transactions(visible_data),
            transactions: transactions.clone(),
        },
        Some("debug") => PageSnapshot {
            text: summary::render_debug(visible_data),
            transactions: transactions.clone(),
        },
        _ => {
            let text = selected_budget_month(visible_data, ui)
                .map(|month| summary::render_categories_for_month(visible_data, month))
                .unwrap_or_else(|| summary::render_categories(visible_data));
            PageSnapshot { text, transactions }
        }
    }
}
