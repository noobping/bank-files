use super::super::*;
use super::files::{read_config_file, write_config_file};
use super::rule_search::combined_rule_search;
use super::rule_terms::{literal_rule_term, mergeable_rule_terms, normalize_literal_text};

#[derive(Debug, Clone)]
pub struct OrphanedRule {
    pub rule: EditableRule,
    pub budget_code: String,
}

pub fn orphaned_rules() -> Result<Vec<OrphanedRule>> {
    let budgets = load_editable_budgets()?;
    let budget_codes = configured_budget_codes(&budgets);
    Ok(load_editable_rules()?
        .into_iter()
        .filter_map(|rule| orphaned_rule(rule, &budget_codes))
        .collect())
}

pub fn remove_orphaned_rules() -> Result<usize> {
    let budgets = load_editable_budgets()?;
    let budget_codes = configured_budget_codes(&budgets);
    let mut rules = load_editable_rules()?;
    let original_len = rules.len();
    rules.retain(|rule| orphaned_rule(rule.clone(), &budget_codes).is_none());
    let removed = original_len.saturating_sub(rules.len());
    if removed > 0 {
        write_editable_rules(&rules)?;
    }
    Ok(removed)
}

pub(super) fn configured_budget_codes(budgets: &[EditableBudget]) -> HashSet<String> {
    budgets
        .iter()
        .map(|budget| normalize_key(&budget.code))
        .filter(|code| !code.is_empty())
        .collect()
}

pub(super) fn orphaned_rule(
    rule: EditableRule,
    budget_codes: &HashSet<String>,
) -> Option<OrphanedRule> {
    let budget_code = rule.budget_code.trim();
    if budget_code.is_empty() || budget_codes.contains(&normalize_key(budget_code)) {
        return None;
    }

    Some(OrphanedRule {
        budget_code: budget_code.to_string(),
        rule,
    })
}

pub fn load_editable_rules() -> Result<Vec<EditableRule>> {
    let (_, contents) = read_config_file("rules.csv")?;
    parse_editable_rules(&contents)
}

