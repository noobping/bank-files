use super::*;
use crate::util::{parse_date, parse_decimal};
use chrono::NaiveDate;
use std::collections::BTreeSet;

const FAKE_TRANSACTION_SOURCE: &str = "Runtime fake transaction";
const DEFAULT_FAKE_ACCOUNT: &str = "Fake";
const DEFAULT_FAKE_CURRENCY: &str = "EUR";
const FAKE_TRANSACTIONS_LIST_PAGE: &str = "list";
const FAKE_TRANSACTIONS_FORM_PAGE: &str = "form";

#[derive(Debug, Clone)]
pub(in crate::app) struct FakeTransaction {
    pub(in crate::app) id: u64,
    pub(in crate::app) transaction: Transaction,
}

#[derive(Clone)]
pub(in crate::app) struct FakeTransactionStore {
    next_id: Rc<Cell<u64>>,
    transactions: Rc<RefCell<Vec<FakeTransaction>>>,
}

#[derive(Clone)]
pub(in crate::app) struct FakeTransactionWidgets {
    pub(in crate::app) button: gtk::Button,
    pub(in crate::app) badge: gtk::Label,
    pub(in crate::app) summary: gtk::Label,
    pub(in crate::app) busy_box: gtk::Box,
    pub(in crate::app) busy_label: gtk::Label,
    pub(in crate::app) back_button: gtk::Button,
    pub(in crate::app) add_button: gtk::Button,
    pub(in crate::app) clear_button: gtk::Button,
    pub(in crate::app) list_actions: gtk::Box,
    pub(in crate::app) form_actions: gtk::Box,
    pub(in crate::app) stack: gtk::Stack,
    pub(in crate::app) list: gtk::ListBox,
    pub(in crate::app) form_box: gtk::Box,
    pub(in crate::app) popover: gtk::Popover,
}

