use super::super::*;
use super::form::{queue_fake_transaction_update, show_fake_transaction_form};
use super::model::{FakeTransaction, FakeTransactionUpdateOutcome};
use super::presentation::{
    fake_transaction_matches_search, fake_transaction_search_terms, fake_transaction_subtitle,
    fake_transaction_title,
};

pub(super) fn refresh_fake_transactions_ui(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
    let widgets = &ui.fake_transaction_widgets;
    let count = ui.fake_transactions.count();
    let busy = widgets.busy_box.is_visible();
    widgets.badge.set_visible(count > 0);
    widgets.badge.set_text(&count.to_string());
    widgets.button.set_tooltip_text(Some(&trf(
        "Fake transactions: {count}",
        &[("count", count.to_string())],
    )));
    widgets.back_button.set_sensitive(!busy);
    widgets.add_button.set_sensitive(!busy);
    widgets.save_button.set_sensitive(!busy);
    widgets.clear_button.set_visible(count > 0);
    widgets.clear_button.set_sensitive(count > 0 && !busy);
    widgets.stack.set_sensitive(!busy);
    widgets.form_box.set_sensitive(!busy);
    widgets.list.set_sensitive(!busy);
    widgets.summary.set_text(&fake_transaction_summary(count));

    ui::clear_list_box(&widgets.list);
    let fake_transactions = ui.fake_transactions.list();
    if fake_transactions.is_empty() {
        widgets
            .list
            .append(&fake_transaction_text_row(&tr("No fake transactions.")));
        return;
    }

    let search_terms = fake_transaction_search_terms(&widgets.search_entry.text());
    let mut visible_count = 0usize;
    for fake in fake_transactions {
        if fake_transaction_matches_search(&fake, &search_terms) {
            widgets.list.append(&fake_transaction_row(state, ui, fake));
            visible_count += 1;
        }
    }

    if visible_count == 0 {
        widgets.list.append(&fake_transaction_text_row(&tr(
            "No matching fake transactions.",
        )));
    }
}

fn fake_transaction_summary(count: usize) -> String {
    if count == 0 {
        tr("No runtime preview transactions.")
    } else {
        trf(
            "{count} fake transaction(s) affect this session.",
            &[("count", count.to_string())],
        )
    }
}

fn fake_transaction_text_row(text: &str) -> adw::ActionRow {
    ui::text_list_row(text)
}

fn fake_transaction_row(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    fake: FakeTransaction,
) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(fake_transaction_title(&fake.transaction))
        .subtitle(fake_transaction_subtitle(&fake.transaction))
        .build();
    row.set_activatable(false);
    row.set_selectable(false);
    row.set_title_lines(1);
    row.set_subtitle_lines(1);

    let amount = gtk::Label::new(Some(&signed_money(fake.transaction.amount)));
    amount.add_css_class(if fake.transaction.amount >= Decimal::ZERO {
        "success"
    } else {
        "error"
    });
    amount.set_selectable(false);
    amount.set_xalign(1.0);
    row.add_suffix(&amount);

    let actions = ui::linked_button_group();
    actions.set_halign(gtk::Align::End);
    let edit_button = ui::icon_button("document-edit-symbolic", "Edit fake transaction");
    let remove_button = ui::icon_button("user-trash-symbolic", "Remove fake transaction");

    let id = fake.id;
    let state_for_edit = Rc::clone(state);
    let ui_for_edit = Rc::clone(ui);
    edit_button.connect_clicked(move |_| {
        show_fake_transaction_form(&state_for_edit, &ui_for_edit, Some(id));
    });

    let state_for_remove = Rc::clone(state);
    let ui_for_remove = Rc::clone(ui);
    remove_button.connect_clicked(move |_| {
        queue_fake_transaction_update(
            &state_for_remove,
            &ui_for_remove,
            "Removing fake transaction...",
            move |_, ui| {
                if ui.fake_transactions.remove(id) {
                    FakeTransactionUpdateOutcome::Render("Fake transaction removed.")
                } else {
                    FakeTransactionUpdateOutcome::Skip
                }
            },
        );
    });

    actions.append(&edit_button);
    actions.append(&remove_button);
    row.add_suffix(&actions);
    row
}
