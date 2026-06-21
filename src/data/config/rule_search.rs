use super::rule_terms::{literal_regex_fragment, literal_regex_pattern, MergeableRuleTerm};

pub(super) fn combined_rule_search(terms: &[MergeableRuleTerm]) -> String {
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
