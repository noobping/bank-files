use super::super::*;
use super::form::{
    queue_fake_transaction_update, save_fake_transaction_form, show_fake_transaction_form,
};
use super::list::refresh_fake_transactions_ui;
use super::model::FakeTransactionUpdateOutcome;
use super::widgets::show_fake_transaction_list;

pub(in crate::app) fn connect_fake_transactions(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
    let state_for_open = Rc::clone(state);
    let ui_for_open = Rc::clone(ui);
    ui.fake_transaction_widgets
        .button
        .connect_clicked(move |_| show_fake_transactions_dialog(&state_for_open, &ui_for_open));

    let ui_for_back = Rc::clone(ui);
    ui.fake_transaction_widgets
        .back_button
        .connect_clicked(move |_| {
            show_fake_transaction_list(&ui_for_back.fake_transaction_widgets)
        });

    let state_for_add = Rc::clone(state);
    let ui_for_add = Rc::clone(ui);
    ui.fake_transaction_widgets
        .add_button
        .connect_clicked(move |_| show_fake_transaction_form(&state_for_add, &ui_for_add, None));

    let state_for_save = Rc::clone(state);
    let ui_for_save = Rc::clone(ui);
    ui.fake_transaction_widgets
        .save_button
        .connect_clicked(move |_| save_fake_transaction_form(&state_for_save, &ui_for_save));

    let state_for_search = Rc::clone(state);
    let ui_for_search = Rc::clone(ui);
    ui.fake_transaction_widgets
        .search_entry
        .connect_search_changed(move |_| {
            refresh_fake_transactions_ui(&state_for_search, &ui_for_search)
        });

    let state_for_clear = Rc::clone(state);
    let ui_for_clear = Rc::clone(ui);
    ui.fake_transaction_widgets
        .clear_button
        .connect_clicked(move |_| {
            queue_fake_transaction_update(
                &state_for_clear,
                &ui_for_clear,
                "Clearing fake transactions...",
                |_, ui| {
                    if ui.fake_transactions.clear() > 0 {
                        ui.fake_transaction_widgets.dialog.close();
                        FakeTransactionUpdateOutcome::Render("Fake transactions cleared.")
                    } else {
                        FakeTransactionUpdateOutcome::Skip
                    }
                },
            );
        });

    refresh_fake_transactions_ui(state, ui);
}

pub(in crate::app) fn show_fake_transactions_dialog(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    refresh_fake_transactions_ui(state, ui);
    show_fake_transaction_list(&ui.fake_transaction_widgets);
    ui.fake_transaction_widgets.dialog.present(Some(&ui.window));
}

pub(in crate::app) fn duplicate_transaction_as_fake(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    transaction: &Transaction,
) {
    let transaction = transaction.clone();
    queue_fake_transaction_update(state, ui, "Adding fake transaction...", move |state, ui| {
        let id = ui.fake_transactions.add(transaction);
        show_fake_transaction_form(state, ui, Some(id));
        FakeTransactionUpdateOutcome::Render("Fake transaction added.")
    });
}
