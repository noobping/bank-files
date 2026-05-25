use super::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::app) enum RuleUndoChange {
    Edited,
    Removed,
}

enum KeywordEdit {
    Edited(EditableRule),
    RemoveRule,
    NoMatchingKeyword,
}

pub(in crate::app) fn undo_rule_match_config_change(
    rule_match: TransactionRuleMatch,
) -> anyhow::Result<RuleUndoChange> {
    let mut rules = data::load_editable_rules()?;
    let Some(change) = undo_rule_match_in_editable_config(&mut rules, &rule_match) else {
        anyhow::bail!(tr("The matching rule could not be edited."));
    };
    data::write_editable_rules(&rules)?;
    Ok(change)
}

pub(in crate::app) fn undo_rule_match_in_editable_config(
    rules: &mut Vec<EditableRule>,
    rule_match: &TransactionRuleMatch,
) -> Option<RuleUndoChange> {
    let index = rules
        .iter()
        .position(|rule| editable_rule_matches_transaction_rule(rule, rule_match))?;

    match rule_without_matching_keywords(&rules[index], rule_match) {
        KeywordEdit::Edited(rule) => {
            rules[index] = rule;
            Some(RuleUndoChange::Edited)
        }
        KeywordEdit::RemoveRule => {
            rules.remove(index);
            Some(RuleUndoChange::Removed)
        }
        KeywordEdit::NoMatchingKeyword => None,
    }
}

fn rule_without_matching_keywords(
    rule: &EditableRule,
    rule_match: &TransactionRuleMatch,
) -> KeywordEdit {
    let Some(terms) = data::editable_rule_literal_terms(rule) else {
        return KeywordEdit::RemoveRule;
    };
    if terms.is_empty() {
        return KeywordEdit::RemoveRule;
    }

    let remaining = terms
        .iter()
        .filter(|term| !literal_term_matches_text(term, &rule_match.matched_text))
        .cloned()
        .collect::<Vec<_>>();

    if remaining.len() == terms.len() {
        return KeywordEdit::NoMatchingKeyword;
    }
    if remaining.is_empty() {
        return KeywordEdit::RemoveRule;
    }

    let (search, is_regex) = data::rule_search_from_literal_terms(&remaining);
    KeywordEdit::Edited(EditableRule {
        search,
        is_regex,
        ..rule.clone()
    })
}

fn literal_term_matches_text(term: &str, text: &str) -> bool {
    let term = crate::util::normalize_key(term);
    let text = crate::util::normalize_key(text);
    !term.is_empty() && !text.is_empty() && text.contains(&term)
}

fn editable_rule_matches_transaction_rule(
    rule: &EditableRule,
    rule_match: &TransactionRuleMatch,
) -> bool {
    rule.priority == rule_match.priority
        && rule.active
        && rule.field.trim() == rule_match.field.trim()
        && data::pattern_from_form(rule).trim() == rule_match.pattern.trim()
        && rule.category.trim() == rule_match.category.trim()
        && rule
            .budget_code
            .trim()
            .eq_ignore_ascii_case(rule_match.budget_code.trim())
        && rule.direction.trim() == rule_match.direction.trim()
        && crate::util::parse_decimal(&rule.amount_min) == rule_match.amount_min
        && crate::util::parse_decimal(&rule.amount_max) == rule_match.amount_max
}

#[cfg(test)]
#[path = "config_rule_undo_tests.rs"]
mod tests;