enum FakeTransactionUpdateOutcome {
    Render(&'static str),
    Skip,
}

impl FakeTransactionStore {
    pub(in crate::app) fn new() -> Self {
        Self {
            next_id: Rc::new(Cell::new(1)),
            transactions: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub(in crate::app) fn add(&self, transaction: Transaction) -> u64 {
        let id = self.next_id.get();
        self.next_id.set(id.saturating_add(1));
        self.transactions.borrow_mut().push(FakeTransaction {
            id,
            transaction: normalize_fake_transaction(id, transaction),
        });
        id
    }

    pub(in crate::app) fn update(&self, id: u64, transaction: Transaction) -> bool {
        let mut transactions = self.transactions.borrow_mut();
        let Some(fake) = transactions.iter_mut().find(|fake| fake.id == id) else {
            return false;
        };
        fake.transaction = normalize_fake_transaction(id, transaction);
        true
    }

    pub(in crate::app) fn remove(&self, id: u64) -> bool {
        let mut transactions = self.transactions.borrow_mut();
        let Some(index) = transactions.iter().position(|fake| fake.id == id) else {
            return false;
        };
        transactions.remove(index);
        true
    }

    pub(in crate::app) fn clear(&self) -> usize {
        let mut transactions = self.transactions.borrow_mut();
        let count = transactions.len();
        transactions.clear();
        count
    }

    pub(in crate::app) fn count(&self) -> usize {
        self.transactions.borrow().len()
    }

    pub(in crate::app) fn list(&self) -> Vec<FakeTransaction> {
        self.transactions.borrow().clone()
    }

    fn get(&self, id: u64) -> Option<FakeTransaction> {
        self.transactions
            .borrow()
            .iter()
            .find(|fake| fake.id == id)
            .cloned()
    }
}

pub(in crate::app) fn build_fake_transaction_widgets() -> FakeTransactionWidgets {
    let badge = gtk::Label::new(None);
    badge.add_css_class("caption");
    badge.set_visible(false);
    badge.set_halign(gtk::Align::Center);
    badge.set_valign(gtk::Align::Center);

    let icon = gtk::Image::from_icon_name("document-new-symbolic");
    let button_content = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    button_content.append(&badge);
    button_content.append(&icon);

    let button = gtk::Button::builder()
        .tooltip_text(tr("Fake transactions"))
        .build();
    button.add_css_class("flat");
    button.set_focus_on_click(false);
    button.set_child(Some(&button_content));

    let root = ui::compact_popover_root();

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header.set_hexpand(true);
    let title_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    title_box.set_hexpand(true);
    let title = gtk::Label::new(Some(&tr("Fake Transactions")));
    title.add_css_class("heading");
    title.set_selectable(false);
    title.set_xalign(0.0);
    title.set_width_chars(1);
    title.set_max_width_chars(28);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let summary = gtk::Label::new(None);
    summary.add_css_class("dim-label");
    summary.set_selectable(false);
    summary.set_xalign(0.0);
    summary.set_width_chars(1);
    summary.set_max_width_chars(34);
    summary.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title_box.append(&title);
    title_box.append(&summary);

    let back_button = ui::icon_button("go-previous-symbolic", "Back to fake transactions");
    back_button.set_visible(false);
    let add_button = ui::icon_button("list-add-symbolic", "Add fake transaction");
    let clear_button = ui::icon_button("edit-clear-all-symbolic", "Clear fake transactions");
    let list_actions = ui::linked_button_group();
    list_actions.append(&add_button);
    list_actions.append(&clear_button);
    let form_actions = ui::linked_button_group();
    form_actions.set_visible(false);
    header.append(&back_button);
    header.append(&title_box);
    header.append(&list_actions);
    header.append(&form_actions);
    root.append(&header);

    let busy_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    busy_box.add_css_class("dim-label");
    busy_box.set_visible(false);
    let busy_spinner = ui::loading_spinner();
    busy_spinner.set_size_request(16, 16);
    let busy_label = gtk::Label::new(None);
    busy_label.set_selectable(false);
    busy_label.set_xalign(0.0);
    busy_label.set_hexpand(true);
    busy_box.append(&busy_spinner);
    busy_box.append(&busy_label);
    root.append(&busy_box);

    let stack = gtk::Stack::builder()
        .hhomogeneous(false)
        .vhomogeneous(false)
        .transition_type(gtk::StackTransitionType::SlideLeftRight)
        .build();
    stack.set_hexpand(true);

    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    list.set_selection_mode(gtk::SelectionMode::None);
    list.set_hexpand(true);
    stack.add_named(&list, Some(FAKE_TRANSACTIONS_LIST_PAGE));

    let form_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    form_box.set_hexpand(true);
    stack.add_named(&form_box, Some(FAKE_TRANSACTIONS_FORM_PAGE));
    stack.set_visible_child_name(FAKE_TRANSACTIONS_LIST_PAGE);

    let scroll = ui::compact_popover_scroll(&stack);
    root.append(&scroll);

    let popover = gtk::Popover::builder().autohide(true).build();
    popover.set_child(Some(&root));
    popover.set_parent(&button);
    let popover_for_button = popover.clone();
    button.connect_clicked(move |_| {
        if popover_for_button.is_visible() {
            popover_for_button.popdown();
        } else {
            popover_for_button.popup();
        }
    });

    FakeTransactionWidgets {
        button,
        badge,
        summary,
        busy_box,
        busy_label,
        back_button,
        add_button,
        clear_button,
        list_actions,
        form_actions,
        stack,
        list,
        form_box,
        popover,
    }
}

pub(in crate::app) fn connect_fake_transactions(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
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
                        show_fake_transaction_list(&ui.fake_transaction_widgets);
                        ui.fake_transaction_widgets.popover.popdown();
                        FakeTransactionUpdateOutcome::Render("Fake transactions cleared.")
                    } else {
                        FakeTransactionUpdateOutcome::Skip
                    }
                },
            );
        });

    connect_fake_transaction_popover_autoclose(ui);
    refresh_fake_transactions_ui(state, ui);
}

