use super::LocalAiDraft;
use crate::data::{self, EditableRule, GeneratedConfiguration, GeneratedConfigurationSummary};
use crate::util::normalize_key;
use anyhow::Result;
use std::collections::BTreeSet;

pub(super) const LOCAL_AI_NOTE: &str = "Generated with local AI smart insights.";

pub(super) fn generated_configuration_from_draft(
    draft: LocalAiDraft,
    fallback: &GeneratedConfiguration,
) -> Result<Option<GeneratedConfiguration>> {
    if draft_configuration_is_empty(&draft) {
        return Ok(None);
    }

    let mut configuration = fallback.clone();
    merge_draft(&mut configuration, draft);
    if &configuration == fallback {
        return Ok(None);
    }

    refresh_summary(&mut configuration, fallback);
    data::validate_generated_configuration(&configuration)?;
    Ok(Some(configuration))
}

fn draft_configuration_is_empty(draft: &LocalAiDraft) -> bool {
    draft.budgets.is_empty()
        && draft.rules.is_empty()
        && draft.aliases.is_empty()
        && draft.ignored_patterns.is_empty()
}

fn merge_draft(configuration: &mut GeneratedConfiguration, draft: LocalAiDraft) {
    let mut budget_codes = configuration
        .budgets
        .iter()
        .map(|budget| normalize_key(&budget.code))
        .collect::<BTreeSet<_>>();
    for mut budget in draft.budgets {
        if budget_codes.insert(normalize_key(&budget.code)) {
            budget.notes = local_ai_note_with_detail(&budget.notes);
            configuration.budgets.push(budget);
        }
    }

    let mut rule_keys = configuration
        .rules
        .iter()
        .map(rule_key)
        .collect::<BTreeSet<_>>();
    for mut rule in draft.rules {
        if rule_keys.insert(rule_key(&rule)) {
            rule.notes = local_ai_note_with_detail(&rule.notes);
            configuration.rules.push(rule);
        }
    }

    let mut alias_keys = configuration
        .aliases
        .iter()
        .map(|alias| (normalize_key(&alias.canonical), normalize_key(&alias.alias)))
        .collect::<BTreeSet<_>>();
    for alias in draft.aliases {
        if alias_keys.insert((normalize_key(&alias.canonical), normalize_key(&alias.alias))) {
            configuration.aliases.push(alias);
        }
    }

    let mut ignored_keys = configuration
        .ignored_patterns
        .iter()
        .map(|pattern| pattern.key.trim().to_string())
        .collect::<BTreeSet<_>>();
    for pattern in draft.ignored_patterns {
        if ignored_keys.insert(pattern.key.trim().to_string()) {
            configuration.ignored_patterns.push(pattern);
        }
    }
}

fn rule_key(rule: &EditableRule) -> (String, String, String, String) {
    (
        normalize_key(&rule.field),
        normalize_key(&rule.search),
        normalize_key(&rule.budget_code),
        normalize_key(&rule.direction),
    )
}

fn refresh_summary(configuration: &mut GeneratedConfiguration, fallback: &GeneratedConfiguration) {
    configuration.summary = GeneratedConfigurationSummary {
        budgets: configuration.budgets.len(),
        rules: configuration.rules.len(),
        field_mappings: configuration.aliases.len(),
        ignored_patterns: configuration.ignored_patterns.len(),
        complete_years: fallback.summary.complete_years,
        budget_months: fallback.summary.budget_months,
    };
}

fn local_ai_note_with_detail(notes: &str) -> String {
    let notes = notes.trim();
    if notes.is_empty() || notes == LOCAL_AI_NOTE {
        LOCAL_AI_NOTE.to_string()
    } else {
        format!("{LOCAL_AI_NOTE} {notes}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{EditableBudget, EditableRule};

    #[test]
    fn draft_merges_with_fallback_without_renoting_fallback() {
        let fallback = GeneratedConfiguration {
            budgets: vec![budget("FOOD", "Food", "keep")],
            rules: Vec::new(),
            aliases: Vec::new(),
            ignored_patterns: Vec::new(),
            summary: GeneratedConfigurationSummary {
                budgets: 1,
                complete_years: 1,
                ..GeneratedConfigurationSummary::default()
            },
        };
        let draft = LocalAiDraft {
            budgets: vec![budget("TRAN", "Transport", "adapter")],
            rules: vec![rule("Train", "Transport", "TRAN")],
            ..LocalAiDraft::default()
        };

        let generated = generated_configuration_from_draft(draft, &fallback)
            .expect("merged draft should validate")
            .expect("new AI items should change the fallback");

        assert_eq!(generated.budgets.len(), 2);
        assert_eq!(generated.budgets[0].notes, "keep");
        assert!(generated.budgets[1].notes.starts_with(LOCAL_AI_NOTE));
        assert_eq!(generated.summary.complete_years, 1);
        assert_eq!(generated.summary.rules, 1);
    }

    fn budget(code: &str, category: &str, notes: &str) -> EditableBudget {
        EditableBudget {
            code: code.to_string(),
            category: category.to_string(),
            monthly_budget: "0".to_string(),
            yearly_budget: String::new(),
            direction: "expense".to_string(),
            income_basis: "real".to_string(),
            notes: notes.to_string(),
        }
    }

    fn rule(search: &str, category: &str, budget_code: &str) -> EditableRule {
        EditableRule {
            priority: 120,
            active: true,
            field: "any".to_string(),
            search: search.to_string(),
            is_regex: false,
            category: category.to_string(),
            budget_code: budget_code.to_string(),
            direction: "expense".to_string(),
            amount_min: String::new(),
            amount_max: String::new(),
            notes: String::new(),
        }
    }
}
