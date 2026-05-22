use super::*;

const CONFIG_FILE_NAMES: [&str; 4] = [
    "rules.csv",
    "budgetcodes.csv",
    "field_aliases.csv",
    "ignored_transaction_patterns.csv",
];
const CONFIG_ARCHIVE_DIR: &str = "archive";
const EMPTY_IGNORED_TRANSACTION_PATTERNS: &str = "key,label\n";

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

fn configured_budget_codes(budgets: &[EditableBudget]) -> HashSet<String> {
    budgets
        .iter()
        .map(|budget| normalize_key(&budget.code))
        .filter(|code| !code.is_empty())
        .collect()
}

fn orphaned_rule(rule: EditableRule, budget_codes: &HashSet<String>) -> Option<OrphanedRule> {
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

fn load_ignored_transaction_patterns() -> Result<Vec<IgnoredTransactionPattern>> {
    let (_, contents) = read_config_file("ignored_transaction_patterns.csv")?;
    parse_ignored_transaction_patterns(&contents)
}

pub fn ignored_transaction_pattern_keys() -> Result<HashSet<String>> {
    Ok(load_ignored_transaction_patterns()?
        .into_iter()
        .map(|pattern| pattern.key)
        .collect())
}

pub fn ignore_transaction_pattern(key: &str, label: &str) -> Result<bool> {
    let key = key.trim();
    if key.is_empty() {
        anyhow::bail!("Ignored transaction pattern needs a key");
    }

    let mut patterns = load_ignored_transaction_patterns()?;
    if patterns.iter().any(|pattern| pattern.key == key) {
        return Ok(false);
    }

    patterns.push(IgnoredTransactionPattern {
        key: key.to_string(),
        label: label.trim().to_string(),
    });
    patterns.sort_by(|left, right| left.label.cmp(&right.label).then(left.key.cmp(&right.key)));
    write_ignored_transaction_patterns(&patterns)?;
    Ok(true)
}

pub fn reopen_transaction_pattern(key: &str) -> Result<bool> {
    let mut patterns = load_ignored_transaction_patterns()?;
    let original_len = patterns.len();
    patterns.retain(|pattern| pattern.key != key.trim());
    let changed = patterns.len() != original_len;
    if changed {
        write_ignored_transaction_patterns(&patterns)?;
    }
    Ok(changed)
}

fn write_ignored_transaction_patterns(patterns: &[IgnoredTransactionPattern]) -> Result<PathBuf> {
    let contents = serialize_ignored_transaction_patterns(patterns)?;
    write_config_file("ignored_transaction_patterns.csv", &contents)
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

fn trimmed(input: &str) -> String {
    input.trim().to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MergeableRuleTerm {
    pattern: String,
    dedupe_key: String,
    literal: Option<String>,
}

fn mergeable_rule_terms(rule: &EditableRule) -> Option<Vec<MergeableRuleTerm>> {
    let search = rule.search.trim();
    if search.is_empty() {
        return None;
    }
    if !rule.is_regex {
        return Some(vec![literal_rule_term(search)]);
    }

    regex::RegexBuilder::new(search)
        .case_insensitive(true)
        .build()
        .ok()?;
    let pattern = mergeable_regex_body(search);
    split_top_level_alternatives(pattern)?
        .into_iter()
        .map(|term| regex_rule_term(&term))
        .collect()
}

fn literal_rule_term(literal: &str) -> MergeableRuleTerm {
    let literal = normalize_literal_text(literal);
    MergeableRuleTerm {
        pattern: literal_regex_pattern(&literal),
        dedupe_key: format!("literal:{}", literal.to_lowercase()),
        literal: Some(literal),
    }
}

fn regex_rule_term(pattern: &str) -> Option<MergeableRuleTerm> {
    let pattern = mergeable_regex_body(pattern).trim();
    if pattern.is_empty() || contains_unsupported_inline_construct(pattern) {
        return None;
    }
    let literal =
        validation::unescape_regex_literal(pattern).map(|literal| normalize_literal_text(&literal));
    let dedupe_key = literal
        .as_ref()
        .map(|literal| format!("literal:{}", literal.to_lowercase()))
        .unwrap_or_else(|| format!("regex:{pattern}"));

    Some(MergeableRuleTerm {
        pattern: pattern.to_string(),
        dedupe_key,
        literal,
    })
}

fn mergeable_regex_body(mut pattern: &str) -> &str {
    loop {
        pattern = pattern.trim();
        if let Some(stripped) = pattern.strip_prefix("(?i)") {
            pattern = stripped;
        } else if let Some(inner) = strip_outer_case_insensitive_group(pattern) {
            pattern = inner;
        } else if let Some(inner) = strip_outer_non_capturing_group(pattern) {
            pattern = inner;
        } else {
            return pattern;
        }
    }
}

fn strip_outer_case_insensitive_group(pattern: &str) -> Option<&str> {
    if !pattern.starts_with("(?i:") || !pattern.ends_with(')') {
        return None;
    }
    outer_group_closes_at_end(pattern).then(|| &pattern[4..pattern.len() - 1])
}

fn strip_outer_non_capturing_group(pattern: &str) -> Option<&str> {
    if !pattern.starts_with("(?:") || !pattern.ends_with(')') {
        return None;
    }
    outer_group_closes_at_end(pattern).then(|| &pattern[3..pattern.len() - 1])
}

fn outer_group_closes_at_end(pattern: &str) -> bool {
    let mut escaped = false;
    let mut in_class = false;
    let mut depth = 0usize;
    for (index, ch) in pattern.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '[' if !in_class => in_class = true,
            ']' if in_class => in_class = false,
            '(' if !in_class => depth += 1,
            ')' if !in_class => {
                depth = depth.saturating_sub(1);
                if depth == 0 && index != pattern.len() - 1 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0 && !in_class && !escaped
}

fn split_top_level_alternatives(pattern: &str) -> Option<Vec<String>> {
    let mut terms = Vec::new();
    let mut escaped = false;
    let mut in_class = false;
    let mut depth = 0usize;
    let mut start = 0usize;

    for (index, ch) in pattern.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '[' if !in_class => in_class = true,
            ']' if in_class => in_class = false,
            '(' if !in_class => depth += 1,
            ')' if !in_class => depth = depth.saturating_sub(1),
            '|' if !in_class && depth == 0 => {
                push_regex_term(&mut terms, &pattern[start..index])?;
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    push_regex_term(&mut terms, &pattern[start..])?;
    Some(terms)
}

fn push_regex_term(terms: &mut Vec<String>, term: &str) -> Option<()> {
    let term = term.trim();
    if term.is_empty() {
        return None;
    }
    terms.push(term.to_string());
    Some(())
}

fn contains_unsupported_inline_construct(pattern: &str) -> bool {
    let mut escaped = false;
    let mut in_class = false;
    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => {
                escaped = true;
                continue;
            }
            '[' if !in_class => {
                in_class = true;
                continue;
            }
            ']' if in_class => {
                in_class = false;
                continue;
            }
            _ => {}
        }
        if in_class || ch != '(' || chars.peek() != Some(&'?') {
            continue;
        }
        let mut lookahead = chars.clone();
        lookahead.next();
        match lookahead.next() {
            Some(':') => {}
            Some('i') if lookahead.next() == Some(':') => {}
            _ => return true,
        }
    }
    false
}

fn normalize_literal_text(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn literal_regex_pattern(literal: &str) -> String {
    literal_regex_fragment(literal.trim())
}

fn literal_regex_fragment(literal: &str) -> String {
    let mut pattern = String::new();
    let mut pending_whitespace = false;

    for ch in literal.chars() {
        if ch.is_whitespace() {
            pending_whitespace = true;
            continue;
        }
        if pending_whitespace {
            pattern.push_str(r"\s+");
            pending_whitespace = false;
        }
        pattern.push_str(&regex::escape(&ch.to_string()));
    }

    pattern
}

fn combined_rule_search(terms: &[MergeableRuleTerm]) -> String {
    if let Some(pattern) = combined_literal_search(terms) {
        return pattern;
    }
    format!(
        "(?:{})",
        terms
            .iter()
            .map(|term| term.pattern.as_str())
            .collect::<Vec<_>>()
            .join("|")
    )
}

fn combined_literal_search(terms: &[MergeableRuleTerm]) -> Option<String> {
    let literals = terms
        .iter()
        .map(|term| term.literal.as_ref())
        .collect::<Option<Vec<_>>>()?;
    if literals.len() < 2 {
        return None;
    }

    let prefix_len = common_literal_prefix_len(&literals);
    if prefix_len == 0 {
        return Some(format!(
            "(?:{})",
            literals
                .iter()
                .map(|literal| literal_regex_pattern(literal))
                .collect::<Vec<_>>()
                .join("|")
        ));
    }

    let prefix = &literals[0][..prefix_len];
    let alternatives = literals
        .iter()
        .map(|literal| literal_regex_fragment(&literal[prefix_len..]))
        .collect::<Vec<_>>()
        .join("|");
    Some(format!(
        "{}(?:{})",
        literal_regex_pattern(prefix),
        alternatives
    ))
}

fn common_literal_prefix_len(literals: &[&String]) -> usize {
    if literals.len() < 2 {
        return 0;
    }

    let mut prefix_len = literals[0].len();
    for literal in &literals[1..] {
        while prefix_len > 0 && !literal.starts_with(&literals[0][..prefix_len]) {
            prefix_len = previous_char_boundary(literals[0], prefix_len);
        }
    }

    let mut best = 0;
    let mut boundaries = literals[0]
        .char_indices()
        .map(|(index, ch)| index + ch.len_utf8())
        .filter(|boundary| *boundary <= prefix_len)
        .collect::<Vec<_>>();
    boundaries.push(0);
    boundaries.sort_unstable();
    boundaries.dedup();

    for boundary in boundaries {
        if boundary == 0 || literal_alnum_count(&literals[0][..boundary]) < 3 {
            continue;
        }
        if literals
            .iter()
            .all(|literal| literal_prefix_boundary_is_safe(literal, boundary))
        {
            best = boundary;
        }
    }

    trim_trailing_whitespace_boundary(literals[0], best)
}

fn previous_char_boundary(input: &str, index: usize) -> usize {
    input[..index]
        .char_indices()
        .next_back()
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn literal_alnum_count(input: &str) -> usize {
    input.chars().filter(|ch| ch.is_alphanumeric()).count()
}

fn literal_prefix_boundary_is_safe(literal: &str, boundary: usize) -> bool {
    let previous = literal[..boundary].chars().next_back();
    let next = literal[boundary..].chars().next();
    match (previous, next) {
        (Some(previous), Some(next)) => !previous.is_alphanumeric() || !next.is_alphanumeric(),
        (Some(_), None) => true,
        _ => false,
    }
}

fn trim_trailing_whitespace_boundary(input: &str, mut boundary: usize) -> usize {
    while boundary > 0 {
        let Some(ch) = input[..boundary].chars().next_back() else {
            return 0;
        };
        if !ch.is_whitespace() {
            return boundary;
        }
        boundary -= ch.len_utf8();
    }
    0
}

fn parse_ignored_transaction_patterns(contents: &str) -> Result<Vec<IgnoredTransactionPattern>> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(contents.as_bytes());
    let headers = rdr
        .headers()
        .context("ignored_transaction_patterns.csv has no header")?
        .iter()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let mut patterns = Vec::new();
    let mut seen = HashSet::new();

    for row in rdr.records() {
        let row = row?;
        let key = csv_cell(&headers, &row, "key");
        if key.trim().is_empty() || !seen.insert(key.clone()) {
            continue;
        }
        patterns.push(IgnoredTransactionPattern {
            key,
            label: csv_cell(&headers, &row, "label"),
        });
    }

    Ok(patterns)
}

fn serialize_ignored_transaction_patterns(
    patterns: &[IgnoredTransactionPattern],
) -> Result<String> {
    let mut wtr = csv::WriterBuilder::new().from_writer(Vec::new());
    wtr.write_record(["key", "label"])?;

    for pattern in patterns
        .iter()
        .filter(|pattern| !pattern.key.trim().is_empty())
    {
        wtr.write_record([pattern.key.trim(), pattern.label.trim()])?;
    }

    writer_to_string(wtr)
}

pub fn load_editable_budgets() -> Result<Vec<EditableBudget>> {
    let (_, contents) = read_config_file("budgetcodes.csv")?;
    parse_editable_budgets(&contents)
}

pub fn write_editable_budgets(budgets: &[EditableBudget]) -> Result<PathBuf> {
    validate_editable_budgets(budgets)?;
    let contents = serialize_editable_budgets(budgets)?;
    write_config_file("budgetcodes.csv", &contents)
}

pub fn restore_default_configuration() -> Result<PathBuf> {
    let dirs = prepare_app_storage()?;
    restore_default_configuration_in(&dirs)
}

pub fn restore_empty_configuration() -> Result<PathBuf> {
    let dirs = prepare_app_storage()?;
    restore_empty_configuration_in(&dirs)
}

pub fn configuration_archive_exists() -> Result<bool> {
    let dirs = app_dirs()?;
    Ok(configuration_archive_exists_in(&dirs))
}

pub fn archive_configuration() -> Result<PathBuf> {
    let dirs = prepare_app_storage()?;
    archive_configuration_in(&dirs)
}

pub fn restore_configuration_archive() -> Result<PathBuf> {
    let dirs = prepare_app_storage()?;
    restore_configuration_archive_in(&dirs)
}

pub fn load_editable_aliases() -> Result<Vec<EditableAlias>> {
    let (_, contents) = read_config_file("field_aliases.csv")?;
    parse_editable_aliases(&contents)
}

pub fn write_editable_aliases(aliases: &[EditableAlias]) -> Result<PathBuf> {
    let contents = serialize_editable_aliases(aliases)?;
    write_config_file("field_aliases.csv", &contents)
}

pub fn upsert_editable_alias(canonical: &str, alias: &str) -> Result<bool> {
    let mut aliases = load_editable_aliases()?;
    if !upsert_alias(&mut aliases, canonical, alias)? {
        return Ok(false);
    }

    write_editable_aliases(&aliases)?;
    Ok(true)
}

pub(in crate::data) fn upsert_alias(
    aliases: &mut Vec<EditableAlias>,
    canonical: &str,
    alias: &str,
) -> Result<bool> {
    let canonical = canonical.trim();
    let alias = alias.trim();
    if canonical.is_empty() || alias.is_empty() {
        anyhow::bail!("Field alias needs both an app field and a CSV header");
    }

    let already_exists = aliases.iter().any(|existing| {
        existing.canonical.trim() == canonical && existing.alias.trim().eq_ignore_ascii_case(alias)
    });
    if already_exists {
        return Ok(false);
    }

    aliases.push(EditableAlias {
        canonical: canonical.to_string(),
        alias: alias.to_string(),
    });
    Ok(true)
}

pub fn read_config_file(name: &str) -> Result<(PathBuf, String)> {
    let dirs = app_dirs()?;
    let path = config_file_path(&dirs, name)?;
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            default_config_contents(name)?.to_string()
        }
        Err(error) => {
            return Err(error).with_context(|| format!("Could not read {}", path.display()))
        }
    };
    Ok((path, contents))
}

fn default_config_contents(name: &str) -> Result<&'static str> {
    match name {
        "rules.csv" => Ok(default_rules()),
        "budgetcodes.csv" => Ok(default_budgets()),
        "field_aliases.csv" => Ok(default_aliases()),
        "ignored_transaction_patterns.csv" => Ok(EMPTY_IGNORED_TRANSACTION_PATTERNS),
        _ => anyhow::bail!("Unknown configuration file: {name}"),
    }
}

pub fn write_config_file(name: &str, contents: &str) -> Result<PathBuf> {
    let dirs = prepare_app_storage()?;
    let path = config_file_path(&dirs, name)?;
    fs::write(&path, contents).with_context(|| format!("Could not save {}", path.display()))?;
    Ok(path)
}

pub(in crate::data) fn config_file_path(dirs: &AppDirs, name: &str) -> Result<PathBuf> {
    match name {
        "rules.csv"
        | "budgetcodes.csv"
        | "field_aliases.csv"
        | "ignored_transaction_patterns.csv" => Ok(dirs.config.join(name)),
        _ => anyhow::bail!("Unknown configuration file: {name}"),
    }
}

struct ConfigurationContents<'a> {
    rules: &'a str,
    budgets: &'a str,
    aliases: &'a str,
    ignored_patterns: &'a str,
}

