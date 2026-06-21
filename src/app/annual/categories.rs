use super::*;

pub(in crate::app) fn annual_spending_section_from_rows(
    categories: Vec<analytics::CategorySummary>,
    year: i32,
    subtitle: &str,
    preview_limit: usize,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) -> gtk::Box {
    let section = ui::card_list_section_group("Annual Spending", subtitle);
    let categories_box = ui::card_grid(Vec::new(), 2);
    let show_all = ui_handles.show_all.get();
    let preview = categories
        .iter()
        .filter(|category| category.totals.expenses > Decimal::ZERO)
        .take(if show_all { usize::MAX } else { preview_limit })
        .cloned()
        .collect::<Vec<_>>();
    append_annual_category_rows(&categories_box, &preview, year, ui_handles, state);
    let has_more = !show_all && categories.len() > preview.len();
    section.append(&categories_box);
    if has_more {
        append_annual_categories_more_row(
            &section,
            &categories_box,
            categories,
            year,
            ui_handles,
            state,
        );
    }
    section
}

fn append_annual_categories_more_row(
    section: &gtk::Box,
    rows_box: &gtk::FlowBox,
    categories: Vec<analytics::CategorySummary>,
    year: i32,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    let more_row = more_categories_row();
    let more_container = more_row.container.clone();
    let rows_box = rows_box.clone();
    let ui_for_more = Rc::clone(ui_handles);
    let state_for_more = Rc::clone(state);
    more_row.row.connect_activated(move |_| {
        ui::clear_card_grid(&rows_box);
        append_annual_category_rows(&rows_box, &categories, year, &ui_for_more, &state_for_more);
        more_container.set_visible(false);
    });
    section.append(&more_row.container);
}

fn append_annual_category_rows(
    container: &gtk::FlowBox,
    categories: &[analytics::CategorySummary],
    year: i32,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    let max_expense = categories
        .iter()
        .map(|category| category.totals.expenses)
        .max()
        .unwrap_or(Decimal::ONE)
        .max(Decimal::ONE);
    for category in categories {
        append_annual_category_row(container, category, year, max_expense, ui_handles, state);
    }
}

fn append_annual_category_row(
    container: &gtk::FlowBox,
    category: &analytics::CategorySummary,
    year: i32,
    max_expense: Decimal,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    let state_for_row = Rc::clone(state);
    let ui_for_row = Rc::clone(ui_handles);
    let filter = TransactionFilter::budget_for_year(category.budget_code.clone(), year);
    let edit_button =
        budget_edit_button(&category.budget_code, &category.category, ui_handles, state)
            .upcast::<gtk::Widget>();
    let row = ui::progress_row_with_action(
        &category.category,
        &category_transaction_detail(
            category.totals.count,
            &category.budget_code,
            ui_handles.advanced_features.get(),
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

pub(in crate::app) fn annual_category_matches(
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
