use super::*;

mod budget_move;
mod detail_actions;
mod detail_primary;
mod detail_state;
mod filter;
mod form_options;
mod list;
mod page;
mod rule_dialog;
mod rule_helpers;
mod rule_ops;
mod search;
mod text;

#[cfg(test)]
mod tests;

pub(in crate::app) use filter::filtered_transactions;
pub(in crate::app) use list::transaction_list;
pub(in crate::app) use page::{
    append_page_header, current_page_snapshot, current_real_page_snapshot, search_empty_page,
};
pub(crate) use text::transaction_search_text;

#[cfg(test)]
use budget_move::{
    transaction_budget_direction_change, transaction_budget_more_options_visible,
    transaction_budget_move_available, transaction_budget_move_dialog_title,
    transaction_budget_move_form_values_changed,
    transaction_budget_move_list_max_height_for_window, transaction_budget_move_targets,
    transaction_budget_target_allowed, transaction_budget_target_is_changed,
    transaction_budget_target_is_current, transaction_budget_target_search_keywords,
    transaction_budget_target_subtitle, transaction_is_markable_as_transfer,
    TransactionBudgetTarget,
};
#[cfg(test)]
use detail_state::{
    transaction_detail_move_budget_code_placement, visible_transaction_detail_actions,
    TransactionDetailAction, TransactionDetailActionPlacement,
};
#[cfg(test)]
use list::{transaction_detail_rows, TransactionDetailRow};
#[cfg(test)]
use rule_helpers::invalid_auto_detection_rule_for_transaction;