pub(in crate::data) fn restore_default_configuration_in(dirs: &AppDirs) -> Result<PathBuf> {
    write_configuration_contents(
        dirs,
        ConfigurationContents {
            rules: default_rules(),
            budgets: default_budgets(),
            aliases: default_aliases(),
            ignored_patterns: EMPTY_IGNORED_TRANSACTION_PATTERNS,
        },
    )
}

pub(in crate::data) fn restore_empty_configuration_in(dirs: &AppDirs) -> Result<PathBuf> {
    let rules = serialize_editable_rules(&[])?;
    let budgets = serialize_editable_budgets(&[])?;
    write_configuration_contents(
        dirs,
        ConfigurationContents {
            rules: &rules,
            budgets: &budgets,
            aliases: default_aliases(),
            ignored_patterns: EMPTY_IGNORED_TRANSACTION_PATTERNS,
        },
    )
}

fn write_configuration_contents(
    dirs: &AppDirs,
    contents: ConfigurationContents<'_>,
) -> Result<PathBuf> {
    ensure_layout(dirs)?;

    fs::write(config_file_path(dirs, "rules.csv")?, contents.rules)
        .with_context(|| "Could not write rules configuration".to_string())?;
    fs::write(config_file_path(dirs, "budgetcodes.csv")?, contents.budgets)
        .with_context(|| "Could not write budget configuration".to_string())?;
    fs::write(
        config_file_path(dirs, "field_aliases.csv")?,
        contents.aliases,
    )
    .with_context(|| "Could not write field name configuration".to_string())?;
    fs::write(
        config_file_path(dirs, "ignored_transaction_patterns.csv")?,
        contents.ignored_patterns,
    )
    .with_context(|| "Could not write ignored transaction patterns".to_string())?;

    Ok(dirs.config.clone())
}

