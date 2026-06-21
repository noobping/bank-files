use super::*;

pub(in crate::app::warnings) fn budget_attention_warnings(
    totals: BudgetWarningTotals,
) -> Vec<AttentionWarning> {
    [
        actual_spending_warning(totals)
            .map(|message| AttentionWarning::new("Spending is above income", message)),
        planned_expenses_warning(totals)
            .map(|message| AttentionWarning::new("Check your budget", message)),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn planned_expenses_warning(totals: BudgetWarningTotals) -> Option<String> {
    if totals.planned_expenses <= Decimal::ZERO || totals.planned_expenses <= totals.planned_income
    {
        return None;
    }

    if totals.planned_income <= Decimal::ZERO {
        return Some(trf(
            "Planned expenses total {expenses}, but this period has no planned income.",
            &[("expenses", money(totals.planned_expenses))],
        ));
    }

    Some(trf(
        "Planned expenses total {expenses}, above planned income of {income} by {overage}.",
        &[
            ("expenses", money(totals.planned_expenses)),
            ("income", money(totals.planned_income)),
            (
                "overage",
                money(totals.planned_expenses - totals.planned_income),
            ),
        ],
    ))
}

fn actual_spending_warning(totals: BudgetWarningTotals) -> Option<String> {
    let threshold = totals.real_income.max(totals.planned_income);
    let annual_budget_room = totals
        .annual_budget_room_used
        .max(Decimal::ZERO)
        .min(totals.real_expenses.max(Decimal::ZERO));
    let spending_limit = threshold + annual_budget_room;
    if totals.real_expenses <= spending_limit {
        return None;
    }

    let loss = totals.real_expenses - spending_limit;
    if annual_budget_room > Decimal::ZERO {
        return Some(actual_spending_above_income_and_annual_room_warning(
            totals,
            threshold,
            annual_budget_room,
            loss,
        ));
    }

    if threshold <= Decimal::ZERO {
        return Some(trf(
            "Expenses are {expenses}, with no real or planned income in this period.",
            &[("expenses", money(totals.real_expenses))],
        ));
    }

    if totals.real_income >= totals.planned_income {
        return Some(trf(
            "Expenses are {expenses}, above real income of {income} by {loss}.",
            &[
                ("expenses", money(totals.real_expenses)),
                ("income", money(totals.real_income)),
                ("loss", money(totals.real_expenses - totals.real_income)),
            ],
        ));
    }

    Some(trf(
        "Expenses are {expenses}, above planned income of {income} by {loss}.",
        &[
            ("expenses", money(totals.real_expenses)),
            ("income", money(totals.planned_income)),
            ("loss", money(totals.real_expenses - totals.planned_income)),
        ],
    ))
}

fn actual_spending_above_income_and_annual_room_warning(
    totals: BudgetWarningTotals,
    threshold: Decimal,
    annual_budget_room: Decimal,
    loss: Decimal,
) -> String {
    if threshold <= Decimal::ZERO {
        return trf(
            "Expenses are {expenses}, above annual budget room of {room} by {loss}.",
            &[
                ("expenses", money(totals.real_expenses)),
                ("room", money(annual_budget_room)),
                ("loss", money(loss)),
            ],
        );
    }

    if totals.real_income >= totals.planned_income {
        return trf(
            "Expenses are {expenses}, above real income of {income} plus annual budget room of {room} by {loss}.",
            &[
                ("expenses", money(totals.real_expenses)),
                ("income", money(totals.real_income)),
                ("room", money(annual_budget_room)),
                ("loss", money(loss)),
            ],
        );
    }

    trf(
        "Expenses are {expenses}, above planned income of {income} plus annual budget room of {room} by {loss}.",
        &[
            ("expenses", money(totals.real_expenses)),
            ("income", money(totals.planned_income)),
            ("room", money(annual_budget_room)),
            ("loss", money(loss)),
        ],
    )
}

pub(in crate::app::warnings) fn positive_budget_total<I>(budget_amounts: I) -> Decimal
where
    I: IntoIterator<Item = Decimal>,
{
    budget_amounts
        .into_iter()
        .fold(Decimal::ZERO, |total, budget| {
            total + budget.max(Decimal::ZERO)
        })
}
