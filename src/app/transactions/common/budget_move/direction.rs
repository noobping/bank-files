use super::*;

pub(in crate::app::transactions::common) fn transaction_budget_direction_change(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
    new_code: &str,
    new_category: &str,
    new_direction: &str,
) -> Option<BudgetDirectionChange> {
    let from = transaction_budget_direction(tx, budgets);
    let to = BudgetDirection::parse(new_direction, new_code, new_category);
    budget_direction_change(
        &budget_code_change_label(&tx.budget_code, new_code),
        from,
        to,
    )
}

pub(in crate::app::transactions::common) fn transaction_budget_direction(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
) -> BudgetDirection {
    if analytics::transaction_is_transfer(tx, budgets) {
        return BudgetDirection::Transfer;
    }

    let code = tx.budget_code.trim();
    if let Some(budget) = budgets
        .iter()
        .find(|budget| budget.code.trim().eq_ignore_ascii_case(code))
    {
        return budget.direction;
    }

    if code.is_empty() {
        if tx.amount > Decimal::ZERO {
            BudgetDirection::Income
        } else {
            BudgetDirection::Expense
        }
    } else {
        BudgetDirection::parse("", code, &tx.category)
    }
}

pub(in crate::app::transactions::common) fn budget_code_change_label(
    old_code: &str,
    new_code: &str,
) -> String {
    let old_code = old_code.trim();
    let new_code = new_code.trim();
    match (old_code.is_empty(), new_code.is_empty()) {
        (true, true) => tr("this transaction"),
        (true, false) => trf(
            "this transaction -> {new}",
            &[("new", new_code.to_string())],
        ),
        (false, true) => trf("{old} -> no budget code", &[("old", old_code.to_string())]),
        (false, false) => trf(
            "{old} -> {new}",
            &[("old", old_code.to_string()), ("new", new_code.to_string())],
        ),
    }
}

pub(in crate::app::transactions::common) fn rule_field_label(field: &str) -> &'static str {
    match field {
        "counterparty" => "Counterparty",
        "description" => "Description",
        "tags" => "Tags",
        "account" => "Account",
        "transaction_id" => "Transaction ID",
        _ => "Everything",
    }
}
