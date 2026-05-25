use super::super::{
    render_views, selected_budget_month, selected_year, show_status, AppData, UiHandles,
};
use super::search::show_search_status;
use super::{current_page, AppPage, TransactionFilter};
use adw::gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub(in crate::app) const SEARCH_PRESET_ACTION: &str = "search-preset";
pub(in crate::app) const SEARCH_PRESET_DETAILED_ACTION: &str = "app.search-preset";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::app) enum SearchPresetSection {
    General,
    Transactions,
    Diagnostics,
}

impl SearchPresetSection {
    pub(in crate::app) fn label(self) -> Option<&'static str> {
        match self {
            Self::General => None,
            Self::Transactions => Some("Transactions"),
            Self::Diagnostics => Some("Diagnostics"),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::app) struct SearchPresetSpec {
    pub(in crate::app) section: SearchPresetSection,
    pub(in crate::app) label: &'static str,
    pub(in crate::app) id: &'static str,
}

const SEARCH_PRESET_SPECS: &[SearchPresetSpec] = &[
    SearchPresetSpec {
        section: SearchPresetSection::General,
        label: "Clear Filter",
        id: "clear",
    },
    SearchPresetSpec {
        section: SearchPresetSection::Transactions,
        label: "Income / positive",
        id: "income",
    },
    SearchPresetSpec {
        section: SearchPresetSection::Transactions,
        label: "Costs / negative",
        id: "expense",
    },
    SearchPresetSpec {
        section: SearchPresetSection::Transactions,
        label: "Transfers",
        id: "transfer",
    },
    SearchPresetSpec {
        section: SearchPresetSection::Transactions,
        label: "Current Month",
        id: "current-month",
    },
    SearchPresetSpec {
        section: SearchPresetSection::Transactions,
        label: "Current Year",
        id: "current-year",
    },
    SearchPresetSpec {
        section: SearchPresetSection::Diagnostics,
        label: "Unconfigured Budgets",
        id: "unconfigured-budgets",
    },
    SearchPresetSpec {
        section: SearchPresetSection::Diagnostics,
        label: "Other Categories",
        id: "other-categories",
    },
    SearchPresetSpec {
        section: SearchPresetSection::Diagnostics,
        label: "Warnings",
        id: "warnings",
    },
    SearchPresetSpec {
        section: SearchPresetSection::Diagnostics,
        label: "Import Reports",
        id: "imports",
    },
    SearchPresetSpec {
        section: SearchPresetSection::Diagnostics,
        label: "Field Mappings",
        id: "fields",
    },
    SearchPresetSpec {
        section: SearchPresetSection::Diagnostics,
        label: "Rules",
        id: "rules",
    },
];

pub(in crate::app) fn search_preset_specs() -> &'static [SearchPresetSpec] {
    SEARCH_PRESET_SPECS
}

pub(in crate::app) fn apply_search_preset(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    preset_id: &str,
) {
    let Some(preset) = SearchPreset::from_id(preset_id) else {
        return;
    };
    let query = {
        let data = state.borrow();
        match preset.query(&data, ui) {
            Some(query) => query,
            None => {
                show_status(ui, preset.unavailable_message());
                return;
            }
        }
    };

    if let Some(page) = preset.target_page() {
        if current_page(ui) != page {
            ui.stack.set_visible_child_name(page.stack_name());
        }
    }

    *ui.active_transaction_filter.borrow_mut() = TransactionFilter::from_query(&query);
    *ui.search_query.borrow_mut() = query.clone();
    ui.search_bar.set_search_mode(!query.is_empty());
    if ui.search_entry.text().as_str() != query.as_str() {
        ui.search_entry.set_text(&query);
    }
    if !query.is_empty() {
        ui.search_entry.grab_focus();
        ui.search_entry.select_region(0, -1);
    }

    render_views(&state.borrow(), ui, state);
    show_search_status(ui, &query);
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum SearchPreset {
    Clear,
    Income,
    Expense,
    Transfer,
    CurrentMonth,
    CurrentYear,
    UnconfiguredBudgets,
    OtherCategories,
    Warnings,
    Imports,
    Fields,
    Rules,
}

impl SearchPreset {
    pub(super) fn from_id(id: &str) -> Option<Self> {
        match id {
            "clear" => Some(Self::Clear),
            "income" => Some(Self::Income),
            "expense" => Some(Self::Expense),
            "transfer" => Some(Self::Transfer),
            "current-month" => Some(Self::CurrentMonth),
            "current-year" => Some(Self::CurrentYear),
            "unconfigured-budgets" => Some(Self::UnconfiguredBudgets),
            "other-categories" => Some(Self::OtherCategories),
            "warnings" => Some(Self::Warnings),
            "imports" => Some(Self::Imports),
            "fields" => Some(Self::Fields),
            "rules" => Some(Self::Rules),
            _ => None,
        }
    }

    fn query(self, data: &AppData, ui: &UiHandles) -> Option<String> {
        match self {
            Self::Clear => Some(String::new()),
            Self::Income => Some("amount:income".to_string()),
            Self::Expense => Some("amount:expense".to_string()),
            Self::Transfer => Some("amount:transfer".to_string()),
            Self::CurrentMonth => selected_budget_month(data, ui)
                .map(TransactionFilter::month)
                .map(|filter| filter.query()),
            Self::CurrentYear => selected_year(data, ui)
                .map(TransactionFilter::year)
                .map(|filter| filter.query()),
            Self::UnconfiguredBudgets => Some(TransactionFilter::UnconfiguredBudgets.query()),
            Self::OtherCategories => Some(TransactionFilter::OtherCategories.query()),
            Self::Warnings => Some("warnings".to_string()),
            Self::Imports => Some("imports".to_string()),
            Self::Fields => Some("fields".to_string()),
            Self::Rules => Some("rules".to_string()),
        }
    }

    pub(super) fn target_page(self) -> Option<AppPage> {
        match self {
            Self::Clear => None,
            Self::Income
            | Self::Expense
            | Self::Transfer
            | Self::CurrentMonth
            | Self::CurrentYear
            | Self::UnconfiguredBudgets
            | Self::OtherCategories => Some(AppPage::Transactions),
            Self::Warnings | Self::Imports | Self::Fields | Self::Rules => {
                Some(AppPage::Diagnostics)
            }
        }
    }

    fn unavailable_message(self) -> &'static str {
        match self {
            Self::CurrentMonth => "No month is available for this filter yet.",
            Self::CurrentYear => "No year is available for this filter yet.",
            Self::Clear
            | Self::Income
            | Self::Expense
            | Self::Transfer
            | Self::UnconfiguredBudgets
            | Self::OtherCategories
            | Self::Warnings
            | Self::Imports
            | Self::Fields
            | Self::Rules => "This filter is not available yet.",
        }
    }
}
