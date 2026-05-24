use super::*;
use crate::analytics;

mod aliases;
mod budget_code;
mod patterns;
mod period;
mod rule_fields;
mod rules;

use aliases::generated_field_aliases;
pub use budget_code::generated_budget_code_for_category;
use patterns::{cancellation_pattern_keys, generated_ignored_patterns, transfer_pattern_keys};
use period::{complete_period_transactions, generation_period, uncategorized_transactions};
use rules::{
    append_grouped_transaction_configuration, append_transfer_configuration, generated_rule_order,
};

pub(super) const TRANSFER_CODE: &str = "TRANSFER";
pub(super) const PLANNED_INCOME_CODE: &str = "INC";
pub(super) const GENERATED_NOTE: &str =
    "Generated from {count} complete imported year(s), covering {months} month(s), on {date}.";
pub(super) const GENERATED_RULE_NOTE: &str = "Generated from configuration generation.";
pub(super) const SMART_INSIGHTS_REQUIRED_MESSAGE: &str =
    "Configuration generation requires Smart Insights.";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeneratedConfigurationSummary {
    pub budgets: usize,
    pub rules: usize,
    pub field_mappings: usize,
    pub ignored_patterns: usize,
    pub complete_years: usize,
    pub budget_months: usize,
}

impl GeneratedConfigurationSummary {
    pub fn is_empty(&self) -> bool {
        self.budgets == 0
            && self.rules == 0
            && self.field_mappings == 0
            && self.ignored_patterns == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedConfiguration {
    pub rules: Vec<EditableRule>,
    pub budgets: Vec<EditableBudget>,
    pub aliases: Vec<EditableAlias>,
    pub ignored_patterns: Vec<IgnoredTransactionPattern>,
    pub summary: GeneratedConfigurationSummary,
}

pub fn generate_automatic_configuration(
    data: &AppData,
    smart_insights_enabled: bool,
) -> Result<GeneratedConfiguration> {
    ensure_smart_insights_for_automatic_configuration(smart_insights_enabled)?;

    let transactions = uncategorized_transactions(&data.transactions);
    let period = generation_period(&transactions);
    let analysis_transactions = complete_period_transactions(&transactions, &period);
    let patterns = analytics::transaction_pattern_analysis(
        &analysis_transactions,
        data.dedupe_mode.is_enabled(),
    );
    let ignored_patterns = generated_ignored_patterns(&patterns.patterns);
    let mut excluded_keys = cancellation_pattern_keys(&patterns.patterns);
    excluded_keys.extend(transfer_pattern_keys(&patterns.patterns));

    let mut reserved_codes = BTreeSet::<String>::new();
    let mut budgets = Vec::new();
    let mut rules = Vec::new();
    append_transfer_configuration(
        &analysis_transactions,
        &patterns.patterns,
        &mut reserved_codes,
        &mut budgets,
        &mut rules,
    );
    append_grouped_transaction_configuration(
        &analysis_transactions,
        &excluded_keys,
        &period,
        &mut reserved_codes,
        &mut budgets,
        &mut rules,
    );

    rules.sort_by(generated_rule_order);
    for (offset, rule) in rules.iter_mut().enumerate() {
        rule.priority = 200 - offset as i32;
    }

    let (aliases, generated_alias_count) = generated_field_aliases(&data.reports)?;
    let summary = GeneratedConfigurationSummary {
        budgets: budgets.len(),
        rules: rules.len(),
        field_mappings: generated_alias_count,
        ignored_patterns: ignored_patterns.len(),
        complete_years: period.year_count(),
        budget_months: period.month_count(),
    };

    Ok(GeneratedConfiguration {
        rules,
        budgets,
        aliases,
        ignored_patterns,
        summary,
    })
}

fn ensure_smart_insights_for_automatic_configuration(smart_insights_enabled: bool) -> Result<()> {
    if cfg!(feature = "smart-insights") && smart_insights_enabled {
        Ok(())
    } else {
        anyhow::bail!(SMART_INSIGHTS_REQUIRED_MESSAGE)
    }
}

#[cfg(all(test, not(feature = "smart-insights")))]
mod smart_insights_disabled_tests;
#[cfg(all(test, feature = "smart-insights"))]
mod tests;
