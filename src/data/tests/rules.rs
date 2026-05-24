use super::*;

#[test]
fn editable_rules_roundtrip_simple_and_regex_patterns() {
    let rules = vec![
        EditableRule {
            search: "Albert Heijn".to_string(),
            is_regex: false,
            ..EditableRule::new_default()
        },
        EditableRule {
            priority: 90,
            search: "(?i)github|openai".to_string(),
            is_regex: true,
            category: "Software".to_string(),
            budget_code: "CLOUD".to_string(),
            ..EditableRule::new_default()
        },
    ];

    let csv = serialize_editable_rules(&rules).unwrap();
    let parsed = parse_editable_rules(&csv).unwrap();

    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].search, "Albert Heijn");
    assert!(!parsed[0].is_regex);
    assert_eq!(parsed[1].search, "(?i)github|openai");
    assert!(parsed[1].is_regex);
}

#[test]
fn simple_text_patterns_are_regex_escaped_on_save() {
    let rule = EditableRule {
        search: "C++ winkel?".to_string(),
        is_regex: false,
        ..EditableRule::new_default()
    };

    let csv = serialize_editable_rules(&[rule]).unwrap();

    assert!(csv.contains("C\\+\\+ winkel\\?"));
}

#[test]
fn group_rules_for_combining_moves_compatible_rules_together() {
    let groceries_alpha = EditableRule {
        search: "alpha".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };
    let software = EditableRule {
        search: "hosting".to_string(),
        category: "Software".to_string(),
        budget_code: "CLOUD".to_string(),
        ..EditableRule::new_default()
    };
    let groceries_beta = EditableRule {
        search: "beta".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };

    let report = group_editable_rules_for_combining(&[groceries_alpha, software, groceries_beta]);

    assert!(report.changed);
    assert_eq!(report.grouped_groups, 1);
    assert_eq!(report.rules[0].search, "alpha");
    assert_eq!(report.rules[1].search, "beta");
    assert_eq!(report.rules[2].search, "hosting");
    assert_eq!(combine_editable_rules(&report.rules).after_count, 2);
}

#[test]
fn group_rules_for_combining_detects_already_grouped_rules() {
    let first = EditableRule {
        search: "alpha".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };
    let second = EditableRule {
        search: "beta".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };

    let report = group_editable_rules_for_combining(&[first, second]);

    assert!(!report.changed);
    assert_eq!(report.grouped_groups, 1);
}

#[test]
fn group_rules_for_combining_ignores_singletons() {
    let groceries = EditableRule {
        search: "alpha".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };
    let software = EditableRule {
        search: "hosting".to_string(),
        category: "Software".to_string(),
        budget_code: "CLOUD".to_string(),
        ..EditableRule::new_default()
    };

    let report = group_editable_rules_for_combining(&[groceries, software]);

    assert!(!report.changed);
    assert_eq!(report.grouped_groups, 0);
}

#[test]
fn combine_rules_merges_adjacent_plain_rules() {
    let first = EditableRule {
        search: "alpha".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };
    let second = EditableRule {
        search: "beta".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };

    let report = combine_editable_rules(&[first, second]);

    assert_eq!(report.before_count, 2);
    assert_eq!(report.after_count, 1);
    assert_eq!(report.combined_groups, 1);
    assert!(report.rules[0].is_regex);
    assert_eq!(report.rules[0].search, "(?:alpha|beta)");
}

#[test]
fn combine_rules_expands_existing_clean_regex() {
    let regex_rule = EditableRule {
        search: "(?:alpha|beta)".to_string(),
        is_regex: true,
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };
    let plain_rule = EditableRule {
        search: "gamma".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };

    let report = combine_editable_rules(&[regex_rule, plain_rule]);

    assert_eq!(report.after_count, 1);
    assert_eq!(report.rules[0].search, "(?:alpha|beta|gamma)");
}

#[test]
fn combine_rules_builds_literal_regex_with_flexible_spaces_and_shared_prefix() {
    let first = EditableRule {
        search: "Albert  Heijn Amsterdam".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };
    let second = EditableRule {
        search: "Albert Heijn Utrecht".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };

    let report = combine_editable_rules(&[first, second]);

    assert_eq!(report.after_count, 1);
    assert_eq!(
        report.rules[0].search,
        r"Albert\s+Heijn(?:\s+Amsterdam|\s+Utrecht)"
    );
    assert!(report.rules[0].is_regex);
}

#[test]
fn combine_rules_merges_existing_regex_terms() {
    let first = EditableRule {
        search: r"(?:amazon(?: marketplace)?|bol\.com)".to_string(),
        is_regex: true,
        category: "Shopping".to_string(),
        budget_code: "SHOP".to_string(),
        ..EditableRule::new_default()
    };
    let second = EditableRule {
        search: r"(?:bol\.com|coolblue)".to_string(),
        is_regex: true,
        category: "Shopping".to_string(),
        budget_code: "SHOP".to_string(),
        ..EditableRule::new_default()
    };

    let report = combine_editable_rules(&[first, second]);

    assert_eq!(report.after_count, 1);
    assert_eq!(
        report.rules[0].search,
        r"(?:amazon(?: marketplace)?|bol\.com|coolblue)"
    );
    assert!(report.rules[0].is_regex);
}

#[test]
fn combine_rules_merges_case_insensitive_regex_wrappers() {
    let regex_rule = EditableRule {
        search: "(?i:github|openai)".to_string(),
        is_regex: true,
        category: "Software".to_string(),
        budget_code: "CLOUD".to_string(),
        ..EditableRule::new_default()
    };
    let plain_rule = EditableRule {
        search: "Anthropic".to_string(),
        category: "Software".to_string(),
        budget_code: "CLOUD".to_string(),
        ..EditableRule::new_default()
    };

    let report = combine_editable_rules(&[regex_rule, plain_rule]);

    assert_eq!(report.after_count, 1);
    assert_eq!(report.rules[0].search, "(?:github|openai|Anthropic)");
    assert!(report.rules[0].is_regex);
}

#[test]
fn combine_rules_keeps_non_adjacent_rules_in_place() {
    let groceries_alpha = EditableRule {
        search: "alpha".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };
    let software = EditableRule {
        search: "hosting".to_string(),
        category: "Software".to_string(),
        budget_code: "CLOUD".to_string(),
        ..EditableRule::new_default()
    };
    let groceries_beta = EditableRule {
        search: "beta".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };

    let report = combine_editable_rules(&[groceries_alpha, software, groceries_beta]);

    assert_eq!(report.after_count, 3);
    assert_eq!(report.combined_groups, 0);
}
