use super::*;

pub(in crate::app) fn annual_budgets_section(
    data: &AppData,
    year: i32,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) -> Option<gtk::Box> {
    let budgets = analytics::annual_budget_usage(
        &data.transactions,
        &data.budgets,
        year,
        comparison_mode(ui_handles.as_ref()),
    );
    if budgets.is_empty() {
        return None;
    }

    let subtitle = if ui_handles.compare_categories_previous_period.get() {
        trf(
            "Budgets needing attention in {year}. More shows everything; gray bars show {previous_year}.",
            &[
                ("year", year.to_string()),
                ("previous_year", (year - 1).to_string()),
            ],
        )
    } else {
        trf(
            "Budgets needing attention in {year}. More shows everything.",
            &[("year", year.to_string())],
        )
    };
    let section = ui::section_group("Annual Budgets", &subtitle);
    let budgets_box = annual_budget_grid();
    let show_all = ui_handles.show_all.get();
    let visible_budgets = if show_all {
        budgets.clone()
    } else {
        budgets
            .iter()
            .filter(|budget| annual_budget_needs_attention(budget))
            .cloned()
            .collect::<Vec<_>>()
    };
    let hidden_budgets = if show_all {
        0
    } else {
        budgets.len().saturating_sub(visible_budgets.len())
    };
    if visible_budgets.is_empty() {
        budgets_box.append(&ui::text_card(&tr(
            "All annual budgets are within plan. Use More to show every budget.",
        )));
    } else {
        append_annual_budget_rows(&budgets_box, &visible_budgets, year, ui_handles, state);
    }
    section.append(&budgets_box);
    if hidden_budgets > 0 {
        append_annual_budgets_more_button(&section, &budgets_box, budgets, year, ui_handles, state);
    }
    Some(section)
}

const ANNUAL_BUDGET_CARD_WIDTH: i32 = 320;
const ANNUAL_BUDGET_COLUMN_SPACING: i32 = 8;
const ANNUAL_BUDGET_LINE_SPACING: i32 = 8;
const ANNUAL_BUDGET_NATURAL_LINE_LENGTH: i32 =
    ANNUAL_BUDGET_CARD_WIDTH * 2 + ANNUAL_BUDGET_COLUMN_SPACING;

fn annual_budget_grid() -> adw::WrapBox {
    adw::WrapBox::builder()
        .orientation(gtk::Orientation::Horizontal)
        .child_spacing(ANNUAL_BUDGET_COLUMN_SPACING)
        .child_spacing_unit(adw::LengthUnit::Px)
        .line_spacing(ANNUAL_BUDGET_LINE_SPACING)
        .line_spacing_unit(adw::LengthUnit::Px)
        .natural_line_length(ANNUAL_BUDGET_NATURAL_LINE_LENGTH)
        .natural_line_length_unit(adw::LengthUnit::Px)
        .wrap_policy(adw::WrapPolicy::Natural)
        .justify(adw::JustifyMode::Fill)
        .hexpand(true)
        .halign(gtk::Align::Fill)
        .build()
}

fn append_annual_budget_rows(
    container: &adw::WrapBox,
    budgets: &[analytics::AnnualBudgetUsage],
    year: i32,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    for budget in budgets {
        append_annual_budget_wrap_row(container, budget, year, ui_handles, state);
    }
}

fn append_annual_budget_wrap_row(
    container: &adw::WrapBox,
    budget: &analytics::AnnualBudgetUsage,
    year: i32,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    let row = annual_budget_row(budget, year, ui_handles, state);
    row.set_hexpand(true);
    row.set_halign(gtk::Align::Fill);
    row.set_width_request(ANNUAL_BUDGET_CARD_WIDTH);
    row.set_valign(gtk::Align::Start);
    container.append(&row);
}

pub(in crate::app) fn append_annual_budget_row(
    container: &gtk::Box,
    budget: &analytics::AnnualBudgetUsage,
    year: i32,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    container.append(&annual_budget_row(budget, year, ui_handles, state));
}

