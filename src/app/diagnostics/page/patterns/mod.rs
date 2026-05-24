use super::*;

mod data;
mod dialog;
mod rows;

use data::{transaction_patterns_render_data, TransactionPatternsRenderData};
use rows::{
    append_transaction_pattern_rows, append_transaction_patterns_more_button,
    transaction_patterns_hidden_button,
};

pub(super) fn transaction_patterns_section_visible(
    search: Option<&SearchFilter>,
    smart_patterns_enabled: bool,
) -> bool {
    smart_patterns_enabled && transaction_patterns_section_matches(search)
}

pub(super) fn transaction_patterns_section_matches(search: Option<&SearchFilter>) -> bool {
    search
        .map(|filter| {
            filter.matches(
                "Transaction Patterns repeating transactions refunds fully offsetting groups tags categorization rules",
            )
        })
        .unwrap_or(true)
}

pub(super) fn append_transaction_patterns_disabled_section(ui_handles: &Rc<UiHandles>) {
    ui_handles.debug.append(&ui::section_title(
        "Transaction Patterns",
        "Repeating payments, possible transfers, refunds, and fully offsetting groups detected from imported transactions.",
    ));
    ui_handles.debug.append(&ui::text_card(&tr(
        "Smart Insights is disabled. Enable Smart Insights to detect transaction patterns.",
    )));
}

pub(super) fn append_transaction_patterns_section_async(
    data: &AppData,
    search: Option<SearchFilter>,
    selected_year: Option<i32>,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let subtitle = selected_year
        .map(|year| {
            trf(
                "Repeating payments, possible transfers, refunds, and offsetting groups detected in {year}.",
                &[("year", year.to_string())],
            )
        })
        .unwrap_or_else(|| {
            tr("Repeating payments, possible transfers, refunds, and fully offsetting groups detected from imported transactions.")
        });
    ui_handles
        .debug
        .append(&ui::section_title("Transaction Patterns", &subtitle));
    if let Some(year) = selected_year {
        ui_handles.debug.append(&year_selector_row(
            &data.available_years,
            year,
            ui_handles,
            state,
        ));
    }
    let patterns_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    append_transaction_patterns_loading(&patterns_box);
    ui_handles.debug.append(&patterns_box);

    let data = data.clone();
    let mode = data.dedupe_mode;
    let needs_all_reload = transaction_patterns_need_all_reload(data.loaded_scope);
    let fake_transactions = ui_handles.fake_transactions.list();
    let show_all = ui_handles.show_all.get() || search.is_some();
    let smart_insights_enabled = smart_pattern_detection_enabled(
        ui_handles.advanced_features.get(),
        ui_handles.show_predictions.get(),
    );
    let hide_canceled = smart_insights_enabled && ui_handles.hide_canceled_transactions.get();
    let generation = ui_handles.render_generation.get();
    let state_for_patterns = Rc::clone(state);
    let ui_for_patterns = Rc::clone(ui_handles);
    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            let data = if needs_all_reload {
                let (data, _) = crate::data::load_app_data_read_only_aware(
                    mode,
                    false,
                    TransactionLoadScope::All,
                    smart_insights_enabled,
                )?;
                data_with_fake_transactions(data, fake_transactions)
            } else {
                data
            };
            anyhow::Ok(transaction_patterns_render_data(
                data,
                search,
                show_all,
                hide_canceled,
                selected_year,
                smart_insights_enabled,
            ))
        });

        match task.await {
            Ok(Ok(render_data)) => {
                if ui_for_patterns.render_generation.get() != generation
                    || ui_for_patterns.stack.visible_child_name().as_deref() != Some("debug")
                {
                    return;
                }
                ui::clear_box(&patterns_box);
                append_transaction_patterns_result(
                    &patterns_box,
                    render_data,
                    &state_for_patterns,
                    &ui_for_patterns,
                );
            }
            Ok(Err(error)) => {
                if ui_for_patterns.render_generation.get() == generation {
                    ui::clear_box(&patterns_box);
                    patterns_box.append(&ui::text_card(&trf(
                        "Could not detect transaction patterns: {error}",
                        &[("error", format!("{error:#}"))],
                    )));
                }
            }
            Err(_) => {
                if ui_for_patterns.render_generation.get() == generation {
                    ui::clear_box(&patterns_box);
                    patterns_box.append(&ui::text_card(&tr(
                        "Transaction pattern detection stopped unexpectedly.",
                    )));
                }
            }
        }
    });
}

fn transaction_patterns_need_all_reload(scope: TransactionLoadScope) -> bool {
    !matches!(scope, TransactionLoadScope::All)
}

fn append_transaction_patterns_loading(container: &gtk::Box) {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 8);
    card.add_css_class("card");
    card.set_margin_top(4);
    card.set_margin_bottom(4);
    card.set_margin_start(4);
    card.set_margin_end(4);
    card.set_halign(gtk::Align::Fill);
    card.set_hexpand(true);

    let content = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let spinner = ui::loading_spinner();
    content.append(&spinner);

    let text = ui::wrapped_label(&tr("Detecting transaction patterns..."));
    text.add_css_class("dim-label");
    text.set_hexpand(true);
    content.append(&text);

    card.append(&content);
    container.append(&card);
}

fn append_transaction_patterns_result(
    patterns_box: &gtk::Box,
    render_data: TransactionPatternsRenderData,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    if render_data.hidden_count > 0 {
        let message = if render_data.hide_canceled {
            trf(
                "{count} refund or offsetting transactions are hidden from normal views.",
                &[("count", render_data.hidden_count.to_string())],
            )
        } else {
            trf(
                "{count} refund or offsetting transactions were detected. Enable Hide Refunded Transactions to exclude them from normal views.",
                &[("count", render_data.hidden_count.to_string())],
            )
        };
        patterns_box.append(&ui::text_card(&message));
    }

    if render_data.patterns.is_empty() {
        patterns_box.append(&ui::text_card(&tr(
            "No transaction patterns detected yet. Import more transactions or adjust tags and categorization rules.",
        )));
        return;
    }

    let rows_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    if render_data.preview_patterns.is_empty() {
        let message = if render_data.hidden_patterns.len() == render_data.patterns.len() {
            "All detected transaction patterns are hidden. Use Show Hidden to review them."
        } else {
            "All detected transaction patterns are already covered. Use More to review them."
        };
        rows_box.append(&ui::text_card(&tr(message)));
    } else {
        append_transaction_pattern_rows(
            &rows_box,
            &render_data.preview_patterns,
            state,
            ui_handles,
        );
    }
    patterns_box.append(&rows_box);

    let hidden_button = if render_data.hidden_patterns.is_empty() {
        None
    } else {
        let button = transaction_patterns_hidden_button(
            &rows_box,
            render_data.hidden_patterns,
            state,
            ui_handles,
        );
        if render_data.more_patterns.len() > render_data.preview_patterns.len() {
            button.set_visible(false);
        }
        Some(button)
    };

    if render_data.more_patterns.len() > render_data.preview_patterns.len() {
        append_transaction_patterns_more_button(
            patterns_box,
            &rows_box,
            render_data.more_patterns,
            hidden_button.as_ref(),
            state,
            ui_handles,
        );
    }
    if let Some(hidden_button) = hidden_button {
        patterns_box.append(&hidden_button);
    }
}

#[cfg(test)]
mod tests;
