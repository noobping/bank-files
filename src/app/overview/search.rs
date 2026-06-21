use super::*;

pub(super) fn render_overview_search(
    data: &AppData,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
    filter: &SearchFilter,
) {
    let mut has_results = false;

    let years = data.available_years.clone();
    if let Some(year) = years.last().copied() {
        let budgets = analytics::annual_budget_usage(
            &data.transactions,
            &data.budgets,
            year,
            comparison_mode(ui_handles.as_ref()),
        )
        .into_iter()
        .filter(|budget| annual_budget_matches(budget, filter))
        .take(6)
        .collect::<Vec<_>>();
        let mut annual_sections = Vec::new();
        if !budgets.is_empty() {
            has_results = true;
            let section = ui::card_list_section_group(
                "Annual Budgets",
                &trf("Filtered on {year}.", &[("year", year.to_string())]),
            );
            let box_ = ui::card_grid(Vec::new(), 2);
            for budget in &budgets {
                append_annual_budget_row(&box_, budget, year, ui_handles, state);
            }
            section.append(&box_);
            annual_sections.push(section);
        }

        let categories = analytics::category_totals_for_year(
            &data.transactions,
            &data.budgets,
            year,
            usize::MAX,
        )
        .into_iter()
        .filter(|category| annual_category_matches(category, filter))
        .collect::<Vec<_>>();
        if !categories.is_empty() {
            has_results = true;
            annual_sections.push(annual_spending_section_from_rows(
                categories,
                year,
                &trf("Filtered on {year}.", &[("year", year.to_string())]),
                SEARCH_CATEGORY_PREVIEW_LIMIT,
                ui_handles,
                state,
            ));
        }

        sections::append_overview_sections(&ui_handles.overview, annual_sections);
    }

    if !has_results {
        ui_handles.overview.append(&search_empty_page(
            "No results",
            "Adjust your search term or clear the search bar to see the full overview.",
        ));
    }
}
