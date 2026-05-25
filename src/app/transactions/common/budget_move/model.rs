use super::direction::rule_field_label;
use super::*;

pub(in crate::app::transactions::common) fn transaction_budget_move_dialog_title(
    match_name: &str,
    fallback_title: &str,
) -> String {
    if match_name.trim().is_empty() {
        tr(fallback_title)
    } else {
        match_name.to_string()
    }
}

pub(in crate::app::transactions::common) fn transaction_budget_move_match_summary(
    initial: &EditableRule,
) -> String {
    trf(
        "Matching by {field}: {value}",
        &[
            ("field", tr(rule_field_label(&initial.field))),
            ("value", truncate(&initial.search, 80)),
        ],
    )
}

pub(in crate::app::transactions::common) fn transaction_budget_move_list_min_height(
    row_count: usize,
) -> i32 {
    let visible_rows = row_count.clamp(3, 7) as i32;
    72 + visible_rows * 54
}

pub(in crate::app::transactions::common) fn transaction_budget_move_list_max_height(
    window: &impl IsA<gtk::Widget>,
) -> i32 {
    transaction_budget_move_list_max_height_for_window(window.as_ref().allocated_height())
}

pub(in crate::app::transactions::common) fn transaction_budget_move_list_max_height_for_window(
    window_height: i32,
) -> i32 {
    if window_height <= 0 {
        return 620;
    }
    window_height.saturating_sub(96).clamp(180, 900)
}

pub(in crate::app::transactions::common) fn select_transaction_budget_target_row(
    rows: &[TransactionBudgetTargetRow],
    selected_target: &Rc<RefCell<Option<TransactionBudgetTarget>>>,
    save_button: &gtk::Button,
    tx: &Transaction,
    advanced_features: bool,
    selected_index: usize,
) {
    let mut selected = None;
    for (index, row) in rows.iter().enumerate() {
        let row_selected = index == selected_index;
        row.check.set_visible(row_selected);
        if row_selected {
            selected = Some(row.target.clone());
        }
    }
    save_button.set_sensitive(
        selected.as_ref().is_some_and(|target| {
            transaction_budget_target_is_changed(tx, target, advanced_features)
        }),
    );
    *selected_target.borrow_mut() = selected;
}

#[derive(Clone)]
pub(in crate::app::transactions::common) struct TransactionBudgetTargetRow {
    pub(in crate::app::transactions::common) row: adw::ActionRow,
    pub(in crate::app::transactions::common) check: gtk::Image,
    pub(in crate::app::transactions::common) target: TransactionBudgetTarget,
}

#[derive(Clone)]
pub(in crate::app::transactions::common) struct TransactionBudgetMoveValues {
    pub(in crate::app::transactions::common) category: String,
    pub(in crate::app::transactions::common) budget_code: String,
    pub(in crate::app::transactions::common) direction: String,
}

pub(in crate::app::transactions::common) fn transaction_budget_move_rule(
    initial: &EditableRule,
    values: TransactionBudgetMoveValues,
    advanced_features: bool,
) -> EditableRule {
    EditableRule {
        priority: 140,
        active: true,
        field: initial.field.clone(),
        search: initial.search.clone(),
        is_regex: initial.is_regex,
        category: values.category,
        budget_code: values.budget_code,
        direction: values.direction,
        amount_min: initial.amount_min.clone(),
        amount_max: initial.amount_max.clone(),
        notes: tr(if advanced_features {
            "Generated from transaction budget code change."
        } else {
            "Generated from transaction category change."
        }),
    }
}

pub(in crate::app::transactions::common) fn transaction_budget_target_title(
    target: &TransactionBudgetTarget,
    advanced_features: bool,
) -> String {
    if !target.category.trim().is_empty() {
        target.category.clone()
    } else if advanced_features {
        target.code.clone()
    } else {
        tr("Uncategorized")
    }
}