fn annual_budget_row(
    budget: &analytics::AnnualBudgetUsage,
    year: i32,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) -> gtk::Box {
    let previous_actual = budget.previous_actual.unwrap_or(Decimal::ZERO);
    let scale = if budget.budget > Decimal::ZERO {
        budget.budget
    } else {
        budget.actual.max(previous_actual).max(Decimal::ONE)
    };
    let (detail, state_kind) = annual_budget_progress_detail(budget);
    let previous = budget.previous_actual.map(|amount| ui::ComparisonMeasure {
        label: trf(
            "{year} spent - {amount}",
            &[("year", (year - 1).to_string()), ("amount", money(amount))],
        ),
        fraction: fraction(amount, scale),
        state: annual_budget_previous_state(budget),
    });
    let state_for_row = Rc::clone(state);
    let ui_for_row = Rc::clone(ui_handles);
    let filter = TransactionFilter::budget_for_year(budget.code.clone(), year);
    let edit_button = budget_edit_button(&budget.code, &budget.category, ui_handles, state)
        .upcast::<gtk::Widget>();
    let row = ui::comparison_progress_row_with_action(
        ui::ComparisonProgressRow {
            title: format!("{} · {}", budget.code, budget.category),
            subtitle: trf(
                "Planned annual budget: {budget}",
                &[(
                    "budget",
                    planned_budget_label(budget.budget, &budget.budget_basis),
                )],
            ),
            current: ui::ComparisonMeasure {
                label: trf(
                    "{year} spent - {amount}",
                    &[("year", year.to_string()), ("amount", money(budget.actual))],
                ),
                fraction: fraction(budget.actual, scale),
                state: state_kind,
            },
            previous,
            detail,
        },
        Some(edit_button),
    );
    ui::activatable_card(row, move || {
        show_transactions_filter(&state_for_row, &ui_for_row, filter.clone())
    })
}

pub(in crate::app) fn append_annual_budgets_more_button(
    section: &gtk::Box,
    rows_box: &adw::WrapBox,
    budgets: Vec<analytics::AnnualBudgetUsage>,
    year: i32,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    let more_button = more_budgets_button();
    let rows_box = rows_box.clone();
    let ui_for_more = Rc::clone(ui_handles);
    let state_for_more = Rc::clone(state);
    more_button.connect_clicked(move |button| {
        rows_box.remove_all();
        append_annual_budget_rows(&rows_box, &budgets, year, &ui_for_more, &state_for_more);
        button.set_visible(false);
    });
    section.append(&more_button);
}

pub(in crate::app) fn annual_budget_needs_attention(budget: &analytics::AnnualBudgetUsage) -> bool {
    budget.actual > Decimal::ZERO
        && (budget.budget <= Decimal::ZERO || budget.remaining <= Decimal::ZERO)
}

pub(in crate::app) fn annual_budget_matches(
    budget: &analytics::AnnualBudgetUsage,
    filter: &SearchFilter,
) -> bool {
    filter.matches(&format!(
        "{} {} {} {} {} {} {}",
        budget.code,
        budget.category,
        budget.notes,
        budget.budget_basis,
        money(budget.budget),
        money(budget.actual),
        money(budget.previous_actual.unwrap_or(Decimal::ZERO)),
    ))
}

pub(in crate::app) fn annual_spending_section_from_rows(
    categories: Vec<analytics::CategorySummary>,
    year: i32,
    subtitle: &str,
    preview_limit: usize,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) -> gtk::Box {
    let section = ui::section_group("Annual Spending", subtitle);
    let categories_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
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
        append_annual_categories_more_button(
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

fn append_annual_categories_more_button(
    section: &gtk::Box,
    rows_box: &gtk::Box,
    categories: Vec<analytics::CategorySummary>,
    year: i32,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    let more_button = more_categories_button();
    let rows_box = rows_box.clone();
    let ui_for_more = Rc::clone(ui_handles);
    let state_for_more = Rc::clone(state);
    more_button.connect_clicked(move |button| {
        ui::clear_box(&rows_box);
        append_annual_category_rows(&rows_box, &categories, year, &ui_for_more, &state_for_more);
        button.set_visible(false);
    });
    section.append(&more_button);
}

fn append_annual_category_rows(
    container: &gtk::Box,
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
    container: &gtk::Box,
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
        &trf(
            "{count} transactions · budget code {code}",
            &[
                ("count", category.totals.count.to_string()),
                ("code", category.budget_code.clone()),
            ],
        ),
        fraction(category.totals.expenses, max_expense),
        &money(category.totals.expenses),
        Some(edit_button),
    );
    let card = ui::activatable_card(row, move || {
        show_transactions_filter(&state_for_row, &ui_for_row, filter.clone())
    });
    container.append(&card);
}

pub(in crate::app) fn annual_category_matches(
    category: &analytics::CategorySummary,
    filter: &SearchFilter,
) -> bool {
    filter.matches(&format!(
        "{} {} {} {} {} {}",
        category.category,
        category.budget_code,
        category.totals.count,
        money(category.totals.income),
        money(category.totals.expenses),
        signed_money(category.totals.balance),
    ))
}