fn connect_fake_transaction_popover_autoclose(ui: &Rc<UiHandles>) {
    let click = gtk::GestureClick::new();
    click.set_propagation_phase(gtk::PropagationPhase::Capture);
    let popover = ui.fake_transaction_widgets.popover.clone();
    let button: gtk::Widget = ui.fake_transaction_widgets.button.clone().upcast();
    click.connect_pressed(move |gesture, _, x, y| {
        if !popover.is_visible() {
            return;
        }
        let Some(widget) = gesture.widget() else {
            popover.popdown();
            return;
        };
        let clicked_button = widget
            .pick(x, y, gtk::PickFlags::DEFAULT)
            .is_some_and(|target| target == button || target.is_ancestor(&button));
        if !clicked_button {
            popover.popdown();
        }
    });
    ui.window.add_controller(click);
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

pub(in crate::app) fn data_with_fake_transactions(
    mut data: AppData,
    fake_transactions: Vec<FakeTransaction>,
) -> AppData {
    if fake_transactions.is_empty() {
        return data;
    }

    data.transactions
        .extend(fake_transactions.into_iter().map(|fake| fake.transaction));
    sort_transactions(&mut data.transactions);
    extend_available_periods(&mut data);
    data
}

pub(in crate::app) fn transaction_is_fake(transaction: &Transaction) -> bool {
    transaction.source_file == FAKE_TRANSACTION_SOURCE
        && transaction.transaction_id.starts_with("FAKE-")
}

pub(in crate::app) fn real_transactions(transactions: &[Transaction]) -> Vec<Transaction> {
    transactions
        .iter()
        .filter(|transaction| !transaction_is_fake(transaction))
        .cloned()
        .collect()
}

fn show_fake_transaction_list(widgets: &FakeTransactionWidgets) {
    widgets
        .stack
        .set_visible_child_name(FAKE_TRANSACTIONS_LIST_PAGE);
    widgets.back_button.set_visible(false);
    widgets.list_actions.set_visible(true);
    widgets.form_actions.set_visible(false);
}

fn show_fake_transaction_form_page(widgets: &FakeTransactionWidgets) {
    widgets
        .stack
        .set_visible_child_name(FAKE_TRANSACTIONS_FORM_PAGE);
    widgets.back_button.set_visible(true);
    widgets.list_actions.set_visible(false);
    widgets.form_actions.set_visible(true);
}

fn refresh_fake_transactions_ui(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
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
    widgets.form_actions.set_sensitive(!busy);
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

    for fake in fake_transactions {
        widgets.list.append(&fake_transaction_row(state, ui, fake));
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

fn show_fake_transaction_form(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    edit_id: Option<u64>,
) {
    let initial = edit_id
        .and_then(|id| ui.fake_transactions.get(id).map(|fake| fake.transaction))
        .unwrap_or_else(|| default_fake_transaction(&state.borrow(), ui.as_ref()));

    let widgets = &ui.fake_transaction_widgets;
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
    let tags = ui::entry(&initial.tags, "Tags");
    let category = ui::text_combo(&initial.category, app_category_values(&state.borrow()));
    let budget_code = ui::text_combo(
        &initial.budget_code,
        app_budget_code_values(&state.borrow()),
    );
    let account = ui::entry(&initial.account, DEFAULT_FAKE_ACCOUNT);
    let currency = ui::entry(&initial.currency, DEFAULT_FAKE_CURRENCY);
    let notes = ui::entry(&initial.notes, "Notes");
    let direction = ui::combo_from_options(
        &[
            ("expense", "Expenses"),
            ("income", "Income"),
            ("transfer", "Transfers"),
        ],
        direction_for_amount(initial.amount),
    );
    direction.set_visible(false);

    connect_budget_fields_autofill(
        &category,
        &budget_code,
        &direction,
        app_budget_autofill_entries(&state.borrow()),
        &ui.advanced_autofill,
    );
    connect_amount_direction(&amount, &direction);

    ui::add_labeled(&main_grid, 0, "Date", &date);
    ui::add_labeled(&main_grid, 1, "Amount", &amount);
    ui::add_labeled(&main_grid, 2, "Counterparty", &counterparty);
    ui::add_labeled(&main_grid, 3, "Description", &description);
    ui::add_labeled(&main_grid, 4, "Category", &category);
    if ui.advanced_features.get() {
        ui::add_labeled(&main_grid, 5, "Budget code", &budget_code);
    } else {
        budget_code.set_visible(false);
    }
    ui::add_labeled(&details_grid, 0, "Tags", &tags);
    ui::add_labeled(&details_grid, 1, "Account", &account);
    ui::add_labeled(&details_grid, 2, "Currency", &currency);
    ui::add_labeled(&details_grid, 3, "Notes", &notes);
    widgets.form_box.append(&main_grid);
    widgets.form_box.append(&details_grid);

    let status = ui::wrapped_label("");
    status.add_css_class("dim-label");
    status.set_selectable(false);
    widgets.form_box.append(&status);

    ui::clear_box(&widgets.form_actions);
    let save_button = ui::icon_button("document-save-symbolic", "Save fake transaction");
    save_button.add_css_class("suggested-action");
    widgets.form_actions.append(&save_button);

    let state_for_save = Rc::clone(state);
    let ui_for_save = Rc::clone(ui);
    save_button.connect_clicked(move |_| {
        let Some(transaction) = ({
            let data = state_for_save.borrow();
            transaction_from_form(FakeTransactionFormFields {
                date: &date,
                amount: &amount,
                counterparty: &counterparty,
                description: &description,
                tags: &tags,
                category: &category,
                budget_code: &budget_code,
                account: &account,
                currency: &currency,
                notes: &notes,
                status: &status,
                budgets: &data.budgets,
                advanced_features: ui_for_save.advanced_features.get(),
            })
        }) else {
            return;
        };

        let status_for_save = status.clone();
        queue_fake_transaction_update(
            &state_for_save,
            &ui_for_save,
            "Saving fake transaction...",
            move |_, ui| {
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
            },
        );
    });

    widgets.popover.popup();
}

fn queue_fake_transaction_update<F>(
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
    widgets.form_actions.set_sensitive(!busy);
    widgets
        .clear_button
        .set_sensitive(!busy && ui.fake_transactions.count() > 0);
    widgets.stack.set_sensitive(!busy);
    widgets.form_box.set_sensitive(!busy);
    widgets.list.set_sensitive(!busy);
}

struct FakeTransactionFormFields<'a> {
    date: &'a gtk::Entry,
    amount: &'a gtk::Entry,
    counterparty: &'a gtk::Entry,
    description: &'a gtk::Entry,
    tags: &'a gtk::Entry,
    category: &'a gtk::ComboBoxText,
    budget_code: &'a gtk::ComboBoxText,
    account: &'a gtk::Entry,
    currency: &'a gtk::Entry,
    notes: &'a gtk::Entry,
    status: &'a gtk::Label,
    budgets: &'a [crate::model::BudgetCode],
    advanced_features: bool,
}

fn transaction_from_form(fields: FakeTransactionFormFields<'_>) -> Option<Transaction> {
    let Some(date) = parse_date(&fields.date.text()) else {
        fields
            .status
            .set_text(&tr("Fake transaction date is invalid."));
        fields.date.grab_focus();
        return None;
    };
    let Some(amount) = parse_decimal(&fields.amount.text()) else {
        fields
            .status
            .set_text(&tr("Fake transaction amount is invalid."));
        fields.amount.grab_focus();
        return None;
    };

    let counterparty = fields.counterparty.text().trim().to_string();
    let description = fields.description.text().trim().to_string();
    let category = non_empty_or(&ui::combo_text(fields.category), "Uncategorized");
    let budget_code = fake_transaction_budget_code_for_save(
        &ui::combo_text(fields.budget_code),
        &category,
        fields.budgets,
        fields.advanced_features,
    );
    let amount =
        fake_transaction_amount_for_budget(amount, &budget_code, &category, fields.budgets);
    Some(Transaction {
        date,
        amount,
        description: if description.is_empty() {
            tr("Fake transaction")
        } else {
            description
        },
        counterparty,
        tags: fields.tags.text().trim().to_string(),
        account: non_empty_or(&fields.account.text(), DEFAULT_FAKE_ACCOUNT),
        transaction_id: String::new(),
        currency: non_empty_or(&fields.currency.text(), DEFAULT_FAKE_CURRENCY),
        source_file: FAKE_TRANSACTION_SOURCE.to_string(),
        source_row: 0,
        category,
        budget_code,
        notes: fields.notes.text().trim().to_string(),
        strict_key: String::new(),
        loose_key: String::new(),
    })
}

fn fake_transaction_budget_code_for_save(
    entered_code: &str,
    category: &str,
    budgets: &[crate::model::BudgetCode],
    advanced_features: bool,
) -> String {
    if advanced_features {
        return entered_code.trim().to_string();
    }

    fake_transaction_budget_for_category(category, budgets)
        .map(|budget| budget.code.trim().to_string())
        .filter(|code| !code.is_empty())
        .unwrap_or_else(|| entered_code.trim().to_string())
}

fn fake_transaction_amount_for_budget(
    amount: Decimal,
    budget_code: &str,
    category: &str,
    budgets: &[crate::model::BudgetCode],
) -> Decimal {
    match fake_transaction_budget_direction(budget_code, category, budgets) {
        BudgetDirection::Expense => -decimal_abs(amount),
        BudgetDirection::Income => decimal_abs(amount),
        BudgetDirection::Transfer => amount,
    }
}

fn fake_transaction_budget_direction(
    budget_code: &str,
    category: &str,
    budgets: &[crate::model::BudgetCode],
) -> BudgetDirection {
    let budget_code = budget_code.trim();
    budgets
        .iter()
        .find(|budget| budget.code.trim().eq_ignore_ascii_case(budget_code))
        .or_else(|| fake_transaction_budget_for_category(category, budgets))
        .map(|budget| budget.direction)
        .unwrap_or_else(|| BudgetDirection::parse("", budget_code, category))
}

fn fake_transaction_budget_for_category<'a>(
    category: &str,
    budgets: &'a [crate::model::BudgetCode],
) -> Option<&'a crate::model::BudgetCode> {
    let category = category.trim();
    if category.is_empty() {
        return None;
    }

    budgets
        .iter()
        .find(|budget| budget.category.trim().eq_ignore_ascii_case(category))
}

