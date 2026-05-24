use super::LocalAiDraft;
use crate::data::{self, GeneratedConfiguration, GeneratedConfigurationSummary};
use anyhow::Result;

pub(super) const LOCAL_AI_NOTE: &str = "Generated with local AI smart insights.";

pub(super) fn generated_configuration_from_draft(
    draft: LocalAiDraft,
    fallback: &GeneratedConfiguration,
) -> Result<Option<GeneratedConfiguration>> {
    if draft.budgets.is_empty()
        && draft.rules.is_empty()
        && draft.aliases.is_empty()
        && draft.ignored_patterns.is_empty()
    {
        return Ok(None);
    }

    let summary = GeneratedConfigurationSummary {
        budgets: draft.budgets.len(),
        rules: draft.rules.len(),
        field_mappings: draft.aliases.len(),
        ignored_patterns: draft.ignored_patterns.len(),
        complete_years: fallback.summary.complete_years,
        budget_months: fallback.summary.budget_months,
    };
    let mut configuration = GeneratedConfiguration {
        rules: draft.rules,
        budgets: draft.budgets,
        aliases: draft.aliases,
        ignored_patterns: draft.ignored_patterns,
        summary,
    };
    normalize_generated_notes(&mut configuration);
    data::validate_generated_configuration(&configuration)?;
    Ok(Some(configuration))
}

fn normalize_generated_notes(configuration: &mut GeneratedConfiguration) {
    for budget in &mut configuration.budgets {
        budget.notes = local_ai_note_with_detail(&budget.notes);
    }
    for rule in &mut configuration.rules {
        rule.notes = local_ai_note_with_detail(&rule.notes);
    }
}

fn local_ai_note_with_detail(notes: &str) -> String {
    let notes = notes.trim();
    if notes.is_empty() || notes == LOCAL_AI_NOTE {
        LOCAL_AI_NOTE.to_string()
    } else {
        format!("{LOCAL_AI_NOTE} {notes}")
    }
}