pub fn write_editable_rules(rules: &[EditableRule]) -> Result<PathBuf> {
    validate_editable_rules(rules)?;
    let contents = serialize_editable_rules(rules)?;
    write_config_file("rules.csv", &contents)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleCombineReport {
    pub rules: Vec<EditableRule>,
    pub before_count: usize,
    pub after_count: usize,
    pub combined_groups: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleGroupReport {
    pub rules: Vec<EditableRule>,
    pub changed: bool,
    pub grouped_groups: usize,
}

pub fn group_editable_rules_for_combining(rules: &[EditableRule]) -> RuleGroupReport {
    let mut grouped_rules = Vec::with_capacity(rules.len());
    let mut grouped = vec![false; rules.len()];
    let mut grouped_groups = 0;

    for index in 0..rules.len() {
        if grouped[index] {
            continue;
        }

        let key = RuleCombineKey::from(&rules[index]);
        let matching_indices = (index..rules.len())
            .filter(|candidate| {
                !grouped[*candidate] && RuleCombineKey::from(&rules[*candidate]) == key
            })
            .collect::<Vec<_>>();
        let mergeable_count = matching_indices
            .iter()
            .filter(|index| mergeable_rule_terms(&rules[**index]).is_some())
            .count();

        if mergeable_count > 1 {
            grouped_groups += 1;
            for matching_index in matching_indices {
                grouped[matching_index] = true;
                grouped_rules.push(rules[matching_index].clone());
            }
        } else {
            grouped[index] = true;
            grouped_rules.push(rules[index].clone());
        }
    }

    let changed = grouped_rules != rules;
    RuleGroupReport {
        rules: grouped_rules,
        changed,
        grouped_groups,
    }
}

pub fn editable_rule_literal_terms(rule: &EditableRule) -> Option<Vec<String>> {
    let search = rule.search.trim();
    if search.is_empty() {
        return Some(Vec::new());
    }
    if !rule.is_regex {
        return Some(vec![normalize_literal_text(search)]);
    }

    mergeable_rule_terms(rule)?
        .into_iter()
        .map(|term| term.literal)
        .collect()
}

pub fn rule_search_from_literal_terms(terms: &[String]) -> (String, bool) {
    let mut seen = HashSet::new();
    let mut literal_terms = Vec::new();

    for term in terms {
        let literal = normalize_literal_text(term);
        if literal.is_empty() || !seen.insert(literal.to_lowercase()) {
            continue;
        }
        literal_terms.push(literal_rule_term(&literal));
    }

    match literal_terms.as_slice() {
        [] => (String::new(), false),
        [term] => (term.literal.clone().unwrap_or_default(), false),
        terms => (combined_rule_search(terms), true),
    }
}

pub fn combine_editable_rules(rules: &[EditableRule]) -> RuleCombineReport {
    let before_count = rules.len();
    let mut combined_groups = 0;
    let mut combined_rules = Vec::with_capacity(rules.len());
    let mut index = 0;

    while index < rules.len() {
        let key = RuleCombineKey::from(&rules[index]);
        let mut run_end = index + 1;
        while run_end < rules.len() && RuleCombineKey::from(&rules[run_end]) == key {
            run_end += 1;
        }

        let run = &rules[index..run_end];
        let mut terms = Vec::new();
        let mut seen_terms = HashSet::new();
        let mut first_mergeable = None;
        let mut mergeable_count = 0;
        let mut unmergeable = Vec::new();

        for rule in run {
            if let Some(rule_terms) = mergeable_rule_terms(rule) {
                mergeable_count += 1;
                first_mergeable.get_or_insert_with(|| rule.clone());
                for term in rule_terms {
                    if seen_terms.insert(term.dedupe_key.clone()) {
                        terms.push(term);
                    }
                }
            } else {
                unmergeable.push(rule.clone());
            }
        }

        if mergeable_count > 1 {
            if let Some(mut rule) = first_mergeable {
                if terms.len() > 1 {
                    rule.search = combined_rule_search(&terms);
                    rule.is_regex = true;
                }
                combined_rules.push(rule);
                combined_rules.extend(unmergeable);
                combined_groups += 1;
            }
        } else {
            combined_rules.extend(run.iter().cloned());
        }

        index = run_end;
    }

    let after_count = combined_rules.len();
    RuleCombineReport {
        rules: combined_rules,
        before_count,
        after_count,
        combined_groups,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(search: &str, is_regex: bool) -> EditableRule {
        EditableRule {
            search: search.to_string(),
            is_regex,
            ..EditableRule::new_default()
        }
    }

    #[test]
    fn editable_rule_literal_terms_reads_plain_text() {
        assert_eq!(
            editable_rule_literal_terms(&rule("  grocery   store  ", false)),
            Some(vec!["grocery store".to_string()])
        );
    }

    #[test]
    fn editable_rule_literal_terms_reads_combined_literal_regex() {
        assert_eq!(
            editable_rule_literal_terms(&rule(r"(?:alpha|beta\s+shop)", true)),
            Some(vec!["alpha".to_string(), "beta shop".to_string()])
        );
    }

    #[test]
    fn rule_search_from_literal_terms_keeps_single_term_plain() {
        assert_eq!(
            rule_search_from_literal_terms(&["  grocery  store  ".to_string()]),
            ("grocery store".to_string(), false)
        );
    }

    #[test]
    fn rule_search_from_literal_terms_builds_regex_for_multiple_terms() {
        assert_eq!(
            rule_search_from_literal_terms(&[
                "grocery store".to_string(),
                "GROCERY STORE".to_string(),
                "pet shop".to_string(),
            ]),
            (r"(?:grocery\s+store|pet\s+shop)".to_string(), true)
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuleCombineKey {
    priority: i32,
    active: bool,
    field: String,
    category: String,
    budget_code: String,
    direction: String,
    amount_min: String,
    amount_max: String,
    notes: String,
}

impl From<&EditableRule> for RuleCombineKey {
    fn from(rule: &EditableRule) -> Self {
        Self {
            priority: rule.priority,
            active: rule.active,
            field: trimmed(&rule.field),
            category: trimmed(&rule.category),
            budget_code: trimmed(&rule.budget_code),
            direction: trimmed(&rule.direction),
            amount_min: trimmed(&rule.amount_min),
            amount_max: trimmed(&rule.amount_max),
            notes: trimmed(&rule.notes),
        }
    }
}

pub(super) fn trimmed(input: &str) -> String {
    input.trim().to_string()
}
