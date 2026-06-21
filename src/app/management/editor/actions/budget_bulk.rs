use super::*;
use crate::model::BudgetAmount;

#[derive(Debug, Clone, Copy)]
pub(super) enum BudgetValuePeriod {
    Monthly,
    Yearly,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub(super) struct BudgetBulkResult {
    pub(super) changed: usize,
    pub(super) skipped: usize,
}

pub(super) fn set_budget_forms_income_basis(forms: &[BudgetForm], basis: &str) -> usize {
    let mut changed = 0;
    for form in forms
        .iter()
        .filter(|form| !form.deleted.get() && !budget_form_is_planned_income(form))
    {
        let before = combo_active_id(&form.income_basis);
        form.income_basis.set_active_id(Some(basis));
        if form.income_basis.active_id().is_none() {
            form.income_basis.set_active(Some(0));
        }
        if before != combo_active_id(&form.income_basis) {
            changed += 1;
        }
    }
    changed
}

fn budget_form_is_planned_income(form: &BudgetForm) -> bool {
    planned_income::is_planned_income_budget(&form.special, &ui::combo_text(&form.code))
}

pub(super) fn set_budget_forms_value_period(
    forms: &[BudgetForm],
    period: BudgetValuePeriod,
) -> BudgetBulkResult {
    let mut result = BudgetBulkResult::default();
    for form in forms.iter().filter(|form| !form.deleted.get()) {
        let monthly = form.monthly_budget.text().trim().to_string();
        let yearly = form.yearly_budget.text().trim().to_string();
        match budget_values_for_period(&monthly, &yearly, period) {
            BudgetValueUpdate::Changed { monthly, yearly } => {
                form.monthly_budget.set_text(&monthly);
                form.yearly_budget.set_text(&yearly);
                result.changed += 1;
            }
            BudgetValueUpdate::Unchanged => {}
            BudgetValueUpdate::Skipped => result.skipped += 1,
        }
    }
    result
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) enum BudgetValueUpdate {
    Changed { monthly: String, yearly: String },
    Unchanged,
    Skipped,
}

pub(super) fn budget_values_for_period(
    monthly: &str,
    yearly: &str,
    period: BudgetValuePeriod,
) -> BudgetValueUpdate {
    let monthly = monthly.trim();
    let yearly = yearly.trim();
    let values = match period {
        BudgetValuePeriod::Monthly => match monthly_value_from(monthly, yearly) {
            Some(monthly) => Some((monthly, String::new())),
            None if monthly.is_empty() && !yearly.is_empty() => None,
            None => Some((String::new(), String::new())),
        },
        BudgetValuePeriod::Yearly => match yearly_value_from(monthly, yearly) {
            Some(yearly) => Some((String::new(), yearly)),
            None if yearly.is_empty() && !monthly.is_empty() => None,
            None => Some((String::new(), String::new())),
        },
    };

    let Some((new_monthly, new_yearly)) = values else {
        return BudgetValueUpdate::Skipped;
    };
    if new_monthly == monthly && new_yearly == yearly {
        BudgetValueUpdate::Unchanged
    } else {
        BudgetValueUpdate::Changed {
            monthly: new_monthly,
            yearly: new_yearly,
        }
    }
}

fn monthly_value_from(monthly: &str, yearly: &str) -> Option<String> {
    if !monthly.is_empty() {
        Some(monthly.to_string())
    } else if !yearly.is_empty() {
        convert_budget_amount_text(yearly, BudgetAmountConversion::YearlyToMonthly)
    } else {
        Some(String::new())
    }
}

fn yearly_value_from(monthly: &str, yearly: &str) -> Option<String> {
    if !yearly.is_empty() {
        Some(yearly.to_string())
    } else if !monthly.is_empty() {
        convert_budget_amount_text(monthly, BudgetAmountConversion::MonthlyToYearly)
    } else {
        Some(String::new())
    }
}

#[derive(Debug, Clone, Copy)]
enum BudgetAmountConversion {
    MonthlyToYearly,
    YearlyToMonthly,
}

fn convert_budget_amount_text(input: &str, conversion: BudgetAmountConversion) -> Option<String> {
    let amount = BudgetAmount::parse_optional(input)?;
    Some(match (amount, conversion) {
        (BudgetAmount::Fixed(amount), BudgetAmountConversion::MonthlyToYearly) => {
            format_budget_amount(BudgetAmount::Fixed(amount * Decimal::new(12, 0)))
        }
        (BudgetAmount::Fixed(amount), BudgetAmountConversion::YearlyToMonthly) => {
            format_budget_amount(BudgetAmount::Fixed(amount / Decimal::new(12, 0)))
        }
        (BudgetAmount::IncomePercent(percent), _) => {
            format_budget_amount(BudgetAmount::IncomePercent(percent))
        }
    })
}

fn format_budget_amount(amount: BudgetAmount) -> String {
    match amount {
        BudgetAmount::Fixed(amount) => amount.normalize().to_string(),
        BudgetAmount::IncomePercent(percent) => format!("{}%", percent.normalize()),
    }
}

pub(super) fn set_budget_bulk_status(
    status: &gtk::Label,
    changed: usize,
    skipped: usize,
    action: &str,
) {
    let action = tr(action);
    let message = match (changed, skipped) {
        (0, 0) => tr("No budget rows changed."),
        (changed, 0) => trf(
            "Updated {count} {action}. Review, then Save.",
            &[("count", changed.to_string()), ("action", action.clone())],
        ),
        (changed, skipped) => trf(
            "Updated {count} {action}; skipped {skipped} invalid value(s). Review, then Save.",
            &[
                ("count", changed.to_string()),
                ("action", action.clone()),
                ("skipped", skipped.to_string()),
            ],
        ),
    };
    status.set_text(&message);
}
