use super::super::*;
use super::list::refresh_fake_transactions_ui;
use super::model::FakeTransactionUpdateOutcome;
use super::transaction_builder::{
    default_fake_transaction, transaction_from_form, FakeTransactionFormFields,
};
use super::widgets::{show_fake_transaction_form_page, show_fake_transaction_list};
use super::{DEFAULT_FAKE_ACCOUNT, DEFAULT_FAKE_CURRENCY, FAKE_TRANSACTIONS_LIST_PAGE};
use crate::util::parse_decimal;

pub(super) fn show_fake_transaction_form(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    edit_id: Option<u64>,
) {
    let initial = edit_id
        .and_then(|id| ui.fake_transactions.get(id).map(|fake| fake.transaction))
        .unwrap_or_else(|| default_fake_transaction(&state.borrow(), ui.as_ref()));
    let advanced_values = FakeTransactionAdvancedValues::from_transaction(&initial);
    let widgets = &ui.fake_transaction_widgets;
    let advanced_features = ui.advanced_features.get();
    ui::clear_box(&widgets.form_box);
    show_fake_transaction_form_page(widgets);

    let title = gtk::Label::new(Some(&tr(if edit_id.is_some() {
        "Edit Fake Transaction"
    } else {
        "New Fake Transaction"
    })));
    title.add_css_class("heading");
    title.set_xalign(0.0);
    title.set_selectable(false);
    widgets.form_box.append(&title);

    let main_grid = ui::form_grid();
    let details_grid = ui::form_grid();
    details_grid.set_margin_top(8);
    let date = ui::entry(&initial.date.to_string(), "YYYY-MM-DD");
    let amount = ui::entry(&initial.amount.normalize().to_string(), "0.00");
    let counterparty = ui::entry(&initial.counterparty, "Counterparty");
    let description = ui::entry(&initial.description, "Description");
    let category = ui::text_combo(&initial.category, app_category_values(&state.borrow()));
    let budget_code = advanced_features.then(|| {
        ui::text_combo(
            &initial.budget_code,
            app_budget_code_values(&state.borrow()),
        )
    });
    let tags = advanced_features.then(|| ui::entry(&initial.tags, "Tags"));
    let account = advanced_features.then(|| ui::entry(&initial.account, DEFAULT_FAKE_ACCOUNT));
    let currency = advanced_features.then(|| ui::entry(&initial.currency, DEFAULT_FAKE_CURRENCY));
    let notes = advanced_features.then(|| ui::entry(&initial.notes, "Notes"));
    let direction = advanced_features.then(|| {
        let direction = ui::combo_from_options(
            &[
                ("expense", "Expenses"),
                ("income", "Income"),
                ("transfer", "Transfers"),
            ],
            direction_for_amount(initial.amount),
        );
        direction.set_visible(false);
        direction
    });

    if let (Some(budget_code), Some(direction)) = (&budget_code, &direction) {
        connect_budget_fields_autofill(
            &category,
            budget_code,
            direction,
            app_budget_autofill_entries(&state.borrow()),
            &ui.advanced_autofill,
        );
        connect_amount_direction(&amount, direction);
    }

    ui::add_labeled(&main_grid, 0, "Date", &date);
    ui::add_labeled(&main_grid, 1, "Amount", &amount);
    ui::add_labeled(&main_grid, 2, "Counterparty", &counterparty);
    ui::add_labeled(&main_grid, 3, "Description", &description);
    ui::add_labeled(&main_grid, 4, "Category", &category);
    if let (Some(budget_code), Some(tags), Some(account), Some(currency), Some(notes)) =
        (&budget_code, &tags, &account, &currency, &notes)
    {
        ui::add_labeled(&main_grid, 5, "Budget code", budget_code);
        ui::add_labeled(&details_grid, 0, "Tags", tags);
        ui::add_labeled(&details_grid, 1, "Account", account);
        ui::add_labeled(&details_grid, 2, "Currency", currency);
        ui::add_labeled(&details_grid, 3, "Notes", notes);
    }
    widgets.form_box.append(&main_grid);
    if advanced_features {
        widgets.form_box.append(&details_grid);
    }

    let status = ui::wrapped_label("");
    status.add_css_class("dim-label");
    status.set_selectable(false);
    widgets.form_box.append(&status);

    widgets.form_state.replace(Some(FakeTransactionFormState {
        edit_id,
        date,
        amount,
        counterparty,
        description,
        tags,
        category,
        budget_code,
        account,
        currency,
        notes,
        status,
        advanced_features,
        advanced_values,
    }));

    widgets.dialog.present(Some(&ui.window));
}

