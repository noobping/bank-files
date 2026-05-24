use super::LocalAiRecord;
use crate::data::generated_budget_code_for_category;
use crate::util::normalize_key;

const DEFAULT_CATEGORIES_EN: &str = include_str!("../../data/defaults/fallback_categories.tsv");
const DEFAULT_CATEGORIES_NL: &str = include_str!("../../data/defaults/fallback_categories.nl.tsv");
const DEFAULT_CATEGORIES_DE: &str = include_str!("../../data/defaults/fallback_categories.de.tsv");

#[derive(Debug, Clone)]
pub(super) struct CategoryGuess {
    pub(super) category: String,
    pub(super) budget_code: String,
}

pub(super) fn category_guess(record: &LocalAiRecord) -> CategoryGuess {
    if let Some(guess) = existing_category_guess(record) {
        return guess;
    }

    let text = record_match_text(record);
    category_rows()
        .lines()
        .skip(1)
        .find_map(|line| category_guess_from_row(line, &text, record))
        .unwrap_or_else(|| label_category_guess(&record.label))
}

pub(super) fn clean_label(input: &str) -> String {
    input
        .split_whitespace()
        .take(6)
        .collect::<Vec<_>>()
        .join(" ")
}

fn existing_category_guess(record: &LocalAiRecord) -> Option<CategoryGuess> {
    let category = clean_label(&record.existing_category);
    if category.is_empty() {
        return None;
    }

    let budget_code = clean_label(&record.existing_budget_code);
    let budget_code = if budget_code.is_empty() {
        generated_budget_code_for_category(&category, &[])
    } else {
        budget_code
    };
    Some(CategoryGuess {
        category,
        budget_code,
    })
}

fn category_guess_from_row(row: &str, text: &str, record: &LocalAiRecord) -> Option<CategoryGuess> {
    let mut columns = row.splitn(4, '\t');
    let category = columns.next()?.trim();
    let budget_code = columns.next()?.trim();
    let direction = columns.next()?.trim();
    let keywords = columns.next()?.trim();
    if !direction_matches(direction, &record.direction) || !keywords_match(text, keywords) {
        return None;
    }
    Some(CategoryGuess {
        category: category.to_string(),
        budget_code: budget_code.to_string(),
    })
}

fn label_category_guess(label: &str) -> CategoryGuess {
    let category = clean_label(label);
    let category = if category.is_empty() {
        "Generated budget".to_string()
    } else {
        category
    };
    let budget_code = generated_budget_code_for_category(&category, &[]);
    CategoryGuess {
        category,
        budget_code,
    }
}

fn record_match_text(record: &LocalAiRecord) -> String {
    normalize_key(&format!(
        "{} {} {}",
        record.label,
        record.descriptions.join(" "),
        record.tags.join(" ")
    ))
}

fn keywords_match(text: &str, keywords: &str) -> bool {
    keywords.split('|').any(|keyword| {
        let keyword = normalize_key(keyword);
        !keyword.is_empty() && text.contains(&keyword)
    })
}

fn direction_matches(candidate: &str, record_direction: &str) -> bool {
    let candidate = normalize_key(candidate);
    let record_direction = normalize_key(record_direction);
    matches!(candidate.as_str(), "" | "any") || candidate == record_direction
}

fn category_rows() -> &'static str {
    match crate::i18n::active_language() {
        crate::i18n::Language::English => DEFAULT_CATEGORIES_EN,
        crate::i18n::Language::Dutch => DEFAULT_CATEGORIES_NL,
        crate::i18n::Language::German => DEFAULT_CATEGORIES_DE,
    }
}
