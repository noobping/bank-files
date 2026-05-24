use super::*;

pub(in crate::app) fn comparison_mode(ui: &UiHandles) -> ComparisonMode {
    if ui.compare_categories_previous_period.get() {
        ComparisonMode::WithPrevious
    } else {
        ComparisonMode::CurrentOnly
    }
}

pub(in crate::app) fn current_transaction_load_scope(
    data: &AppData,
    ui: &UiHandles,
) -> TransactionLoadScope {
    match ui.stack.visible_child_name().as_deref() {
        Some("debug") => default_year_load_scope(data, ui),
        Some("transactions") => transaction_page_load_scope(data, ui),
        Some("categories") => TransactionLoadScope::for_month(
            ui.selected_budget_month.get().or(data.default_month),
            comparison_mode(ui),
        ),
        _ => default_year_load_scope(data, ui),
    }
}

fn default_year_load_scope(data: &AppData, ui: &UiHandles) -> TransactionLoadScope {
    TransactionLoadScope::for_year(
        selected_year_for_load_scope(data, ui.selected_year.get()),
        comparison_mode(ui),
    )
}

fn selected_year_for_load_scope(data: &AppData, selected_year: Option<i32>) -> Option<i32> {
    selected_year.or_else(|| data.default_month.map(|month| month.year))
}

fn transaction_page_load_scope(data: &AppData, ui: &UiHandles) -> TransactionLoadScope {
    let selected_year = ui
        .selected_year
        .get()
        .or_else(|| data.default_month.map(|month| month.year));
    let Some(filter) = ui.active_transaction_filter.borrow().clone() else {
        if ui.search_query.borrow().trim().is_empty() {
            return TransactionLoadScope::Year(selected_year);
        }
        return TransactionLoadScope::All;
    };
    match filter {
        TransactionFilter::CategoryForYear { year, .. } => TransactionLoadScope::Year(Some(year)),
        TransactionFilter::Scoped {
            year: Some(year),
            month: None,
            ..
        } => TransactionLoadScope::Year(Some(year)),
        TransactionFilter::Scoped {
            month: Some(month), ..
        } => TransactionLoadScope::Month(Some(month)),
        TransactionFilter::Scoped {
            year: None,
            month: None,
            ..
        } => TransactionLoadScope::Year(selected_year),
        TransactionFilter::All
        | TransactionFilter::UnconfiguredBudgets
        | TransactionFilter::OtherCategories
        | TransactionFilter::Pattern(_) => TransactionLoadScope::All,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_year_for_load_scope_prefers_user_selection() {
        let data = AppData {
            default_month: Some(MonthKey::new(2024, 12)),
            ..AppData::default()
        };

        assert_eq!(selected_year_for_load_scope(&data, Some(2026)), Some(2026));
    }

    #[test]
    fn selected_year_for_load_scope_falls_back_to_default_month_year() {
        let data = AppData {
            default_month: Some(MonthKey::new(2024, 12)),
            ..AppData::default()
        };

        assert_eq!(selected_year_for_load_scope(&data, None), Some(2024));
    }
}
