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
            "Budgets needing attention in {year}. More shows everything; faded bars show {previous_year}.",
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
    let section = ui::card_list_section_group("Annual Budgets", &subtitle);
    let budgets_box = ui::card_grid(Vec::new(), 2);
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
        ui::append_card_to_grid(
            &budgets_box,
            ui::text_card(&tr(
                "All annual budgets are within plan. Use More to show every budget.",
            )),
        );
    } else {
        append_annual_budget_rows(&budgets_box, &visible_budgets, year, ui_handles, state);
    }
    section.append(&budgets_box);
    if hidden_budgets > 0 {
        append_annual_budgets_more_button(&section, &budgets_box, budgets, year, ui_handles, state);
    }
    Some(section)
}

fn append_annual_budget_rows(
    container: &gtk::FlowBox,
    budgets: &[analytics::AnnualBudgetUsage],
    year: i32,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    for budget in budgets {
        append_annual_budget_row(container, budget, year, ui_handles, state);
    }
}

pub(in crate::app) fn append_annual_budget_row(
    container: &gtk::FlowBox,
    budget: &analytics::AnnualBudgetUsage,
    year: i32,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    ui::append_card_to_grid(
        container,
        annual_budget_row(budget, year, ui_handles, state),
    );
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
    let title = budget_display_title(
        &budget.code,
        &budget.category,
        ui_handles.advanced_features.get(),
    );
    let row = ui::comparison_progress_row_with_action(
        ui::ComparisonProgressRow {
            title,
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
    rows_box: &gtk::FlowBox,
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
        ui::clear_card_grid(&rows_box);
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
    filter.matches_summary(&format!(
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
