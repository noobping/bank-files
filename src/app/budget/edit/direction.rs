use super::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(in crate::app) struct BudgetDirectionChange {
    pub(in crate::app) item: String,
}

pub(in crate::app) fn budget_direction_change(
    item: &str,
    from: BudgetDirection,
    to: BudgetDirection,
) -> Option<BudgetDirectionChange> {
    if budget_direction_change_needs_confirmation(from, to) {
        Some(BudgetDirectionChange {
            item: item.trim().to_string(),
        })
    } else {
        None
    }
}

fn budget_direction_change_needs_confirmation(from: BudgetDirection, to: BudgetDirection) -> bool {
    matches!(
        (from, to),
        (BudgetDirection::Expense, BudgetDirection::Income)
            | (BudgetDirection::Income, BudgetDirection::Expense)
    )
}

pub(in crate::app) fn confirm_budget_direction_changes<F>(
    parent: &impl IsA<gtk::Widget>,
    changes: Vec<BudgetDirectionChange>,
    on_confirm: F,
) where
    F: FnOnce() + 'static,
{
    if changes.is_empty() {
        on_confirm();
        return;
    }

    let heading = if changes.len() == 1 {
        tr("Change budget direction?")
    } else {
        tr("Change budget directions?")
    };
    let body = budget_direction_change_confirmation_body(&changes);
    let dialog = ui::alert_dialog(heading, body)
        .responses(&[
            ui::AlertResponse::neutral("cancel", "Cancel"),
            ui::AlertResponse::destructive("change", "Change Direction"),
        ])
        .close_response("cancel")
        .default_response("cancel")
        .build();
    dialog.choose(
        Some(parent),
        None::<&gtk::gio::Cancellable>,
        move |response| {
            if response.as_str() == "change" {
                on_confirm();
            }
        },
    );
}

fn budget_direction_change_confirmation_body(changes: &[BudgetDirectionChange]) -> String {
    if let [change] = changes {
        if change.item.is_empty() {
            tr("This item will switch between expenses and income. This can move related transactions between spending and income totals.")
        } else {
            trf(
                "{item} will switch between expenses and income. This can move related transactions between spending and income totals.",
                &[("item", change.item.clone())],
            )
        }
    } else {
        trf(
            "{count} items will switch between expenses and income. This can move related transactions between spending and income totals.",
            &[("count", changes.len().to_string())],
        )
    }
}

pub(in crate::app) fn budget_direction_editable(advanced_features: bool, persisted: bool) -> bool {
    advanced_features || !persisted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_direction_editability_allows_simple_new_budgets() {
        assert!(budget_direction_editable(false, false));
        assert!(!budget_direction_editable(false, true));
        assert!(budget_direction_editable(true, true));
    }

    #[test]
    fn budget_direction_change_confirms_expense_income_crossing() {
        assert!(
            budget_direction_change("FOOD", BudgetDirection::Expense, BudgetDirection::Income,)
                .is_some()
        );
        assert!(budget_direction_change(
            "SALARY",
            BudgetDirection::Income,
            BudgetDirection::Expense,
        )
        .is_some());
    }

    #[test]
    fn budget_direction_change_ignores_same_direction_and_transfers() {
        assert!(budget_direction_change(
            "FOOD",
            BudgetDirection::Expense,
            BudgetDirection::Expense,
        )
        .is_none());
        assert!(budget_direction_change(
            "SAVE",
            BudgetDirection::Expense,
            BudgetDirection::Transfer,
        )
        .is_none());
        assert!(budget_direction_change(
            "SAVE",
            BudgetDirection::Transfer,
            BudgetDirection::Income,
        )
        .is_none());
    }
}
