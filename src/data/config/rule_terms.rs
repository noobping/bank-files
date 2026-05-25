use super::super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MergeableRuleTerm {
    pub(super) pattern: String,
    pub(super) dedupe_key: String,
    pub(super) literal: Option<String>,
}

pub(super) fn mergeable_rule_terms(rule: &EditableRule) -> Option<Vec<MergeableRuleTerm>> {
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

pub(super) fn literal_rule_term(literal: &str) -> MergeableRuleTerm {
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
    let literal = literal_from_regex_term(pattern).map(|literal| normalize_literal_text(&literal));
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

fn literal_from_regex_term(pattern: &str) -> Option<String> {
    let mut literal = String::new();
    let mut chars = pattern.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if chars.peek() == Some(&'s') {
                chars.next();
                if chars.next() != Some('+') {
                    return None;
                }
                literal.push(' ');
                continue;
            }
            let escaped = chars.next()?;
            if validation::is_regex_meta(escaped) || escaped == '\\' {
                literal.push(escaped);
            } else {
                return None;
            }
        } else if validation::is_regex_meta(ch) {
            return None;
        } else {
            literal.push(ch);
        }
    }

    Some(literal)
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

pub(super) fn normalize_literal_text(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn literal_regex_pattern(literal: &str) -> String {
    literal_regex_fragment(literal.trim())
}

pub(super) fn literal_regex_fragment(literal: &str) -> String {
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