fn decimal_abs(amount: Decimal) -> Decimal {
    if amount < Decimal::ZERO {
        -amount
    } else {
        amount
    }
}

fn default_fake_transaction(data: &AppData, ui: &UiHandles) -> Transaction {
    let date = default_fake_date(data, ui);
    let (category, budget_code) = default_fake_budget(data);
    Transaction {
        date,
        amount: Decimal::ZERO,
        description: tr("Fake transaction"),
        counterparty: String::new(),
        tags: String::new(),
        account: DEFAULT_FAKE_ACCOUNT.to_string(),
        transaction_id: String::new(),
        currency: DEFAULT_FAKE_CURRENCY.to_string(),
        source_file: FAKE_TRANSACTION_SOURCE.to_string(),
        source_row: 0,
        category,
        budget_code,
        notes: String::new(),
        strict_key: String::new(),
        loose_key: String::new(),
    }
}

fn default_fake_date(data: &AppData, ui: &UiHandles) -> NaiveDate {
    let month = ui
        .selected_budget_month
        .get()
        .or(data.default_month)
        .or_else(|| ui.selected_year.get().map(|year| MonthKey::new(year, 1)));
    month
        .and_then(|month| NaiveDate::from_ymd_opt(month.year, month.month, 1))
        .unwrap_or_else(|| chrono::Local::now().date_naive())
}

