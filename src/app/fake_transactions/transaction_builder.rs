use super::super::*;
use super::form::FakeTransactionAdvancedValues;
use super::{DEFAULT_FAKE_ACCOUNT, DEFAULT_FAKE_CURRENCY, FAKE_TRANSACTION_SOURCE};
use crate::util::{parse_date, parse_decimal};
use chrono::NaiveDate;

pub(super) struct FakeTransactionFormFields<'a> {
    pub(super) date: &'a gtk::Entry,
    pub(super) amount: &'a gtk::Entry,
    pub(super) counterparty: &'a gtk::Entry,
    pub(super) description: &'a gtk::Entry,
    pub(super) tags: Option<&'a gtk::Entry>,
    pub(super) category: &'a gtk::ComboBoxText,
    pub(super) budget_code: Option<&'a gtk::ComboBoxText>,
    pub(super) account: Option<&'a gtk::Entry>,
    pub(super) currency: Option<&'a gtk::Entry>,
    pub(super) notes: Option<&'a gtk::Entry>,
    pub(super) status: &'a gtk::Label,
    pub(super) budgets: &'a [crate::model::BudgetCode],
    pub(super) advanced_features: bool,
    pub(super) advanced_values: &'a FakeTransactionAdvancedValues,
}

pub(super) fn transaction_from_form(fields: FakeTransactionFormFields<'_>) -> Option<Transaction> {
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
    let entered_code = fields
        .budget_code
        .map(ui::combo_text)
        .unwrap_or_else(|| fields.advanced_values.budget_code.clone());
    let budget_code = fake_transaction_budget_code_for_save(
        &entered_code,
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
        tags: optional_entry_text(fields.tags, &fields.advanced_values.tags),
        account: non_empty_or(
            &optional_entry_text(fields.account, &fields.advanced_values.account),
            DEFAULT_FAKE_ACCOUNT,
        ),
        transaction_id: String::new(),
        currency: non_empty_or(
            &optional_entry_text(fields.currency, &fields.advanced_values.currency),
            DEFAULT_FAKE_CURRENCY,
        ),
        source_file: FAKE_TRANSACTION_SOURCE.to_string(),
        source_row: 0,
        category,
        budget_code,
        notes: optional_entry_text(fields.notes, &fields.advanced_values.notes),
        strict_key: String::new(),
        loose_key: String::new(),
        rule_match: None,
    })
}

fn optional_entry_text(entry: Option<&gtk::Entry>, fallback: &str) -> String {
    entry
        .map(|entry| entry.text().trim().to_string())
        .unwrap_or_else(|| fallback.trim().to_string())
}

pub(super) fn fake_transaction_budget_code_for_save(
    entered_code: &str,
    category: &str,
    budgets: &[crate::model::BudgetCode],
    advanced_features: bool,
) -> String {
    if advanced_features {
        return entered_code.trim().to_string();
    }

    fake_transaction_budget_for_category(category, budgets)
        .or_else(|| preferred_fake_transaction_transfer_budget(category, budgets))
        .map(|budget| budget.code.trim().to_string())
        .filter(|code| !code.is_empty())
        .unwrap_or_else(|| entered_code.trim().to_string())
}

fn preferred_fake_transaction_transfer_budget<'a>(
    category: &str,
    budgets: &'a [crate::model::BudgetCode],
) -> Option<&'a crate::model::BudgetCode> {
    if !BudgetDirection::parse("", "", category).is_transfer() {
        return None;
    }

    budgets
        .iter()
        .find(|budget| {
            budget.direction.is_transfer() && budget.code.trim().eq_ignore_ascii_case("TRANSFER")
        })
        .or_else(|| budgets.iter().find(|budget| budget.direction.is_transfer()))
}

pub(super) fn fake_transaction_amount_for_budget(
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

pub(super) fn default_fake_transaction(data: &AppData, ui: &UiHandles) -> Transaction {
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
        rule_match: None,
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

pub(super) fn normalize_fake_transaction(id: u64, mut transaction: Transaction) -> Transaction {
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

fn non_empty_or(input: &str, fallback: &str) -> String {
    let input = input.trim();
    if input.is_empty() {
        fallback.to_string()
    } else {
        input.to_string()
    }
}
