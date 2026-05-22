use crate::analytics;
use crate::app_info::{self, APP_ID};
use crate::data::{self, EditableAlias, EditableBudget, EditableRule};
use crate::i18n::{self, gettext};
use crate::model::{
    AppData, BudgetDirection, ComparisonMode, DedupeMode, FieldMap, ImportReport, MonthKey,
    Transaction, TransactionLoadScope,
};
#[cfg(all(target_os = "linux", feature = "setup", not(feature = "flatpak")))]
use crate::setup;
use crate::summary;
use crate::ui;
use crate::updater;
use crate::util::{money, signed_money};
use adw::glib::value::ToValue;
use adw::glib::variant::ToVariant;
use adw::gtk;
use adw::gtk::gio::prelude::FileExt;
use adw::gtk::prelude::*;
use adw::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

mod actions;
mod annual;
mod budget;
mod config_ops;
mod configuration_dialog;
mod core;
mod diagnostics;
mod export;
mod fake_transactions;
mod field_alias;
mod filters;
mod form_autofill;
mod import;
mod management;
mod money;
mod operation_queue;
mod overview;
mod planned_income;
mod preferences;
mod preferences_dialog;
mod print;
mod render;
mod settings_dialog;
mod shell;
mod shortcuts;
mod smart;
mod status;
mod text;
mod transactions;
mod warning_cards;
mod warnings;
mod year;

pub use core::run;
#[cfg(target_os = "linux")]
pub(crate) use transactions::transaction_search_text;

use actions::connect_actions;
use annual::{
    annual_budget_matches, annual_budgets_section, annual_category_matches,
    annual_spending_section_from_rows, append_annual_budget_row,
};
use budget::{
    bind_percentage_basis_visibility, budget_direction_change, budget_edit_button,
    budget_values_use_percentage, confirm_budget_direction_changes,
    generate_budgets_from_transactions_with_status, more_budgets_button, more_categories_button,
    render_budget_page, selected_budget_month, selected_year, show_budget_edit_dialog,
    totals_for_month, year_selector_row, BudgetDirectionChange,
};
use config_ops::{
    config_operation_is_active, finish_config_operation, register_config_widget,
    register_exclusive_config_widget, save_rule_in_background, set_config_widget_base_sensitive,
    try_begin_config_operation, update_config_action_widgets, ConfigWidget,
};
use configuration_dialog::show_configuration_dialog;
use core::{
    apply_action_availability, begin_background_operation, build_ui, build_ui_with_opened_uris,
    comparison_mode, config_write_availability, current_transaction_load_scope,
    data_write_availability, finish_background_operation, navigate_back, refresh_write_actions,
    set_storage_capabilities, tr, trf, ActionAvailability, UiHandles, ACTIVE_SESSION,
    CATEGORY_PREVIEW_LIMIT, SEARCH_CATEGORY_PREVIEW_LIMIT,
};
use diagnostics::{delimiter_label, empty_page, render_diagnostics_page};
use export::export_transactions_from_action;
use fake_transactions::{
    build_fake_transaction_widgets, connect_fake_transactions, data_with_fake_transactions,
    duplicate_transaction_as_fake, real_transactions, transaction_is_fake, FakeTransactionStore,
    FakeTransactionWidgets,
};
use field_alias::{field_alias_combo, FIELD_ALIAS_SPECS};
use filters::{
    active_search, connect_search, current_page, filtered_app_data, page_data_for_render,
    show_transactions_filter, AppPage, SearchFilter, TransactionFilter,
};
use form_autofill::{
    app_budget_autofill_entries, app_budget_code_values, app_category_values,
    connect_budget_fields_autofill, editable_budget_autofill_entries, editable_budget_code_values,
    editable_category_values, editable_rule_search_values, pattern_rule_search_values,
    transaction_rule_search_values,
};
use import::{
    connect_drop_target, import_and_reload_in_background, import_uris_into_session, reload_state,
    reload_state_with_scope, reload_state_with_status, set_dedupe_enabled,
};
use management::show_management_dialog;
use money::{
    annual_budget_previous_state, annual_budget_progress_detail, budget_display_title,
    budget_progress_detail, category_transaction_detail, file_size, format_size, fraction,
    planned_budget_label,
};
use operation_queue::{
    build_operation_queue_widgets, connect_operation_queue, enqueue_rule_operation, OperationQueue,
    OperationQueueWidgets, OperationSource,
};
use overview::render_overview;
use preferences::Preferences;
use preferences_dialog::show_preferences_dialog;
use print::{current_print_report, print_report, table_print_report};
use render::{render_loading_placeholder, render_views, request_render_views};
use settings_dialog::{
    build_settings_header, connect_preference_search, SearchablePreferencesGroup,
};
#[cfg(all(target_os = "linux", feature = "setup", not(feature = "flatpak")))]
use shell::build_menu_model;
use shell::{
    add_responsive_page_margins, add_responsive_switcher, add_responsive_switcher_for_dialog,
    build_menu, open_files, refresh_menu,
};
use shortcuts::{build_shortcuts_dialog, install_action_accelerators};
use smart::{effective_hide_canceled_transactions, smart_pattern_detection_enabled};
use status::{
    build_status_bar, connect_embedded_status_bar, connect_static_page_actions,
    connect_status_actions, register_page_copy_feedback_button,
    schedule_status_autohide_after_loading, set_page_actions_menu_namespace,
    show_page_copy_feedback, show_status, StaticPageSnapshot, StatusLogEntry,
};
use text::truncate;
use transactions::{
    append_page_header, current_page_snapshot, current_real_page_snapshot, filtered_transactions,
    render_transactions_page, search_empty_page,
};
use warning_cards::append_attention_warning_card;
use warnings::{
    annual_budget_attention_warnings, attention_warning_messages, monthly_budget_attention_warnings,
};
use year::render_year_comparison;
