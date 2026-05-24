use super::super::*;

pub(super) fn monthly_spending_subtitle(month_label: &str) -> String {
    trf(
        "Largest monthly expenses by category in {month}. The bar shows the share compared with the largest expense in this list.",
        &[("month", month_label.to_string())],
    )
}

pub(super) fn append_budget_rows(
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

pub(super) fn append_budgets_more_button(
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

pub(super) fn budget_needs_attention(budget: &analytics::BudgetUsage) -> bool {
    budget.actual > Decimal::ZERO
        && (budget.budget <= Decimal::ZERO || budget.remaining <= Decimal::ZERO)
}

pub(super) fn monthly_categories_section(
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

pub(super) fn append_month_categories_more_button(
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

pub(super) fn append_month_category_rows(
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