pub(in crate::data) fn archive_configuration_in(dirs: &AppDirs) -> Result<PathBuf> {
    ensure_layout(dirs)?;
    ensure_default_files(dirs)?;

    let archive = configuration_archive_dir(dirs);
    remove_existing_archive(&archive)?;
    fs::create_dir_all(&archive)
        .with_context(|| format!("Could not create {}", archive.display()))?;

    for name in CONFIG_FILE_NAMES {
        let source = config_file_path(dirs, name)?;
        let target = archive.join(name);
        fs::copy(&source, &target).with_context(|| {
            format!(
                "Could not back up {} to {}",
                source.display(),
                target.display()
            )
        })?;
    }

    Ok(archive)
}

pub(in crate::data) fn restore_configuration_archive_in(dirs: &AppDirs) -> Result<PathBuf> {
    ensure_layout(dirs)?;

    let archive = configuration_archive_dir(dirs);
    if !configuration_archive_exists_in(dirs) {
        anyhow::bail!("No configuration backup exists in {}", archive.display());
    }

    for name in CONFIG_FILE_NAMES {
        let source = archive.join(name);
        let target = config_file_path(dirs, name)?;
        fs::copy(&source, &target).with_context(|| {
            format!(
                "Could not restore {} to {}",
                source.display(),
                target.display()
            )
        })?;
    }

    Ok(archive)
}

