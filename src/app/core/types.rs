use super::*;

#[derive(Clone)]
pub(in crate::app) struct LoadingSensitiveWidget {
    pub(super) widget: gtk::Widget,
    pub(super) base_sensitive: bool,
    pub(super) was_rooted: Rc<Cell<bool>>,
}

#[derive(Clone)]
pub(in crate::app) struct SearchToggleHandle {
    pub(in crate::app) search_bar: gtk::SearchBar,
    pub(in crate::app) search_entry: gtk::SearchEntry,
}

#[derive(Clone)]
pub(in crate::app) struct UiHandles {
    pub(in crate::app) window: adw::ApplicationWindow,
    pub(in crate::app) stack: adw::ViewStack,
    pub(in crate::app) overview: gtk::Box,
    pub(in crate::app) categories: gtk::Box,
    pub(in crate::app) transactions: gtk::Box,
    pub(in crate::app) debug: gtk::Box,
    pub(in crate::app) search_bar: gtk::SearchBar,
    pub(in crate::app) search_entry: gtk::SearchEntry,
    pub(in crate::app) mobile_header_title: adw::WindowTitle,
    pub(in crate::app) search_query: Rc<RefCell<String>>,
    pub(in crate::app) active_transaction_filter: Rc<RefCell<Option<TransactionFilter>>>,
    pub(in crate::app) import_button: gtk::Button,
    pub(in crate::app) loading_count: Rc<Cell<u32>>,
    pub(in crate::app) back_button: gtk::Button,
    pub(in crate::app) menu_button: gtk::MenuButton,
    pub(in crate::app) navigation_history: Rc<RefCell<Vec<NavigationEntry>>>,
    pub(in crate::app) navigation_current_page: Rc<RefCell<String>>,
    pub(in crate::app) navigation_is_restoring: Rc<Cell<bool>>,
    pub(in crate::app) status_bar: gtk::Box,
    pub(in crate::app) status_icon: gtk::Image,
    pub(in crate::app) status_loading_spinner: adw::Spinner,
    pub(in crate::app) status: gtk::Label,
    pub(in crate::app) status_history: Rc<RefCell<Vec<StatusLogEntry>>>,
    pub(in crate::app) operation_queue: OperationQueue,
    pub(in crate::app) operation_queue_widgets: OperationQueueWidgets,
    pub(in crate::app) fake_transactions: FakeTransactionStore,
    pub(in crate::app) fake_transaction_widgets: FakeTransactionWidgets,
    pub(in crate::app) status_autohide: Rc<Cell<bool>>,
    pub(in crate::app) page_copy_buttons: Rc<RefCell<Vec<gtk::Button>>>,
    pub(in crate::app) page_copy_feedback_generation: Rc<Cell<u64>>,
    pub(in crate::app) show_all: Rc<Cell<bool>>,
    pub(in crate::app) show_predictions: Rc<Cell<bool>>,
    #[cfg(not(feature = "flatpak"))]
    pub(in crate::app) online_smart_insights: Rc<Cell<bool>>,
    pub(in crate::app) compare_categories_previous_period: Rc<Cell<bool>>,
    pub(in crate::app) advanced_autofill: Rc<Cell<bool>>,
    pub(in crate::app) advanced_features: Rc<Cell<bool>>,
    pub(in crate::app) remember_mode: Rc<Cell<RememberMode>>,
    pub(in crate::app) auto_clean_config: Rc<Cell<bool>>,
    pub(in crate::app) management_dialog_active: Rc<Cell<bool>>,
    pub(in crate::app) management_search: Rc<RefCell<Option<SearchToggleHandle>>>,
    pub(in crate::app) management_actions: Rc<RefCell<Vec<gtk::gio::SimpleAction>>>,
    pub(in crate::app) config_widgets: Rc<RefCell<Vec<ConfigWidget>>>,
    pub(in crate::app) loading_sensitive_widgets: Rc<RefCell<Vec<LoadingSensitiveWidget>>>,
    pub(in crate::app) hide_canceled_transactions: Rc<Cell<bool>>,
    pub(in crate::app) status_generation: Rc<Cell<u64>>,
    pub(in crate::app) render_generation: Rc<Cell<u64>>,
    pub(in crate::app) render_request_generation: Rc<Cell<u64>>,
    pub(in crate::app) render_loading_generation: Rc<Cell<Option<u64>>>,
    pub(in crate::app) selected_year: Rc<Cell<Option<i32>>>,
    pub(in crate::app) selected_budget_month: Rc<Cell<Option<MonthKey>>>,
    pub(in crate::app) period_user_selected: Rc<Cell<bool>>,
    pub(in crate::app) preferences: Preferences,
    pub(in crate::app) storage_capabilities: Rc<RefCell<data::StorageCapabilities>>,
}

#[derive(Clone)]
pub(in crate::app) struct NavigationEntry {
    pub(super) page_name: String,
    pub(super) search_query: String,
    pub(super) transaction_filter: Option<TransactionFilter>,
    pub(super) search_mode: bool,
}
