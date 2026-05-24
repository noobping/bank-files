use super::super::*;
use super::rows::{
    append_budget_rows, append_budgets_more_row, budget_needs_attention,
    monthly_categories_section, monthly_spending_subtitle,
};

pub(super) struct BudgetDetailSectionsData {
    pub(super) budgets: Vec<analytics::BudgetUsage>,
    pub(super) categories: Vec<analytics::CategorySummary>,
    pub(super) month_label: String,
    pub(super) month: MonthKey,
    pub(super) search_active: bool,
    pub(super) show_all: bool,
    pub(super) has_immediate_results: bool,
}

pub(super) fn append_budget_detail_sections(
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

pub(super) fn append_budget_detail_loading_sections(
    container: &gtk::Box,
    data: &BudgetDetailSectionsData,
) {
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

pub(super) fn budget_room_subtitle(search_active: bool, month_label: &str) -> String {
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

pub(super) fn append_budget_detail_sections_now(
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
            append_budgets_more_row(
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