fn configuration_archive_exists_in(dirs: &AppDirs) -> bool {
    let archive = configuration_archive_dir(dirs);
    archive.is_dir()
        && CONFIG_FILE_NAMES
            .iter()
            .all(|name| archive.join(name).is_file())
}

fn configuration_archive_dir(dirs: &AppDirs) -> PathBuf {
    dirs.config.join(CONFIG_ARCHIVE_DIR)
}

fn remove_existing_archive(archive: &Path) -> Result<()> {
    if archive.is_dir() {
        fs::remove_dir_all(archive)
            .with_context(|| format!("Could not replace {}", archive.display()))?;
    } else if archive.exists() {
        fs::remove_file(archive)
            .with_context(|| format!("Could not replace {}", archive.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignored_transaction_patterns_roundtrip_and_dedupe_keys() {
        let contents = "key,label\nrefund:coffee,Coffee refund\nrefund:coffee,Duplicate\ntransfer:rent,Rent transfer\n";

        let patterns = parse_ignored_transaction_patterns(contents).unwrap();

        assert_eq!(patterns.len(), 2);
        assert_eq!(patterns[0].key, "refund:coffee");
        assert_eq!(patterns[0].label, "Coffee refund");
        assert_eq!(patterns[1].key, "transfer:rent");

        let serialized = serialize_ignored_transaction_patterns(&patterns).unwrap();
        assert!(serialized.contains("refund:coffee"));
        assert!(serialized.contains("Coffee refund"));
        assert!(!serialized.contains("Duplicate"));
    }
}
