use super::*;

fn rule(search: &str, is_regex: bool) -> EditableRule {
    EditableRule {
        field: "counterparty".to_string(),
        search: search.to_string(),
        is_regex,
        category: "Transfers".to_string(),
        budget_code: "TRANSFER".to_string(),
        direction: "transfer".to_string(),
        ..EditableRule::new_default()
    }
}

fn rule_match_from(rule: &EditableRule, matched_text: &str) -> TransactionRuleMatch {
    TransactionRuleMatch {
        priority: rule.priority,
        field: rule.field.clone(),
        pattern: data::pattern_from_form(rule),
        matched_text: matched_text.to_string(),
        category: rule.category.clone(),
        budget_code: rule.budget_code.clone(),
        direction: rule.direction.clone(),
        amount_min: crate::util::parse_decimal(&rule.amount_min),
        amount_max: crate::util::parse_decimal(&rule.amount_max),
        notes: rule.notes.clone(),
    }
}

#[test]
fn undo_rule_match_removes_matching_keyword_from_combined_rule() {
    let transfer_rule = rule(r"(?:Savings|Shared\s+account|Card topup)", true);
    let mut rules = vec![transfer_rule.clone()];

    let change = undo_rule_match_in_editable_config(
        &mut rules,
        &rule_match_from(&transfer_rule, "Monthly shared account transfer"),
    );

    assert_eq!(change, Some(RuleUndoChange::Edited));
    assert_eq!(rules.len(), 1);
    assert_eq!(
        data::editable_rule_literal_terms(&rules[0]),
        Some(vec!["Savings".to_string(), "Card topup".to_string()]),
    );
}

#[test]
fn undo_rule_match_removes_rule_when_last_keyword_matches() {
    let transfer_rule = rule("Savings", false);
    let mut rules = vec![rule("Other", false), transfer_rule.clone()];

    let change = undo_rule_match_in_editable_config(
        &mut rules,
        &rule_match_from(&transfer_rule, "Savings transfer"),
    );

    assert_eq!(change, Some(RuleUndoChange::Removed));
    assert_eq!(rules, vec![rule("Other", false)]);
}

#[test]
fn undo_rule_match_does_not_remove_rule_when_keyword_is_unknown() {
    let transfer_rule = rule(r"(?:Savings|Shared\s+account)", true);
    let mut rules = vec![transfer_rule.clone()];

    let change = undo_rule_match_in_editable_config(
        &mut rules,
        &rule_match_from(&transfer_rule, "Different transfer"),
    );

    assert_eq!(change, None);
    assert_eq!(rules, vec![transfer_rule]);
}

#[test]
fn undo_rule_match_removes_uneditable_regex_rule() {
    let transfer_rule = rule(r"Savings.*Transfer", true);
    let mut rules = vec![transfer_rule.clone()];

    let change = undo_rule_match_in_editable_config(
        &mut rules,
        &rule_match_from(&transfer_rule, "Savings monthly Transfer"),
    );

    assert_eq!(change, Some(RuleUndoChange::Removed));
    assert!(rules.is_empty());
}
