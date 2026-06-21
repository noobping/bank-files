use super::*;

pub(in crate::app) struct AttentionWarning {
    pub(super) title: &'static str,
    pub(super) message: String,
}

impl AttentionWarning {
    pub(super) fn new(title: &'static str, message: String) -> Self {
        Self { title, message }
    }

    fn titled_message(&self) -> String {
        format!("{}: {}", tr(self.title), self.message)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::app) struct BudgetWarningTotals {
    pub(in crate::app) real_expenses: Decimal,
    pub(in crate::app) real_income: Decimal,
    pub(in crate::app) planned_expenses: Decimal,
    pub(in crate::app) planned_income: Decimal,
    pub(in crate::app) annual_budget_room_used: Decimal,
}

pub(in crate::app) fn attention_warning_messages(warnings: &[AttentionWarning]) -> Vec<String> {
    warnings
        .iter()
        .map(AttentionWarning::titled_message)
        .collect()
}

pub(in crate::app) fn attention_warning_card_message(
    warnings: &[AttentionWarning],
) -> Option<String> {
    match warnings {
        [] => None,
        [warning] => Some(warning.message.clone()),
        warnings => Some(attention_warning_messages(warnings).join(
            "
",
        )),
    }
}
