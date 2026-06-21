const DEFAULT_RULES_EN: &str = include_str!("../../data/defaults/editable_rules.csv");
const DEFAULT_RULES_NL: &str = include_str!("../../data/defaults/editable_rules.nl.csv");
const DEFAULT_RULES_DE: &str = include_str!("../../data/defaults/editable_rules.de.csv");
const DEFAULT_BUDGETS_EN: &str = include_str!("../../data/defaults/budgetcodes.csv");
const DEFAULT_BUDGETS_NL: &str = include_str!("../../data/defaults/budgetcodes.nl.csv");
const DEFAULT_BUDGETS_DE: &str = include_str!("../../data/defaults/budgetcodes.de.csv");
const DEFAULT_ALIASES_EN: &str = include_str!("../../data/defaults/editable_field_aliases.csv");
const DEFAULT_ALIASES_NL: &str = include_str!("../../data/defaults/editable_field_aliases.nl.csv");
const DEFAULT_ALIASES_DE: &str = include_str!("../../data/defaults/editable_field_aliases.de.csv");
pub(in crate::data) const FALSE_ALIASES: &str =
    include_str!("../../data/defaults/false_aliases.txt");

pub(in crate::data) fn default_rules() -> &'static str {
    localized(DEFAULT_RULES_EN, DEFAULT_RULES_NL, DEFAULT_RULES_DE)
}

pub(in crate::data) fn default_budgets() -> &'static str {
    localized(DEFAULT_BUDGETS_EN, DEFAULT_BUDGETS_NL, DEFAULT_BUDGETS_DE)
}

pub(in crate::data) fn default_aliases() -> &'static str {
    localized(DEFAULT_ALIASES_EN, DEFAULT_ALIASES_NL, DEFAULT_ALIASES_DE)
}

fn localized(english: &'static str, dutch: &'static str, german: &'static str) -> &'static str {
    match crate::i18n::active_language() {
        crate::i18n::Language::English => english,
        crate::i18n::Language::Dutch => dutch,
        crate::i18n::Language::German => german,
    }
}
