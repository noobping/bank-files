use super::*;

#[derive(Debug, Clone)]
pub(super) struct TransactionPatternsRenderData {
    pub(super) hidden_count: usize,
    pub(super) hide_canceled: bool,
    pub(super) patterns: Vec<(analytics::TransactionPattern, TransactionPatternInfo)>,
    pub(super) preview_patterns: Vec<(analytics::TransactionPattern, TransactionPatternInfo)>,
    pub(super) more_patterns: Vec<(analytics::TransactionPattern, TransactionPatternInfo)>,
    pub(super) hidden_patterns: Vec<(analytics::TransactionPattern, TransactionPatternInfo)>,
}

pub(super) fn transaction_patterns_render_data(
    mut data: AppData,
    search: Option<SearchFilter>,
    show_all_patterns: bool,
    hide_canceled: bool,
    selected_year: Option<i32>,
    smart_insights_enabled: bool,
) -> TransactionPatternsRenderData {
    if let Some(year) = selected_year {
        data.transactions
            .retain(|transaction| transaction.year() == year);
    }

    let pattern_rules = crate::data::load_editable_rules().unwrap_or_default();
    let hidden_pattern_keys = crate::data::ignored_transaction_pattern_keys().unwrap_or_default();
    let ai_hints = crate::local_ai::transaction_pattern_hints(&data, smart_insights_enabled).hints;
    let pattern_analysis =
        analytics::transaction_pattern_analysis(&data.transactions, data.dedupe_mode.is_enabled());
    let hidden_count = pattern_analysis.hidden_canceled_transaction_count();
    let section_search_matches = search
        .as_ref()
        .map(|filter| transaction_patterns_section_matches(Some(filter)))
        .unwrap_or(false);
    let mut patterns = pattern_analysis
        .patterns
        .into_iter()
        .filter(|pattern| {
            section_search_matches
                || search
                    .as_ref()
                    .map(|filter| transaction_pattern_matches(pattern, filter))
                    .unwrap_or(true)
        })
        .map(|pattern| {
            let hidden =
                hidden_pattern_keys.contains(&analytics::transaction_pattern_key(&pattern));
            let info = transaction_pattern_info(&pattern, &data, &pattern_rules, hidden, &ai_hints);
            (pattern, info)
        })
        .collect::<Vec<_>>();
    patterns.sort_by(transaction_pattern_render_order);
    let non_hidden_patterns = patterns
        .iter()
        .filter(|(_, info)| !info.hidden)
        .cloned()
        .collect::<Vec<_>>();
    let hidden_patterns = patterns
        .iter()
        .filter(|(_, info)| info.hidden)
        .cloned()
        .collect::<Vec<_>>();
    let preview_source = if show_all_patterns {
        non_hidden_patterns.clone()
    } else {
        non_hidden_patterns
            .iter()
            .filter(|(_, info)| info.needs_rule)
            .cloned()
            .collect::<Vec<_>>()
    };
    let preview_limit = if show_all_patterns {
        usize::MAX
    } else {
        CATEGORY_PREVIEW_LIMIT
    };
    let preview_patterns = preview_source
        .iter()
        .take(preview_limit)
        .cloned()
        .collect::<Vec<_>>();
    let more_patterns = if show_all_patterns {
        Vec::new()
    } else {
        non_hidden_patterns.clone()
    };

    TransactionPatternsRenderData {
        hidden_count,
        hide_canceled,
        patterns,
        preview_patterns,
        more_patterns,
        hidden_patterns,
    }
}

pub(super) fn transaction_pattern_render_order(
    left: &(analytics::TransactionPattern, TransactionPatternInfo),
    right: &(analytics::TransactionPattern, TransactionPatternInfo),
) -> std::cmp::Ordering {
    right
        .1
        .needs_rule
        .cmp(&left.1.needs_rule)
        .then(right.1.affects_totals.cmp(&left.1.affects_totals))
        .then(right.0.last_date.cmp(&left.0.last_date))
        .then(right.0.count.cmp(&left.0.count))
        .then(left.0.label.cmp(&right.0.label))
}

#[derive(Debug, Clone)]
pub(super) struct TransactionPatternInfo {
    pub(super) badges: Vec<String>,
    pub(super) hidden: bool,
    pub(super) needs_rule: bool,
    pub(super) affects_totals: bool,
}

fn transaction_pattern_info(
    pattern: &analytics::TransactionPattern,
    data: &AppData,
    rules: &[EditableRule],
    hidden: bool,
    ai_hints: &[crate::local_ai::LocalAiPatternHint],
) -> TransactionPatternInfo {
    let cancels_out = matches!(
        pattern.kind,
        analytics::TransactionPatternKind::FullRefund
            | analytics::TransactionPatternKind::BillSplit
    );
    let is_transfer = matches!(pattern.kind, analytics::TransactionPatternKind::Transfer);
    let generated_rule = transaction_pattern_has_rule(pattern, rules);
    let covered_by_rule = transaction_pattern_is_covered(pattern, data);

    let mut badges = Vec::new();
    if cancels_out {
        badges.push(tr("Cancels out transactions"));
    }
    if is_transfer {
        badges.push(tr("Possible transfer"));
    }
    if generated_rule {
        badges.push(tr("Generated rule"));
    }
    if hidden {
        badges.push(tr("Hidden"));
    }
    if let Some(hint) = crate::local_ai::pattern_hint_for_label(ai_hints, &pattern.label) {
        badges.push(trf(
            "Local AI: {category}",
            &[("category", hint.category.clone())],
        ));
    }
    if covered_by_rule {
        badges.push(tr("Covered by rule"));
    } else {
        badges.push(tr("Needs rule"));
    }

    TransactionPatternInfo {
        badges,
        hidden,
        needs_rule: !covered_by_rule,
        affects_totals: is_transfer || cancels_out,
    }
}

fn transaction_pattern_is_covered(pattern: &analytics::TransactionPattern, data: &AppData) -> bool {
    let matches = data
        .transactions
        .iter()
        .filter(|transaction| analytics::transaction_matches_pattern(transaction, pattern))
        .collect::<Vec<_>>();
    !matches.is_empty()
        && matches.iter().all(|transaction| {
            let category = transaction.category.trim();
            let code = transaction.budget_code.trim();
            !category.is_empty()
                && category != "Uncategorized"
                && !matches!(code, "" | "OTHER" | "INC-OTHER")
        })
}

fn transaction_pattern_has_rule(
    pattern: &analytics::TransactionPattern,
    rules: &[EditableRule],
) -> bool {
    rules.iter().filter(|rule| rule.active).any(|rule| {
        let search = rule.search.trim();
        !search.is_empty()
            && (search.eq_ignore_ascii_case(pattern.label.trim())
                || pattern
                    .match_labels
                    .iter()
                    .any(|label| search.eq_ignore_ascii_case(label)))
    })
}