pub(super) fn save_fake_transaction_form(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
    let Some(form) = ui.fake_transaction_widgets.form_state.borrow().clone() else {
        return;
    };
    let Some(transaction) = ({
        let data = state.borrow();
        transaction_from_form(FakeTransactionFormFields {
            date: &form.date,
            amount: &form.amount,
            counterparty: &form.counterparty,
            description: &form.description,
            tags: form.tags.as_ref(),
            category: &form.category,
            budget_code: form.budget_code.as_ref(),
            account: form.account.as_ref(),
            currency: form.currency.as_ref(),
            notes: form.notes.as_ref(),
            status: &form.status,
            budgets: &data.budgets,
            advanced_features: form.advanced_features,
            advanced_values: &form.advanced_values,
        })
    }) else {
        return;
    };

    let edit_id = form.edit_id;
    let status_for_save = form.status.clone();
    queue_fake_transaction_update(state, ui, "Saving fake transaction...", move |_, ui| {
        let message = if let Some(id) = edit_id {
            if ui.fake_transactions.update(id, transaction) {
                "Fake transaction updated."
            } else {
                status_for_save.set_text(&tr("Fake transaction no longer exists."));
                return FakeTransactionUpdateOutcome::Skip;
            }
        } else {
            ui.fake_transactions.add(transaction);
            "Fake transaction added."
        };

        show_fake_transaction_list(&ui.fake_transaction_widgets);
        FakeTransactionUpdateOutcome::Render(message)
    });
}

pub(super) fn queue_fake_transaction_update<F>(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    busy_message: &'static str,
    update: F,
) where
    F: FnOnce(&Rc<RefCell<AppData>>, &Rc<UiHandles>) -> FakeTransactionUpdateOutcome + 'static,
{
    show_verbose_status(
        ui.as_ref(),
        format!("fake transaction update queued; message={busy_message}"),
    );
    set_fake_transactions_busy(ui, true, busy_message);
    let state = Rc::clone(state);
    let ui = Rc::clone(ui);
    gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(30), move || {
        match update(&state, &ui) {
            FakeTransactionUpdateOutcome::Render(status) => {
                show_verbose_status(
                    ui.as_ref(),
                    format!("fake transaction update rendered; status={status}"),
                );
                refresh_fake_transactions_ui(&state, &ui);
                request_render_views(&ui, &state);
                set_fake_transactions_busy(&ui, false, "");
                show_status(&ui, status);
            }
            FakeTransactionUpdateOutcome::Skip => {
                show_verbose_status(ui.as_ref(), "fake transaction update skipped");
                set_fake_transactions_busy(&ui, false, "");
            }
        }
    });
}

fn set_fake_transactions_busy(ui: &Rc<UiHandles>, busy: bool, message: &str) {
    let widgets = &ui.fake_transaction_widgets;
    widgets.busy_box.set_visible(busy);
    if busy {
        widgets.busy_label.set_text(&tr(message));
    } else {
        widgets.busy_label.set_text("");
    }
    widgets.back_button.set_sensitive(!busy);
    widgets.add_button.set_sensitive(!busy);
    widgets.save_button.set_sensitive(!busy);
    widgets
        .clear_button
        .set_sensitive(!busy && ui.fake_transactions.count() > 0);
    widgets.stack.set_sensitive(!busy);
    widgets.form_box.set_sensitive(!busy);
    widgets.list.set_sensitive(!busy);
    let showing_list =
        widgets.stack.visible_child_name().as_deref() == Some(FAKE_TRANSACTIONS_LIST_PAGE);
    if !busy && showing_list && !widgets.search_bar.is_search_mode() {
        widgets.add_button.grab_focus();
    }
}

#[derive(Clone)]
pub(super) struct FakeTransactionFormState {
    edit_id: Option<u64>,
    date: gtk::Entry,
    amount: gtk::Entry,
    counterparty: gtk::Entry,
    description: gtk::Entry,
    tags: Option<gtk::Entry>,
    category: gtk::ComboBoxText,
    budget_code: Option<gtk::ComboBoxText>,
    account: Option<gtk::Entry>,
    currency: Option<gtk::Entry>,
    notes: Option<gtk::Entry>,
    status: gtk::Label,
    advanced_features: bool,
    advanced_values: FakeTransactionAdvancedValues,
}

#[derive(Clone)]
pub(super) struct FakeTransactionAdvancedValues {
    pub(super) budget_code: String,
    pub(super) tags: String,
    pub(super) account: String,
    pub(super) currency: String,
    pub(super) notes: String,
}

impl FakeTransactionAdvancedValues {
    fn from_transaction(transaction: &Transaction) -> Self {
        Self {
            budget_code: transaction.budget_code.clone(),
            tags: transaction.tags.clone(),
            account: transaction.account.clone(),
            currency: transaction.currency.clone(),
            notes: transaction.notes.clone(),
        }
    }
}

fn connect_amount_direction(amount: &gtk::Entry, direction: &gtk::ComboBoxText) {
    let direction_for_amount = direction.clone();
    amount.connect_changed(move |entry| {
        if let Some(amount) = parse_decimal(&entry.text()) {
            direction_for_amount.set_active_id(Some(direction_for_amount_value(amount)));
        }
    });
}

fn direction_for_amount(amount: Decimal) -> &'static str {
    direction_for_amount_value(amount)
}

fn direction_for_amount_value(amount: Decimal) -> &'static str {
    if amount > Decimal::ZERO {
        "income"
    } else {
        "expense"
    }
}
