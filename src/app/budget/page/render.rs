use super::super::*;
use super::charts::{append_budget_pie_charts, BudgetPieChartData};
use super::details::{append_budget_detail_sections, BudgetDetailSectionsData};

pub(in crate::app) fn render_budget_page(
    data: &AppData,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    ui::clear_box(&ui_handles.categories);
    let selected_month = selected_budget_month(data, ui_handles.as_ref());
    let search = active_search(ui_handles.as_ref());
    let page_text = selected_month
        .map(|month| summary::render_categories_for_month(data, month))
        .unwrap_or_else(|| summary::render_categories(data));
    let subtitle = search
        .as_ref()
        .map(|filter| {
            selected_month
                .map(|month| {
                    trf(
                        "Filter “{query}” searches budgets and spending for {month}.",
                        &[
                            ("query", filter.raw.clone()),
                            ("month", ui::month_label(month)),
                        ],
                    )
                })
                .unwrap_or_else(|| {
                    trf(
                        "Filter “{query}” searches budgets and spending.",
                        &[("query", filter.raw.clone())],
                    )
                })
        })
        .unwrap_or_else(|| {
            "Choose a month and year to see budget room and spending for that period.".to_string()
        });
    append_page_header(
        &ui_handles.categories,
        ui_handles.as_ref(),
        "Budget",
        &subtitle,
        page_text,
        &data.transactions,
    );

    if data.transactions.is_empty() {
        append_partial_load_notice(&ui_handles.categories, ui_handles, data);
        ui_handles.categories.append(&empty_page(
            "view-list-symbolic",
            "No budget view yet",
            "Import bank files first. Then budgets and spending appear here as graphical rows.",
        ));
        return;
    }

    let Some(selected_month) = selected_month else {
        append_partial_load_notice(&ui_handles.categories, ui_handles, data);
        ui_handles.categories.append(&empty_page(
            "view-list-symbolic",
            "No period found",
            "Import bank files to choose a month and year.",
        ));
        return;
    };

    let month_totals = totals_for_month(data, selected_month);
    let budget_rows = analytics::budget_usage(&data.transactions, &data.budgets, selected_month);
    let warnings = monthly_budget_attention_warnings(data, selected_month);
    append_attention_warning_card(&ui_handles.categories, &warnings);
    append_partial_load_notice(&ui_handles.categories, ui_handles, data);

    ui_handles
        .categories
        .append(&budget_period_row(data, selected_month, ui_handles, state));
    let all_categories = analytics::category_totals_for_month(
        &data.transactions,
        &data.budgets,
        selected_month,
        usize::MAX,
    );
    let month_label = ui::month_label(selected_month);
    let budgets = budget_rows
        .iter()
        .filter(|budget| {
            search
                .as_ref()
                .map(|filter| budget_usage_matches(budget, filter))
                .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();
    let categories = all_categories
        .iter()
        .filter(|category| {
            search
                .as_ref()
                .map(|filter| category_summary_matches(category, filter))
                .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut has_search_results = false;
    ui_handles.categories.append(&ui::metric_grid(
        financial_metric_cards_for_month(
            &month_totals,
            selected_month,
            &month_label,
            state,
            ui_handles,
        ),
        3,
    ));

    if append_budget_pie_charts(
        &ui_handles.categories,
        BudgetPieChartData {
            budget_rows: &budget_rows,
            configured_budgets: &data.budgets,
            real_month_income: month_totals.income,
            planned_month_income: analytics::planned_month_income_total(
                &data.budgets,
                month_totals.income,
            ),
            categories: &all_categories,
            month: selected_month,
            month_label: &month_label,
        },
        ui_handles,
        state,
    ) {
        has_search_results = true;
    }
    append_budget_detail_sections(
        &ui_handles.categories,
        BudgetDetailSectionsData {
            budgets,
            categories,
            month_label,
            month: selected_month,
            search_active: search.is_some(),
            show_all: ui_handles.show_all.get(),
            has_immediate_results: has_search_results,
        },
        ui_handles,
        state,
    );
}

pub(super) fn budget_usage_matches(budget: &analytics::BudgetUsage, filter: &SearchFilter) -> bool {
    filter.matches_summary(&format!(
        "{} {} {} {} {} {} {}",
        budget.code,
        budget.category,
        budget.notes,
        budget.budget_basis,
        money(budget.budget),
        money(budget.actual),
        money(budget.remaining),
    ))
}

pub(super) fn category_summary_matches(
    category: &analytics::CategorySummary,
    filter: &SearchFilter,
) -> bool {
    filter.matches_summary(&format!(
        "{} {} {} {} {} {}",
        category.category,
        category.budget_code,
        category.totals.count,
        money(category.totals.income),
        money(category.totals.expenses),
        signed_money(category.totals.balance),
    ))
}
fn financial_metric_cards_for_month(
    totals: &analytics::Totals,
    month: MonthKey,
    month_label: &str,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> Vec<gtk::Box> {
    financial_metric_order()
        .into_iter()
        .map(|metric| match metric {
            FinancialMetric::Income => {
                let state_for_income = Rc::clone(state);
                let ui_for_income = Rc::clone(ui_handles);
                ui::activatable_metric_card(
                    "Income",
                    &money(totals.income),
                    month_label,
                    move || {
                        show_transactions_filter(
                            &state_for_income,
                            &ui_for_income,
                            TransactionFilter::income_for_month(month),
                        );
                    },
                )
            }
            FinancialMetric::Expenses => {
                let state_for_expenses = Rc::clone(state);
                let ui_for_expenses = Rc::clone(ui_handles);
                ui::activatable_metric_card(
                    "Expenses",
                    &money(totals.expenses),
                    month_label,
                    move || {
                        show_transactions_filter(
                            &state_for_expenses,
                            &ui_for_expenses,
                            TransactionFilter::expenses_for_month(month),
                        );
                    },
                )
            }
            FinancialMetric::Balance => {
                let state_for_balance = Rc::clone(state);
                let ui_for_balance = Rc::clone(ui_handles);
                ui::activatable_metric_card(
                    "Balance",
                    &signed_money(totals.balance),
                    month_label,
                    move || {
                        show_transactions_filter(
                            &state_for_balance,
                            &ui_for_balance,
                            TransactionFilter::month(month),
                        );
                    },
                )
            }
        })
        .collect()
}
