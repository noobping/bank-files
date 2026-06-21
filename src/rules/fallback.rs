use crate::model::Transaction;

use rust_decimal::Decimal;

pub(super) fn fallback_category(tx: &Transaction) -> CategoryAssignment {
    let (category, budget_code) = if tx.amount > Decimal::ZERO {
        other_income_fallback()
    } else {
        other_expense_fallback()
    };
    CategoryAssignment {
        category: category.to_string(),
        budget_code: budget_code.to_string(),
        notes: None,
    }
}

pub(super) struct CategoryAssignment {
    pub(super) category: String,
    pub(super) budget_code: String,
    pub(super) notes: Option<String>,
}

fn other_income_fallback() -> (&'static str, &'static str) {
    match crate::i18n::active_language() {
        crate::i18n::Language::English => ("Other income", "INC-OTHER"),
        crate::i18n::Language::Dutch => ("Overige inkomsten", "INC-OTHER"),
        crate::i18n::Language::German => ("Sonstige Einnahmen", "INC-OTHER"),
    }
}

fn other_expense_fallback() -> (&'static str, &'static str) {
    match crate::i18n::active_language() {
        crate::i18n::Language::English => ("Other", "OTHER"),
        crate::i18n::Language::Dutch => ("Overig", "OTHER"),
        crate::i18n::Language::German => ("Sonstiges", "OTHER"),
    }
}