fn default_fake_budget(data: &AppData) -> (String, String) {
    data.budgets
        .iter()
        .find(|budget| budget.code.eq_ignore_ascii_case("OTHER"))
        .or_else(|| {
            data.budgets
                .iter()
                .find(|budget| budget.direction.is_expense())
        })
        .or_else(|| data.budgets.first())
        .map(|budget| (budget.category.clone(), budget.code.clone()))
        .unwrap_or_else(|| ("Uncategorized".to_string(), String::new()))
}

fn normalize_fake_transaction(id: u64, mut transaction: Transaction) -> Transaction {
    transaction.transaction_id = format!("FAKE-{id}");
    transaction.source_file = FAKE_TRANSACTION_SOURCE.to_string();
    transaction.source_row = usize::try_from(id).unwrap_or(usize::MAX);
    transaction.strict_key = format!("fake-{id}-strict");
    transaction.loose_key = format!("fake-{id}-loose");
    if transaction.account.trim().is_empty() {
        transaction.account = DEFAULT_FAKE_ACCOUNT.to_string();
    }
    if transaction.currency.trim().is_empty() {
        transaction.currency = DEFAULT_FAKE_CURRENCY.to_string();
    }
    transaction
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

fn fake_transaction_title(transaction: &Transaction) -> String {
    let title = if transaction.counterparty.trim().is_empty() {
        transaction.description.trim()
    } else {
        transaction.counterparty.trim()
    };
    if title.is_empty() {
        tr("Fake transaction")
    } else {
        title.to_string()
    }
}

fn fake_transaction_subtitle(transaction: &Transaction) -> String {
    format!(
        "{} · {} · {} · {}",
        transaction.date, transaction.category, transaction.budget_code, transaction.description
    )
}

fn non_empty_or(input: &str, fallback: &str) -> String {
    let input = input.trim();
    if input.is_empty() {
        fallback.to_string()
    } else {
        input.to_string()
    }
}

fn sort_transactions(transactions: &mut [Transaction]) {
    transactions.sort_by(|a, b| {
        b.date
            .cmp(&a.date)
            .then_with(|| a.description.cmp(&b.description))
    });
}

fn extend_available_periods(data: &mut AppData) {
    let mut months = data
        .available_months
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    months.extend(data.transactions.iter().map(Transaction::month_key));
    data.available_months = months.into_iter().collect();
    data.available_years = data
        .available_months
        .iter()
        .map(|month| month.year)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if data.default_month.is_none() {
        data.default_month = data.available_months.last().copied();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(date: &str, amount: i64, description: &str) -> Transaction {
        Transaction {
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            amount: Decimal::new(amount, 0),
            description: description.to_string(),
            counterparty: String::new(),
            tags: String::new(),
            account: DEFAULT_FAKE_ACCOUNT.to_string(),
            transaction_id: String::new(),
            currency: DEFAULT_FAKE_CURRENCY.to_string(),
            source_file: String::new(),
            source_row: 0,
            category: "Other".to_string(),
            budget_code: "OTHER".to_string(),
            notes: String::new(),
            strict_key: String::new(),
            loose_key: String::new(),
        }
    }

    fn budget(code: &str, category: &str, direction: BudgetDirection) -> crate::model::BudgetCode {
        crate::model::BudgetCode {
            code: code.to_string(),
            category: category.to_string(),
            monthly_budget: None,
            yearly_budget: None,
            direction,
            income_basis: crate::model::BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        }
    }

    #[test]
    fn fake_transaction_amount_uses_budget_code_direction() {
        let budgets = vec![
            budget("FOOD", "Food", BudgetDirection::Expense),
            budget("SALARY", "Salary", BudgetDirection::Income),
        ];

        assert_eq!(
            fake_transaction_amount_for_budget(Decimal::new(25, 0), "FOOD", "Food", &budgets),
            Decimal::new(-25, 0)
        );
        assert_eq!(
            fake_transaction_amount_for_budget(Decimal::new(-25, 0), "SALARY", "Salary", &budgets),
            Decimal::new(25, 0)
        );
    }

    #[test]
    fn fake_transaction_amount_keeps_transfer_sign() {
        let budgets = vec![budget("TRANSFER", "Transfers", BudgetDirection::Transfer)];

        assert_eq!(
            fake_transaction_amount_for_budget(
                Decimal::new(-25, 0),
                "TRANSFER",
                "Transfers",
                &budgets,
            ),
            Decimal::new(-25, 0)
        );
        assert_eq!(
            fake_transaction_amount_for_budget(
                Decimal::new(25, 0),
                "TRANSFER",
                "Transfers",
                &budgets,
            ),
            Decimal::new(25, 0)
        );
    }

    #[test]
    fn simple_fake_transaction_budget_code_is_inferred_from_category() {
        let budgets = vec![
            budget("OTHER", "Other", BudgetDirection::Expense),
            budget("SALARY", "Salary", BudgetDirection::Income),
        ];

        assert_eq!(
            fake_transaction_budget_code_for_save("OTHER", "Salary", &budgets, false),
            "SALARY"
        );
        assert_eq!(
            fake_transaction_budget_code_for_save("OTHER", "Salary", &budgets, true),
            "OTHER"
        );
    }

    #[test]
    fn fake_transaction_direction_falls_back_to_code_and_category_context() {
        assert_eq!(
            fake_transaction_amount_for_budget(Decimal::new(-25, 0), "INC-BONUS", "Bonus", &[]),
            Decimal::new(25, 0)
        );
        assert_eq!(
            fake_transaction_amount_for_budget(Decimal::new(25, 0), "MISC", "Other", &[]),
            Decimal::new(-25, 0)
        );
    }

    #[test]
    fn fake_store_add_assigns_stable_ids_and_counts() {
        let store = FakeTransactionStore::new();
        let first = store.add(tx("2025-01-01", -10, "first"));
        let second = store.add(tx("2025-01-02", -20, "second"));

        assert_eq!(first, 1);
        assert_eq!(second, 2);
        assert_eq!(store.count(), 2);
        assert_eq!(store.list()[0].transaction.transaction_id, "FAKE-1");
        assert_eq!(store.list()[1].transaction.transaction_id, "FAKE-2");
    }

    #[test]
    fn fake_store_update_remove_and_clear_work() {
        let store = FakeTransactionStore::new();
        let first = store.add(tx("2025-01-01", -10, "first"));
        let second = store.add(tx("2025-01-02", -20, "second"));

        assert!(store.update(first, tx("2025-01-03", -30, "updated")));
        assert_eq!(store.get(first).unwrap().transaction.description, "updated");
        assert!(store.remove(second));
        assert_eq!(store.count(), 1);
        assert_eq!(store.clear(), 1);
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn merged_data_appends_fakes_without_mutating_real_data() {
        let real = tx("2025-01-01", -10, "real");
        let fake = FakeTransaction {
            id: 1,
            transaction: normalize_fake_transaction(1, tx("2026-02-03", -20, "fake")),
        };
        let data = AppData {
            transactions: vec![real],
            available_months: vec![MonthKey::new(2025, 1)],
            available_years: vec![2025],
            default_month: Some(MonthKey::new(2025, 1)),
            ..AppData::default()
        };

        let merged = data_with_fake_transactions(data.clone(), vec![fake]);

        assert_eq!(data.transactions.len(), 1);
        assert_eq!(merged.transactions.len(), 2);
        assert_eq!(merged.transactions[0].description, "fake");
        assert_eq!(
            merged.available_months,
            vec![MonthKey::new(2025, 1), MonthKey::new(2026, 2)]
        );
        assert_eq!(merged.available_years, vec![2025, 2026]);
        assert!(transaction_is_fake(&merged.transactions[0]));
    }

    #[test]
    fn real_transactions_excludes_runtime_fakes_for_export() {
        let real = tx("2025-01-01", -10, "real");
        let fake = normalize_fake_transaction(1, tx("2025-01-02", -20, "fake"));

        let real_only = real_transactions(&[real.clone(), fake]);

        assert_eq!(real_only.len(), 1);
        assert_eq!(real_only[0].description, real.description);
    }
}
