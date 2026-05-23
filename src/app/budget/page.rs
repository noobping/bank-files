use super::*;

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
        ui_handles.categories.append(&empty_page(
            "view-list-symbolic",
            "No budget view yet",
            "Import CSV files first. Then budgets and spending appear here as graphical rows.",
        ));
        return;
    }

    let Some(selected_month) = selected_month else {
        ui_handles.categories.append(&empty_page(
            "view-list-symbolic",
            "No period found",
            "Import CSV files to choose a month and year.",
        ));
        return;
    };

    let month_totals = totals_for_month(data, selected_month);
    let budget_rows = analytics::budget_usage(&data.transactions, &data.budgets, selected_month);
    let warnings = monthly_budget_attention_warnings(data, selected_month);
    append_attention_warning_card(&ui_handles.categories, &warnings);

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
    let state_for_expenses = Rc::clone(state);
    let ui_for_expenses = Rc::clone(ui_handles);
    let state_for_income = Rc::clone(state);
    let ui_for_income = Rc::clone(ui_handles);
    let state_for_balance = Rc::clone(state);
    let ui_for_balance = Rc::clone(ui_handles);

    ui_handles.categories.append(&ui::metric_grid(
        vec![
            ui::activatable_metric_card(
                "Expenses",
                &money(month_totals.expenses),
                &month_label,
                move || {
                    show_transactions_filter(
                        &state_for_expenses,
                        &ui_for_expenses,
                        TransactionFilter::expenses_for_month(selected_month),
                    );
                },
            ),
            ui::activatable_metric_card(
                "Income",
                &money(month_totals.income),
                &month_label,
                move || {
                    show_transactions_filter(
                        &state_for_income,
                        &ui_for_income,
                        TransactionFilter::income_for_month(selected_month),
                    );
                },
            ),
            ui::activatable_metric_card(
                "Balance",
                &signed_money(month_totals.balance),
                &month_label,
                move || {
                    show_transactions_filter(
                        &state_for_balance,
                        &ui_for_balance,
                        TransactionFilter::month(selected_month),
                    );
                },
            ),
        ],
        3,
    ));

    if append_budget_pie_charts(
        &ui_handles.categories,
        BudgetPieChartData {
            budget_rows: &budget_rows,
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

struct BudgetDetailSectionsData {
    budgets: Vec<analytics::BudgetUsage>,
    categories: Vec<analytics::CategorySummary>,
    month_label: String,
    month: MonthKey,
    search_active: bool,
    show_all: bool,
    has_immediate_results: bool,
}

fn append_budget_detail_sections(
    container: &gtk::Box,
    data: BudgetDetailSectionsData,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    let host = gtk::Box::new(gtk::Orientation::Vertical, 0);
    host.set_hexpand(true);
    container.append(&host);
    append_budget_detail_loading_sections(&host, &data);

    let generation = ui_handles.render_generation.get();
    let ui_for_sections = Rc::clone(ui_handles);
    let state_for_sections = Rc::clone(state);
    gtk::glib::idle_add_local_once(move || {
        if ui_for_sections.render_generation.get() != generation {
            return;
        }

        let search_active = data.search_active;
        let has_immediate_results = data.has_immediate_results;
        ui::clear_box(&host);
        let has_detail_results =
            append_budget_detail_sections_now(&host, data, &ui_for_sections, &state_for_sections);
        if search_active && !has_immediate_results && !has_detail_results {
            host.append(&search_empty_page(
                "No budget results",
                "No budgets or categories match this search term.",
            ));
        }
        host.queue_resize();
        host.queue_draw();
    });
}

fn append_budget_detail_loading_sections(container: &gtk::Box, data: &BudgetDetailSectionsData) {
    let mut sections = Vec::new();
    if !data.search_active || !data.budgets.is_empty() {
        sections.push(ui::loading_section_group(
            "Budget Room",
            &budget_room_subtitle(data.search_active, &data.month_label),
        ));
    }
    if !data.search_active
        || data
            .categories
            .iter()
            .any(|category| category.totals.expenses > Decimal::ZERO)
    {
        sections.push(ui::loading_section_group(
            "Monthly Spending",
            &monthly_spending_subtitle(&data.month_label),
        ));
    }
    if sections.is_empty() && !data.has_immediate_results {
        sections.push(ui::loading_section_group("Budget", "Loading data..."));
    }
    if !sections.is_empty() {
        container.append(&ui::responsive_columns_three_or_one(sections));
    }
}

fn budget_room_subtitle(search_active: bool, month_label: &str) -> String {
    if search_active {
        trf(
            "Budgets for {month}. Yearly-only budgets use remaining annual room.",
            &[("month", month_label.to_string())],
        )
    } else {
        trf(
            "Budgets needing attention in {month}. More shows everything; yearly-only budgets use remaining annual room.",
            &[("month", month_label.to_string())],
        )
    }
}

fn monthly_spending_subtitle(month_label: &str) -> String {
    trf(
        "Largest monthly expenses by category in {month}. The bar shows the share compared with the largest expense in this list.",
        &[("month", month_label.to_string())],
    )
}

fn append_budget_detail_sections_now(
    container: &gtk::Box,
    data: BudgetDetailSectionsData,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) -> bool {
    let mut sections = Vec::new();
    if !data.budgets.is_empty() {
        let visible_budgets = if data.search_active || data.show_all {
            data.budgets.clone()
        } else {
            data.budgets
                .iter()
                .filter(|budget| budget_needs_attention(budget))
                .cloned()
                .collect::<Vec<_>>()
        };
        let hidden_budgets = if data.show_all {
            0
        } else {
            data.budgets.len().saturating_sub(visible_budgets.len())
        };
        let section_subtitle = budget_room_subtitle(data.search_active, &data.month_label);
        let section = ui::card_list_section_group("Budget Room", &section_subtitle);
        let box_ = ui::card_grid(Vec::new(), 2);
        if visible_budgets.is_empty() {
            ui::append_card_to_grid(
                &box_,
                ui::text_card(&tr(
                    "All budgets are within plan. Use More to show every budget.",
                )),
            );
        } else {
            append_budget_rows(&box_, &visible_budgets, data.month, ui_handles, state);
        }
        section.append(&box_);
        if hidden_budgets > 0 {
            append_budgets_more_button(
                &section,
                &box_,
                data.budgets.clone(),
                data.month,
                ui_handles,
                state,
            );
        }
        sections.push(section);
    }

    let has_expense_categories = data
        .categories
        .iter()
        .any(|category| category.totals.expenses > Decimal::ZERO);
    if has_expense_categories || !data.search_active {
        sections.push(monthly_categories_section(
            data.categories,
            &data.month_label,
            data.month,
            ui_handles,
            state,
        ));
    }

    if sections.is_empty() {
        return false;
    }

    for section in sections {
        container.append(&section);
    }
    true
}

struct BudgetPieChartData<'a> {
    budget_rows: &'a [analytics::BudgetUsage],
    real_month_income: Decimal,
    planned_month_income: Decimal,
    categories: &'a [analytics::CategorySummary],
    month: MonthKey,
    month_label: &'a str,
}

fn append_budget_pie_charts(
    container: &gtk::Box,
    data: BudgetPieChartData<'_>,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) -> bool {
    let budget_rows = data.budget_rows;
    let real_month_income = data.real_month_income;
    let planned_month_income = data.planned_month_income;
    let categories = data.categories;
    let month = data.month;
    let month_label = data.month_label;
    let mut charts = Vec::new();
    let advanced_features = ui_handles.advanced_features.get();

    let mut planned = budget_rows
        .iter()
        .filter(|budget| budget.budget > Decimal::ZERO)
        .map(|budget| {
            (
                budget.code.clone(),
                ui::PieSlice::new(
                    budget_display_title(&budget.code, &budget.category, advanced_features),
                    budget.budget,
                    trf(
                        "{amount} planned",
                        &[(
                            "amount",
                            planned_budget_label(budget.budget, &budget.budget_basis),
                        )],
                    ),
                ),
            )
        })
        .collect::<Vec<_>>();
    ui::sort_pie_slices_largest_first(&mut planned);
    if !planned.is_empty() {
        let codes = planned
            .iter()
            .map(|(code, _)| code.clone())
            .collect::<Vec<_>>();
        let planned = planned
            .into_iter()
            .map(|(_, slice)| slice)
            .collect::<Vec<_>>();
        let total = planned.iter().map(|slice| slice.value).sum::<Decimal>();
        let state_for_chart = Rc::clone(state);
        let ui_for_chart = Rc::clone(ui_handles);
        charts.push(ui::pie_chart_with_capacity(
            "Budget distribution",
            &trf(
                "Planned monthly budgets for {month}.",
                &[("month", month_label.to_string())],
            ),
            &planned,
            &money(total),
            Some(planned_month_income),
            move |index| {
                if let Some(code) = codes.get(index) {
                    show_transactions_filter(
                        &state_for_chart,
                        &ui_for_chart,
                        TransactionFilter::budget_for_month(code.clone(), month),
                    );
                }
            },
        ));
    }

    let actual = categories
        .iter()
        .filter(|category| category.totals.expenses > Decimal::ZERO)
        .map(|category| {
            ui::PieSlice::new(
                category.category.clone(),
                category.totals.expenses,
                category_transaction_detail(
                    category.totals.count,
                    &category.budget_code,
                    advanced_features,
                ),
            )
        })
        .collect::<Vec<_>>();
    if !actual.is_empty() {
        let codes = categories
            .iter()
            .filter(|category| category.totals.expenses > Decimal::ZERO)
            .map(|category| category.budget_code.clone())
            .collect::<Vec<_>>();
        let total = actual.iter().map(|slice| slice.value).sum::<Decimal>();
        let state_for_chart = Rc::clone(state);
        let ui_for_chart = Rc::clone(ui_handles);
        charts.push(ui::pie_chart_with_capacity(
            "Spending distribution",
            &trf(
                "Actual expenses by category for {month}.",
                &[("month", month_label.to_string())],
            ),
            &actual,
            &money(total),
            Some(real_month_income),
            move |index| {
                if let Some(code) = codes.get(index) {
                    show_transactions_filter(
                        &state_for_chart,
                        &ui_for_chart,
                        TransactionFilter::budget_for_month(code.clone(), month),
                    );
                }
            },
        ));
    }

    if charts.is_empty() {
        return false;
    }

    container.append(&ui::responsive_chart_columns(charts));
    true
}

pub(in crate::app) fn append_budget_rows(
    container: &gtk::FlowBox,
    budgets: &[analytics::BudgetUsage],
    month: MonthKey,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    let advanced_features = ui_handles.advanced_features.get();
    for budget in budgets {
        let (detail, state_kind) = budget_progress_detail(budget);
        let state_for_row = Rc::clone(state);
        let ui_for_row = Rc::clone(ui_handles);
        let filter = TransactionFilter::budget_for_month(budget.code.clone(), month);
        let edit_button = budget_edit_button(&budget.code, &budget.category, ui_handles, state)
            .upcast::<gtk::Widget>();
        let title = budget_display_title(&budget.code, &budget.category, advanced_features);
        let row = ui::progress_row_with_state_and_action(
            &title,
            &trf(
                "{amount} spent of {budget}",
                &[
                    ("amount", money(budget.actual)),
                    (
                        "budget",
                        planned_budget_label(budget.budget, &budget.budget_basis),
                    ),
                ],
            ),
            fraction(budget.actual, budget.budget),
            &detail,
            state_kind,
            Some(edit_button),
        );
        let card = ui::activatable_card(row, move || {
            show_transactions_filter(&state_for_row, &ui_for_row, filter.clone())
        });
        ui::append_card_to_grid(container, card);
    }
}

pub(in crate::app) fn append_budgets_more_button(
    section: &gtk::Box,
    rows_box: &gtk::FlowBox,
    budgets: Vec<analytics::BudgetUsage>,
    month: MonthKey,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    let more_button = more_budgets_button();
    let rows_box = rows_box.clone();
    let ui_for_more = Rc::clone(ui_handles);
    let state_for_more = Rc::clone(state);
    more_button.connect_clicked(move |button| {
        ui::clear_card_grid(&rows_box);
        append_budget_rows(&rows_box, &budgets, month, &ui_for_more, &state_for_more);
        button.set_visible(false);
    });
    section.append(&more_button);
}

pub(in crate::app) fn budget_needs_attention(budget: &analytics::BudgetUsage) -> bool {
    budget.actual > Decimal::ZERO
        && (budget.budget <= Decimal::ZERO || budget.remaining <= Decimal::ZERO)
}

pub(in crate::app) fn monthly_categories_section(
    categories: Vec<analytics::CategorySummary>,
    month_label: &str,
    month: MonthKey,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) -> gtk::Box {
    let section =
        ui::card_list_section_group("Monthly Spending", &monthly_spending_subtitle(month_label));
    let categories_box = ui::card_grid(Vec::new(), 2);
    let expense_categories = categories
        .into_iter()
        .filter(|category| category.totals.expenses > Decimal::ZERO)
        .collect::<Vec<_>>();
    if expense_categories.is_empty() {
        ui::append_card_to_grid(
            &categories_box,
            ui::text_card(&trf(
                "No monthly expenses in {month}.",
                &[("month", month_label.to_string())],
            )),
        );
        section.append(&categories_box);
        return section;
    }

    let show_all = ui_handles.show_all.get();
    let preview = expense_categories
        .iter()
        .take(if show_all {
            usize::MAX
        } else {
            CATEGORY_PREVIEW_LIMIT
        })
        .cloned()
        .collect::<Vec<_>>();
    append_month_category_rows(&categories_box, &preview, month, ui_handles, state);
    let has_more = !show_all && expense_categories.len() > preview.len();
    section.append(&categories_box);
    if has_more {
        append_month_categories_more_button(
            &section,
            &categories_box,
            expense_categories,
            month,
            ui_handles,
            state,
        );
    }

    section
}

pub(in crate::app) fn append_month_categories_more_button(
    section: &gtk::Box,
    rows_box: &gtk::FlowBox,
    categories: Vec<analytics::CategorySummary>,
    month: MonthKey,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    let more_button = more_categories_button();
    let rows_box = rows_box.clone();
    let ui_for_more = Rc::clone(ui_handles);
    let state_for_more = Rc::clone(state);
    more_button.connect_clicked(move |button| {
        ui::clear_card_grid(&rows_box);
        append_month_category_rows(&rows_box, &categories, month, &ui_for_more, &state_for_more);
        button.set_visible(false);
    });
    section.append(&more_button);
}

pub(in crate::app) fn append_month_category_rows(
    container: &gtk::FlowBox,
    categories: &[analytics::CategorySummary],
    month: MonthKey,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    let max_expense = categories
        .iter()
        .map(|category| category.totals.expenses)
        .max()
        .unwrap_or(Decimal::ONE)
        .max(Decimal::ONE);
    let advanced_features = ui_handles.advanced_features.get();
    for category in categories {
        let state_for_row = Rc::clone(state);
        let ui_for_row = Rc::clone(ui_handles);
        let filter = TransactionFilter::budget_for_month(category.budget_code.clone(), month);
        let edit_button =
            budget_edit_button(&category.budget_code, &category.category, ui_handles, state)
                .upcast::<gtk::Widget>();
        let row = ui::progress_row_with_action(
            &category.category,
            &category_transaction_detail(
                category.totals.count,
                &category.budget_code,
                advanced_features,
            ),
            fraction(category.totals.expenses, max_expense),
            &money(category.totals.expenses),
            Some(edit_button),
        );
        let card = ui::activatable_card(row, move || {
            show_transactions_filter(&state_for_row, &ui_for_row, filter.clone())
        });
        ui::append_card_to_grid(container, card);
    }
}

pub(in crate::app) fn more_categories_button() -> gtk::Button {
    more_button("Show all spending")
}

pub(in crate::app) fn more_budgets_button() -> gtk::Button {
    more_button("Show all budgets")
}

fn more_button(tooltip: &str) -> gtk::Button {
    ui::plain_text_icon_button("view-more-symbolic", "More", tooltip)
}

pub(in crate::app) fn budget_usage_matches(
    budget: &analytics::BudgetUsage,
    filter: &SearchFilter,
) -> bool {
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

pub(in crate::app) fn category_summary_matches(
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
