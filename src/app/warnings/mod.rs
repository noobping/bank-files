use super::core::{tr, trf};
use crate::analytics;
use crate::model::{AppData, BudgetCode, ComparisonMode, MonthKey, Transaction};
use crate::util::money;
use rust_decimal::Decimal;
use std::collections::HashMap;

mod annual;
mod evaluator;
mod message;
mod monthly;

#[cfg(test)]
mod tests;

pub(in crate::app) use annual::annual_budget_attention_warnings;
pub(in crate::app) use message::{
    attention_warning_card_message, attention_warning_messages, AttentionWarning,
    BudgetWarningTotals,
};
pub(in crate::app) use monthly::monthly_budget_attention_warnings;
