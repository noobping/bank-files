use crate::util::normalize_key;

const RULE_FIELD_ALIASES: &str = include_str!("../../data/defaults/rule_field_aliases.tsv");
const DIRECTION_ALIASES: &str = include_str!("../../data/defaults/direction_aliases.tsv");
const FALLBACK_CATEGORIES_EN: &str = include_str!("../../data/defaults/fallback_categories.tsv");
const FALLBACK_CATEGORIES_NL: &str = include_str!("../../data/defaults/fallback_categories.nl.tsv");
const FALLBACK_CATEGORIES_DE: &str = include_str!("../../data/defaults/fallback_categories.de.tsv");
const FALSE_ALIASES: &str = include_str!("../../data/defaults/false_aliases.txt");
const DEFAULT_RULES_EN: &str = include_str!("../../data/defaults/editable_rules.csv");
const DEFAULT_RULES_NL: &str = include_str!("../../data/defaults/editable_rules.nl.csv");
const DEFAULT_RULES_DE: &str = include_str!("../../data/defaults/editable_rules.de.csv");
const DEFAULT_BUDGETS_EN: &str = include_str!("../../data/defaults/budgetcodes.csv");
const DEFAULT_BUDGETS_NL: &str = include_str!("../../data/defaults/budgetcodes.nl.csv");
const DEFAULT_BUDGETS_DE: &str = include_str!("../../data/defaults/budgetcodes.de.csv");

pub(super) const AUTO_DETECTED_CATEGORY_NOTE: &str =
    "Auto detected from built-in category keywords.";
pub(super) const GENERATED_AUTOMATIC_NOTE: &str = "Generated from configuration generation.";
pub(super) const GENERATED_PATTERN_NOTE: &str = "Generated from detected transaction pattern.";
pub(super) const LOCAL_AI_NOTE: &str = "Generated with local AI smart insights.";

pub(super) fn canonical_rule_field(field: &str) -> Option<&'static str> {
    canonical_from_alias_table(field, RULE_FIELD_ALIASES)
}

pub(super) fn canonical_direction(direction: &str) -> Option<&'static str> {
    if normalize_key(direction).is_empty() {
        return Some("any");
    }
    canonical_from_alias_table(direction, DIRECTION_ALIASES)
}

pub(super) fn parse_bool(input: &str) -> bool {
    let input = normalize_key(input);
    !FALSE_ALIASES
        .lines()
        .any(|alias| normalize_key(alias) == input)
}

pub(super) fn localized_default_rules() -> &'static str {
    localized_default(DEFAULT_RULES_EN, DEFAULT_RULES_NL, DEFAULT_RULES_DE)
}

pub(super) fn localized_default_budgets() -> &'static str {
    localized_default(DEFAULT_BUDGETS_EN, DEFAULT_BUDGETS_NL, DEFAULT_BUDGETS_DE)
}

pub(super) fn fallback_categories() -> &'static str {
    match crate::i18n::active_language() {
        crate::i18n::Language::English => FALLBACK_CATEGORIES_EN,
        crate::i18n::Language::Dutch => FALLBACK_CATEGORIES_NL,
        crate::i18n::Language::German => FALLBACK_CATEGORIES_DE,
    }
}

fn canonical_from_alias_table(input: &str, table: &'static str) -> Option<&'static str> {
    let input = normalize_key(input);
    table.lines().skip(1).find_map(|line| {
        let mut cols = line.splitn(2, '\t');
        let canonical = cols.next()?.trim();
        let aliases = cols.next()?.trim();
        aliases
            .split('|')
            .any(|alias| normalize_key(alias) == input)
            .then_some(canonical)
    })
}

fn localized_default(
    english: &'static str,
    dutch: &'static str,
    german: &'static str,
) -> &'static str {
    match crate::i18n::active_language() {
        crate::i18n::Language::English => english,
        crate::i18n::Language::Dutch => dutch,
        crate::i18n::Language::German => german,
    }
}