pub(in crate::app::transactions::common) fn transaction_budget_target_subtitle(
    target: &TransactionBudgetTarget,
    advanced_features: bool,
) -> String {
    let mut parts = Vec::new();
    if advanced_features {
        let code = target.code.trim();
        if !code.is_empty() {
            parts.push(code.to_string());
        }
    }

    parts.push(transaction_budget_direction_label(target.direction));

    let description = target.description.trim();
    if !description.is_empty() {
        parts.push(description.to_string());
    }

    parts.join(" · ")
}

pub(in crate::app::transactions::common) fn transaction_budget_direction_label(
    direction: BudgetDirection,
) -> String {
    tr(match direction {
        BudgetDirection::Income => "Income",
        BudgetDirection::Transfer => "Transfers",
        BudgetDirection::Expense => "Expenses",
    })
}

pub(in crate::app::transactions::common) fn transaction_budget_target_search_keywords(
    target: &TransactionBudgetTarget,
    advanced_features: bool,
    match_summary: &str,
) -> Vec<String> {
    let mut keywords = vec![
        target.category.clone(),
        target.description.clone(),
        transaction_budget_direction_label(target.direction),
        match_summary.to_string(),
    ];
    if advanced_features {
        keywords.push(target.code.clone());
    }
    keywords
}

pub(in crate::app::transactions::common) fn transaction_budget_target_is_current(
    tx: &Transaction,
    target: &TransactionBudgetTarget,
    advanced_features: bool,
) -> bool {
    if !advanced_features
        && !tx.category.trim().is_empty()
        && target
            .category
            .trim()
            .eq_ignore_ascii_case(tx.category.trim())
    {
        return true;
    }

    target
        .code
        .trim()
        .eq_ignore_ascii_case(tx.budget_code.trim())
}

pub(in crate::app::transactions::common) fn transaction_budget_target_is_changed(
    tx: &Transaction,
    target: &TransactionBudgetTarget,
    advanced_features: bool,
) -> bool {
    !transaction_budget_target_is_current(tx, target, advanced_features)
}

pub(in crate::app::transactions::common) fn transaction_budget_more_options_visible(
    advanced_features: bool,
) -> bool {
    advanced_features
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(in crate::app::transactions::common) struct TransactionBudgetTarget {
    pub(in crate::app::transactions::common) code: String,
    pub(in crate::app::transactions::common) category: String,
    pub(in crate::app::transactions::common) description: String,
    pub(in crate::app::transactions::common) direction: BudgetDirection,
}

pub(in crate::app::transactions::common) fn transaction_budget_move_targets(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
    advanced_features: bool,
) -> Vec<TransactionBudgetTarget> {
    let simple_direction = transaction_budget_simple_move_direction(tx, budgets);
    budgets
        .iter()
        .filter_map(|budget| {
            let code = budget.code.trim();
            if code.is_empty() {
                return None;
            }
            let target = TransactionBudgetTarget {
                code: code.to_string(),
                category: budget.category.trim().to_string(),
                description: budget.notes.trim().to_string(),
                direction: budget.direction,
            };
            transaction_budget_target_allowed(tx, budgets, &target, advanced_features)
                .then_some(target)
        })
        .filter(|target| advanced_features || target.direction == simple_direction)
        .collect()
}

pub(in crate::app::transactions::common) fn transaction_budget_simple_move_direction(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
) -> BudgetDirection {
    if analytics::transaction_is_transfer(tx, budgets) {
        BudgetDirection::Transfer
    } else if tx.amount > Decimal::ZERO {
        BudgetDirection::Income
    } else {
        BudgetDirection::Expense
    }
}

pub(in crate::app::transactions::common) fn transaction_budget_target_allowed(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
    target: &TransactionBudgetTarget,
    advanced_features: bool,
) -> bool {
    advanced_features || target.direction == transaction_budget_simple_move_direction(tx, budgets)
}

pub(in crate::app::transactions::common) fn transaction_budget_move_available(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
    advanced_features: bool,
) -> bool {
    transaction_budget_move_targets(tx, budgets, advanced_features)
        .iter()
        .any(|target| transaction_budget_target_is_changed(tx, target, advanced_features))
}

pub(in crate::app::transactions::common) fn transaction_is_markable_as_transfer(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
) -> bool {
    !analytics::transaction_is_transfer(tx, budgets)
}
