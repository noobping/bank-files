use super::*;

pub fn run() {
    i18n::init();
    register_icon_resources();

    let pending_startup_request = Rc::new(RefCell::new(StartupRequest::default()));
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(
            gtk::gio::ApplicationFlags::HANDLES_OPEN
                | gtk::gio::ApplicationFlags::HANDLES_COMMAND_LINE,
        )
        .build();

    app.connect_startup(|_| {
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::Default);
        ui::install_css();
        add_icon_resource_path();
    });
    let request_for_activate = Rc::clone(&pending_startup_request);
    app.connect_activate(move |app| {
        let request = {
            let mut pending = request_for_activate.borrow_mut();
            std::mem::take(&mut *pending)
        };
        build_ui_with_startup_request(app, request);
    });
    let request_for_command_line = Rc::clone(&pending_startup_request);
    app.connect_command_line(move |app, command_line| {
        let request = startup_request_from_args(&command_line.arguments());
        if request.is_empty() {
            app.activate();
        } else if !apply_startup_request_to_active_session(request.clone()) {
            *request_for_command_line.borrow_mut() = request;
            app.activate();
        }
        0.into()
    });
    app.connect_open(open_files);
    app.connect_shutdown(updater::shutdown);
    app.run();
}

fn register_icon_resources() {
    if let Err(err) = crate::resources::register() {
        eprintln!("Failed to register icon resources: {err}");
    }
}

fn add_icon_resource_path() {
    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };

    crate::resources::add_icon_theme_path(&display);
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct StartupRequest {
    opened_uris: Vec<String>,
    transaction_search: Option<String>,
}

impl StartupRequest {
    fn is_empty(&self) -> bool {
        self.opened_uris.is_empty()
            && self
                .transaction_search
                .as_ref()
                .map(|query| query.trim().is_empty())
                .unwrap_or(true)
    }
}

fn startup_request_from_args(args: &[std::ffi::OsString]) -> StartupRequest {
    let mut request = StartupRequest::default();
    let mut index = 1;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--transaction-search" {
            if let Some(query) = args.get(index + 1) {
                request.transaction_search = Some(query.to_string_lossy().trim().to_string());
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }

        if arg == "--" {
            for file_arg in &args[index + 1..] {
                request.opened_uris.push(command_line_file_uri(file_arg));
            }
            break;
        }

        if arg.to_string_lossy().starts_with('-') {
            index += 1;
            continue;
        }

        request.opened_uris.push(command_line_file_uri(arg));
        index += 1;
    }

    request
}

fn command_line_file_uri(arg: &std::ffi::OsStr) -> String {
    gtk::gio::File::for_commandline_arg(arg).uri().to_string()
}

fn apply_startup_request_to_active_session(request: StartupRequest) -> bool {
    ACTIVE_SESSION.with(|active| {
        let Some(session) = active.borrow().clone() else {
            return false;
        };

        if !request.opened_uris.is_empty() {
            import_uris_into_session(
                request.opened_uris,
                Rc::clone(&session.state),
                Rc::clone(&session.ui),
            );
        }
        if let Some(query) = request.transaction_search {
            apply_transaction_search(&session.state, &session.ui, &query);
        }
        true
    })
}

fn apply_transaction_search(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>, query: &str) {
    let query = query.trim();
    if query.is_empty() {
        ui.window.present();
        return;
    }

    show_transaction_search(state, ui, query, TransactionFilter::from_query(query));
    ui.window.present();
}

pub(in crate::app) fn tr(message: &str) -> String {
    gettext(message)
}

pub(in crate::app) fn trf(message: &str, replacements: &[(&str, String)]) -> String {
    i18n::format(message, replacements)
}

#[derive(Clone)]
pub(in crate::app) struct LoadingSensitiveWidget {
    widget: gtk::Widget,
    base_sensitive: bool,
    was_rooted: Rc<Cell<bool>>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) enum ActionAvailability {
    Available,
    Hidden,
    Disabled(String),
}

pub(in crate::app) fn write_action_available(
    writable: bool,
    advanced_features: bool,
    reason: &str,
) -> ActionAvailability {
    if writable {
        ActionAvailability::Available
    } else if advanced_features {
        ActionAvailability::Disabled(reason.to_string())
    } else {
        ActionAvailability::Hidden
    }
}

pub(in crate::app) fn apply_action_availability<W: IsA<gtk::Widget>>(
    widget: &W,
    availability: &ActionAvailability,
) {
    let widget = widget.as_ref();
    match availability {
        ActionAvailability::Available => {
            widget.set_visible(true);
            widget.set_sensitive(true);
            widget.set_tooltip_text(None);
        }
        ActionAvailability::Hidden => {
            widget.set_visible(false);
            widget.set_sensitive(false);
        }
        ActionAvailability::Disabled(reason) => {
            widget.set_visible(true);
            widget.set_sensitive(false);
            widget.set_tooltip_text(Some(&tr(reason)));
        }
    }
}

pub(in crate::app) fn data_write_availability(ui: &UiHandles) -> ActionAvailability {
    let capabilities = ui.storage_capabilities.borrow();
    write_action_available(
        capabilities.data_writable,
        ui.advanced_features.get(),
        capabilities.data_write_reason(),
    )
}

pub(in crate::app) fn config_write_availability(ui: &UiHandles) -> ActionAvailability {
    let capabilities = ui.storage_capabilities.borrow();
    write_action_available(
        capabilities.config_writable,
        ui.advanced_features.get(),
        capabilities.config_write_reason(),
    )
}

pub(in crate::app) fn set_storage_capabilities(
    ui: &Rc<UiHandles>,
    capabilities: data::StorageCapabilities,
) {
    *ui.storage_capabilities.borrow_mut() = capabilities;
    refresh_write_actions(ui.as_ref());
    update_header_navigation_button(ui.as_ref());
}

pub(in crate::app) fn refresh_write_actions(ui: &UiHandles) {
    let capabilities = ui.storage_capabilities.borrow().clone();
    let not_loading = loading_sensitive_items_enabled(ui.loading_count.get());
    let idle = not_loading && !ui.management_dialog_active.get();
    set_app_action_enabled(ui, "reload", not_loading);
    set_app_action_enabled(ui, "reload-all", not_loading);
    set_app_action_enabled(ui, "clear-cache-and-reload", not_loading);
    set_app_action_enabled(ui, "copy-page", not_loading);
    set_app_action_enabled(ui, "print-page", not_loading);
    set_app_action_enabled(ui, "export-csv", not_loading);
    set_app_action_enabled(ui, "import-csv", idle);
    set_app_action_enabled(ui, "configuration", capabilities.config_writable && idle);
    set_app_action_enabled(
        ui,
        "manage-rules",
        capabilities.config_writable && ui.advanced_features.get() && idle,
    );
    set_app_action_enabled(ui, "manage-budgets", capabilities.config_writable && idle);
    set_app_action_enabled(ui, "manage-aliases", capabilities.config_writable && idle);
    set_app_action_enabled(ui, "preferences", ui.preferences.any_writable());
    set_app_action_enabled(
        ui,
        "remember-mode",
        ui.preferences.action_is_writable("remember-mode") && not_loading,
    );
    update_config_action_widgets(ui);
    update_loading_sensitive_widgets(ui);
}

fn loading_sensitive_items_enabled(loading_count: u32) -> bool {
    loading_count == 0
}

pub(in crate::app) fn register_loading_sensitive_widget<W: IsA<gtk::Widget>>(
    ui: &Rc<UiHandles>,
    widget: &W,
) {
    let widget = widget.clone().upcast::<gtk::Widget>();
    let base_sensitive = widget.is_sensitive();
    let was_rooted = widget.root().is_some();
    widget.set_sensitive(base_sensitive && loading_sensitive_items_enabled(ui.loading_count.get()));
    ui.loading_sensitive_widgets
        .borrow_mut()
        .push(LoadingSensitiveWidget {
            widget,
            base_sensitive,
            was_rooted: Rc::new(Cell::new(was_rooted)),
        });
}

fn update_loading_sensitive_widgets(ui: &UiHandles) {
    let sensitive = loading_sensitive_items_enabled(ui.loading_count.get());
    let mut widgets = ui.loading_sensitive_widgets.borrow_mut();
    widgets.retain(loading_sensitive_widget_should_remain_registered);
    for item in widgets.iter() {
        item.widget.set_sensitive(item.base_sensitive && sensitive);
    }
}

fn loading_sensitive_widget_should_remain_registered(item: &LoadingSensitiveWidget) -> bool {
    let rooted = item.widget.root().is_some();
    if rooted {
        item.was_rooted.set(true);
    }
    widget_registration_is_live(rooted, item.was_rooted.get())
}

fn widget_registration_is_live(rooted: bool, was_rooted: bool) -> bool {
    rooted || !was_rooted
}

fn set_app_action_enabled(ui: &UiHandles, name: &str, enabled: bool) {
    if let Some(action) = ui
        .window
        .application()
        .and_then(|app| app.lookup_action(name))
        .and_then(|action| action.downcast::<gtk::gio::SimpleAction>().ok())
    {
        action.set_enabled(enabled);
    }
}

#[derive(Clone)]
pub(in crate::app) struct ActiveSession {
    pub(in crate::app) state: Rc<RefCell<AppData>>,
    pub(in crate::app) ui: Rc<UiHandles>,
}

thread_local! {
    pub(in crate::app) static ACTIVE_SESSION: RefCell<Option<ActiveSession>> = const { RefCell::new(None) };
}

fn connect_navigation_history(ui: &Rc<UiHandles>) {
    update_header_navigation_button(ui.as_ref());
    let ui_for_stack = Rc::clone(ui);
    ui.stack.connect_visible_child_name_notify(move |stack| {
        let Some(next_page) = stack.visible_child_name().map(|name| name.to_string()) else {
            return;
        };

        let mut current_page = ui_for_stack.navigation_current_page.borrow_mut();
        if *current_page == next_page {
            return;
        }

        if ui_for_stack.navigation_is_restoring.replace(false) {
            *current_page = next_page;
            update_header_navigation_button(ui_for_stack.as_ref());
            return;
        }

        let entry = NavigationEntry {
            page_name: current_page.clone(),
            search_query: ui_for_stack.search_query.borrow().clone(),
            transaction_filter: ui_for_stack.active_transaction_filter.borrow().clone(),
            search_mode: ui_for_stack.search_bar.is_search_mode(),
        };
        let mut history = ui_for_stack.navigation_history.borrow_mut();
        history.push(entry);
        if history.len() > 50 {
            history.remove(0);
        }
        *current_page = next_page;
        drop(history);
        update_header_navigation_button(ui_for_stack.as_ref());
        render_active_session_soon();
    });
}

fn render_active_session_soon() {
    gtk::glib::idle_add_local_once(|| {
        ACTIVE_SESSION.with(|active| {
            if let Some(session) = active.borrow().clone() {
                render_views(&session.state.borrow(), &session.ui, &session.state);
            }
        });
    });
}

pub(in crate::app) fn navigate_back(ui: &Rc<UiHandles>, state: &Rc<RefCell<AppData>>) {
    let Some(entry) = ui.navigation_history.borrow_mut().pop() else {
        update_header_navigation_button(ui.as_ref());
        return;
    };

    ui.navigation_is_restoring.set(true);
    ui.stack.set_visible_child_name(&entry.page_name);
    *ui.search_query.borrow_mut() = entry.search_query.clone();
    *ui.active_transaction_filter.borrow_mut() = entry.transaction_filter;
    ui.search_bar
        .set_search_mode(entry.search_mode || !entry.search_query.is_empty());
    if ui.search_entry.text().as_str() != entry.search_query.as_str() {
        ui.search_entry.set_text(&entry.search_query);
    }
    render_views(&state.borrow(), ui, state);
    update_header_navigation_button(ui.as_ref());
}

fn update_header_navigation_button(ui: &UiHandles) {
    let can_go_back = !ui.navigation_history.borrow().is_empty();
    ui.back_button.set_visible(can_go_back);
    if can_go_back {
        ui.import_button.set_visible(false);
        return;
    }

    match data_write_availability(ui) {
        ActionAvailability::Available => {
            ui.import_button.set_visible(true);
            ui.import_button.set_sensitive(ui.loading_count.get() == 0);
            ui.import_button
                .set_tooltip_text(Some(&tr("Open CSV files")));
        }
        ActionAvailability::Hidden => {
            ui.import_button.set_visible(false);
            ui.import_button.set_sensitive(false);
        }
        ActionAvailability::Disabled(reason) => {
            ui.import_button.set_visible(true);
            ui.import_button.set_sensitive(false);
            ui.import_button.set_tooltip_text(Some(&tr(&reason)));
        }
    }
}

pub(in crate::app) fn begin_background_operation(ui: &UiHandles) {
    let active = ui.loading_count.get();
    let next = active.saturating_add(1);
    ui.loading_count.set(next);
    show_verbose_status(ui, format!("background operation started; active={next}"));
    if active == 0 {
        set_background_loading(ui, true);
    }
}

pub(in crate::app) fn finish_background_operation(ui: &UiHandles) {
    let active = ui.loading_count.get();
    if active <= 1 {
        ui.loading_count.set(0);
        show_verbose_status(ui, "background operation finished; active=0");
        set_background_loading(ui, false);
    } else {
        let next = active - 1;
        ui.loading_count.set(next);
        show_verbose_status(ui, format!("background operation finished; active={next}"));
    }
}

fn set_background_loading(ui: &UiHandles, loading: bool) {
    ui.status_icon.set_visible(!loading);
    ui.status_loading_spinner.set_visible(loading);
    set_period_controls_enabled(ui, !loading);
    if loading {
        ui.status_bar.set_visible(true);
    } else {
        schedule_status_autohide_after_loading(ui);
    }
    update_header_navigation_button(ui);
    refresh_write_actions(ui);
    refresh_active_operation_queue_ui();
}

fn set_period_controls_enabled(ui: &UiHandles, enabled: bool) {
    set_period_controls_enabled_in(&ui.overview.clone().upcast::<gtk::Widget>(), enabled);
    set_period_controls_enabled_in(&ui.categories.clone().upcast::<gtk::Widget>(), enabled);
    set_period_controls_enabled_in(&ui.transactions.clone().upcast::<gtk::Widget>(), enabled);
    set_period_controls_enabled_in(&ui.debug.clone().upcast::<gtk::Widget>(), enabled);
}

fn set_period_controls_enabled_in(widget: &gtk::Widget, enabled: bool) {
    if widget.has_css_class("period-controls") {
        widget.set_sensitive(enabled);
    }

    let mut child = widget.first_child();
    while let Some(current) = child {
        child = current.next_sibling();
        set_period_controls_enabled_in(&current, enabled);
    }
}

#[derive(Clone)]
pub(in crate::app) struct NavigationEntry {
    page_name: String,
    search_query: String,
    transaction_filter: Option<TransactionFilter>,
    search_mode: bool,
}

pub(in crate::app) const CATEGORY_PREVIEW_LIMIT: usize = 5;
pub(in crate::app) const SEARCH_CATEGORY_PREVIEW_LIMIT: usize = 6;

pub(in crate::app) fn comparison_mode(ui: &UiHandles) -> ComparisonMode {
    if ui.compare_categories_previous_period.get() {
        ComparisonMode::WithPrevious
    } else {
        ComparisonMode::CurrentOnly
    }
}

pub(in crate::app) fn current_transaction_load_scope(
    data: &AppData,
    ui: &UiHandles,
) -> TransactionLoadScope {
    match ui.stack.visible_child_name().as_deref() {
        Some("debug") => default_year_load_scope(data, ui),
        Some("transactions") => transaction_page_load_scope(data, ui),
        Some("categories") => TransactionLoadScope::for_month(
            ui.selected_budget_month.get().or(data.default_month),
            comparison_mode(ui),
        ),
        _ => default_year_load_scope(data, ui),
    }
}

fn default_year_load_scope(data: &AppData, ui: &UiHandles) -> TransactionLoadScope {
    TransactionLoadScope::for_year(
        selected_year_for_load_scope(data, ui.selected_year.get()),
        comparison_mode(ui),
    )
}

fn selected_year_for_load_scope(data: &AppData, selected_year: Option<i32>) -> Option<i32> {
    selected_year.or_else(|| data.default_month.map(|month| month.year))
}

fn transaction_page_load_scope(data: &AppData, ui: &UiHandles) -> TransactionLoadScope {
    let selected_year = ui
        .selected_year
        .get()
        .or_else(|| data.default_month.map(|month| month.year));
    let Some(filter) = ui.active_transaction_filter.borrow().clone() else {
        if ui.search_query.borrow().trim().is_empty() {
            return TransactionLoadScope::Year(selected_year);
        }
        return TransactionLoadScope::All;
    };
    match filter {
        TransactionFilter::CategoryForYear { year, .. } => TransactionLoadScope::Year(Some(year)),
        TransactionFilter::Scoped {
            year: Some(year),
            month: None,
            ..
        } => TransactionLoadScope::Year(Some(year)),
        TransactionFilter::Scoped {
            month: Some(month), ..
        } => TransactionLoadScope::Month(Some(month)),
        TransactionFilter::Scoped {
            year: None,
            month: None,
            ..
        } => TransactionLoadScope::Year(selected_year),
        TransactionFilter::All
        | TransactionFilter::UnconfiguredBudgets
        | TransactionFilter::OtherCategories
        | TransactionFilter::Pattern(_) => TransactionLoadScope::All,
    }
}

pub fn build_ui(app: &adw::Application) {
    build_ui_with_startup_request(app, StartupRequest::default());
}

pub(in crate::app) fn build_ui_with_opened_uris(app: &adw::Application, opened_uris: Vec<String>) {
    build_ui_with_startup_request(
        app,
        StartupRequest {
            opened_uris,
            transaction_search: None,
        },
    );
}

fn build_ui_with_startup_request(app: &adw::Application, startup_request: StartupRequest) {
    let preferences = Preferences::new();
    let initial_storage_capabilities = data::current_storage_capabilities();
    let initial_dedupe_mode = DedupeMode::from_enabled(preferences.dedupe_enabled());
    let initial = AppData {
        dedupe_mode: initial_dedupe_mode,
        remember_mode: preferences.remember_mode(),
        ..Default::default()
    };
    let state = Rc::new(RefCell::new(initial));

    let window_title = app_info::display_name();
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(&window_title)
        .default_width(preferences.window_width())
        .default_height(preferences.window_height())
        .build();
    if preferences.window_maximized() {
        window.maximize();
    }
    ui::install_css();

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let stack = adw::ViewStack::new();
    stack.set_vexpand(true);
    stack.set_hexpand(true);

    let overview = ui::page_box();
    let categories = ui::page_box();
    let transactions = ui::page_box();
    let debug = ui::page_box();

    let overview_scroll = ui::scroll(&overview);
    let categories_scroll = ui::scroll(&categories);
    let transactions_scroll = ui::scroll(&transactions);
    let debug_scroll = ui::scroll(&debug);

    stack
        .add_titled(&overview_scroll, Some("overview"), &tr("Overview"))
        .set_icon_name(Some("view-grid-symbolic"));
    stack
        .add_titled(&categories_scroll, Some("categories"), &tr("Budget"))
        .set_icon_name(Some("view-list-symbolic"));
    stack
        .add_titled(
            &transactions_scroll,
            Some("transactions"),
            &tr("Transactions"),
        )
        .set_icon_name(Some("view-list-symbolic"));
    stack
        .add_titled(&debug_scroll, Some("debug"), &tr("Diagnostics"))
        .set_icon_name(Some("dialog-information-symbolic"));
    stack.set_visible_child_name(&preferences.active_tab());

    let header = adw::HeaderBar::new();
    let switcher = adw::ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .build();
    let mobile_header_title = adw::WindowTitle::new(&tr("Overview"), "");
    mobile_header_title.set_visible(false);
    let header_title = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    header_title.append(&switcher);
    header_title.append(&mobile_header_title);
    header.set_title_widget(Some(&header_title));

    let import_button = ui::icon_button("document-open-symbolic", "Open CSV files");
    import_button.add_css_class("flat");
    let back_button = ui::icon_button("go-previous-symbolic", "Back");
    back_button.add_css_class("flat");
    back_button.set_action_name(Some("app.go-back"));
    back_button.set_visible(false);
    let header_start = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    header_start.append(&import_button);
    header_start.append(&back_button);
    header.pack_start(&header_start);

    let menu_button = build_menu(
        &state.borrow(),
        preferences.advanced_features(),
        &initial_storage_capabilities,
        &preferences,
    );
    menu_button.add_css_class("flat");
    header.pack_end(&menu_button);
    root.append(&header);

    let search_entry = gtk::SearchEntry::builder()
        .placeholder_text(tr("Search this page"))
        .hexpand(true)
        .build();
    let search_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    search_box.set_margin_top(6);
    search_box.set_margin_bottom(6);
    search_box.set_margin_start(12);
    search_box.set_margin_end(12);
    search_box.append(&search_entry);
    let search_bar = gtk::SearchBar::builder()
        .child(&search_box)
        .show_close_button(true)
        .search_mode_enabled(false)
        .build();
    search_bar.connect_entry(&search_entry);
    search_bar.set_key_capture_widget(Some(&window));
    root.append(&search_bar);

    root.append(&stack);

    let switcher_bar = adw::ViewSwitcherBar::builder()
        .stack(&stack)
        .reveal(false)
        .build();
    root.append(&switcher_bar);
    add_responsive_switcher(&window, &switcher, &switcher_bar, &mobile_header_title);
    add_responsive_page_margins(
        &window,
        &switcher,
        &switcher_bar,
        &mobile_header_title,
        &[&overview, &categories, &transactions, &debug],
    );

    let status_bar = build_status_bar();
    let operation_queue = OperationQueue::new();
    let operation_queue_widgets = build_operation_queue_widgets();
    let fake_transactions = FakeTransactionStore::new();
    let fake_transaction_widgets = build_fake_transaction_widgets();
    status_bar
        .action_group
        .prepend(&operation_queue_widgets.button);
    status_bar
        .action_group
        .prepend(&fake_transaction_widgets.button);
    root.append(&status_bar.container);

    let ui = Rc::new(UiHandles {
        window: window.clone(),
        stack: stack.clone(),
        overview,
        categories,
        transactions,
        debug,
        search_bar,
        search_entry,
        mobile_header_title: mobile_header_title.clone(),
        search_query: Rc::new(RefCell::new(String::new())),
        active_transaction_filter: Rc::new(RefCell::new(None)),
        import_button: import_button.clone(),
        loading_count: Rc::new(Cell::new(0)),
        back_button: back_button.clone(),
        menu_button: menu_button.clone(),
        navigation_history: Rc::new(RefCell::new(Vec::new())),
        navigation_current_page: Rc::new(RefCell::new(
            stack
                .visible_child_name()
                .map(|name| name.to_string())
                .unwrap_or_else(|| "overview".to_string()),
        )),
        navigation_is_restoring: Rc::new(Cell::new(false)),
        status_bar: status_bar.container.clone(),
        status_icon: status_bar.icon.clone(),
        status_loading_spinner: status_bar.spinner.clone(),
        status: status_bar.label.clone(),
        status_history: Rc::new(RefCell::new(Vec::new())),
        operation_queue,
        operation_queue_widgets,
        fake_transactions,
        fake_transaction_widgets,
        status_autohide: Rc::new(Cell::new(preferences.autohide_status_bar())),
        page_copy_buttons: Rc::new(RefCell::new(Vec::new())),
        page_copy_feedback_generation: Rc::new(Cell::new(0)),
        show_all: Rc::new(Cell::new(preferences.show_all())),
        show_predictions: Rc::new(Cell::new(preferences.show_predictions())),
        #[cfg(not(feature = "flatpak"))]
        online_smart_insights: Rc::new(Cell::new(preferences.online_smart_insights())),
        compare_categories_previous_period: Rc::new(Cell::new(
            preferences.compare_categories_previous_period(),
        )),
        advanced_autofill: Rc::new(Cell::new(preferences.advanced_autofill())),
        advanced_features: Rc::new(Cell::new(preferences.advanced_features())),
        remember_mode: Rc::new(Cell::new(preferences.remember_mode())),
        auto_clean_config: Rc::new(Cell::new(preferences.auto_clean_config())),
        management_dialog_active: Rc::new(Cell::new(false)),
        management_actions: Rc::new(RefCell::new(Vec::new())),
        config_widgets: Rc::new(RefCell::new(Vec::new())),
        loading_sensitive_widgets: Rc::new(RefCell::new(Vec::new())),
        hide_canceled_transactions: Rc::new(Cell::new(preferences.hide_canceled_transactions())),
        status_generation: Rc::new(Cell::new(0)),
        render_generation: Rc::new(Cell::new(0)),
        render_request_generation: Rc::new(Cell::new(0)),
        render_loading_generation: Rc::new(Cell::new(None)),
        selected_year: Rc::new(Cell::new(preferences.selected_year())),
        selected_budget_month: Rc::new(Cell::new(preferences.selected_budget_month())),
        period_user_selected: Rc::new(Cell::new(false)),
        preferences: preferences.clone(),
        storage_capabilities: Rc::new(RefCell::new(initial_storage_capabilities)),
    });
    status_bar.search_preset_button.set_visible(true);
    register_loading_sensitive_widget(&ui, &status_bar.search_preset_button);
    register_loading_sensitive_widget(&ui, &status_bar.page_actions_button);
    connect_navigation_history(&ui);
    connect_preference_sync(&ui);

    show_status(
        &ui,
        "Choose bank files or drop them onto the window to review spending, budgets, and trends.",
    );

    connect_status_actions(app, &ui, status_bar.history_button, status_bar.hide_button);
    connect_operation_queue(&state, &ui);
    connect_fake_transactions(&state, &ui);
    connect_actions(
        app,
        &window,
        &state,
        &ui,
        import_button,
        menu_button.clone(),
    );
    connect_drop_target(&root, &state, &ui);
    render_loading_placeholder(ui.as_ref());

    window.set_content(Some(&root));
    window.present();
    let session = ActiveSession {
        state: Rc::clone(&state),
        ui: Rc::clone(&ui),
    };
    ACTIVE_SESSION.with(|active| {
        active.replace(Some(session));
    });
    updater::after_window_presented(app, &window);

    if let Some(query) = startup_request.transaction_search.as_ref() {
        apply_transaction_search(&state, &ui, query);
    }

    let state_for_startup = Rc::clone(&state);
    let ui_for_startup = Rc::clone(&ui);
    let opened_uris = startup_request.opened_uris;
    gtk::glib::idle_add_local_once(move || {
        if opened_uris.is_empty() {
            reload_state_with_status(
                &state_for_startup,
                &ui_for_startup,
                "Loading saved CSV files...",
                tr("Saved CSV files loaded."),
                "Startup error: {error}",
                Vec::new(),
            );
        } else {
            import_uris_into_session(opened_uris, state_for_startup, ui_for_startup);
        }
    });
}

fn connect_preference_sync(ui: &Rc<UiHandles>) {
    let preferences_for_tab = ui.preferences.clone();
    ui.stack.connect_visible_child_name_notify(move |stack| {
        if let Some(name) = stack.visible_child_name() {
            preferences_for_tab.set_active_tab(name.as_str());
        }
    });

    let preferences_for_window = ui.preferences.clone();
    ui.window.connect_close_request(move |window| {
        let maximized = window.is_maximized();
        let (width, height) = if maximized {
            (
                preferences_for_window.window_width(),
                preferences_for_window.window_height(),
            )
        } else {
            (window.width(), window.height())
        };
        preferences_for_window.set_window_state(width, height, maximized);
        gtk::glib::Propagation::Proceed
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_year_for_load_scope_prefers_user_selection() {
        let data = AppData {
            default_month: Some(MonthKey::new(2024, 12)),
            ..AppData::default()
        };

        assert_eq!(selected_year_for_load_scope(&data, Some(2026)), Some(2026));
    }

    #[test]
    fn selected_year_for_load_scope_falls_back_to_default_month_year() {
        let data = AppData {
            default_month: Some(MonthKey::new(2024, 12)),
            ..AppData::default()
        };

        assert_eq!(selected_year_for_load_scope(&data, None), Some(2024));
    }
    #[test]
    fn write_action_availability_follows_storage_and_mode() {
        assert_eq!(
            write_action_available(true, false, "Locked"),
            ActionAvailability::Available
        );
        assert_eq!(
            write_action_available(false, false, "Locked"),
            ActionAvailability::Hidden
        );
        assert_eq!(
            write_action_available(false, true, "Locked"),
            ActionAvailability::Disabled("Locked".to_string())
        );
    }

    #[test]
    fn loading_sensitive_items_are_enabled_only_when_idle() {
        assert!(loading_sensitive_items_enabled(0));
        assert!(!loading_sensitive_items_enabled(1));
        assert!(!loading_sensitive_items_enabled(3));
    }

    #[test]
    fn unrooted_loading_widgets_stay_registered_until_first_root() {
        assert!(widget_registration_is_live(false, false));
        assert!(widget_registration_is_live(true, false));
        assert!(!widget_registration_is_live(false, true));
    }

    #[test]
    fn startup_request_parses_transaction_search() {
        let args = vec![
            std::ffi::OsString::from("bank-files"),
            std::ffi::OsString::from("--transaction-search"),
            std::ffi::OsString::from("rent may"),
        ];

        assert_eq!(
            startup_request_from_args(&args),
            StartupRequest {
                opened_uris: Vec::new(),
                transaction_search: Some("rent may".to_string()),
            }
        );
    }

    #[test]
    fn startup_request_keeps_file_open_arguments() {
        let args = vec![
            std::ffi::OsString::from("bank-files"),
            std::ffi::OsString::from("--"),
            std::ffi::OsString::from("/tmp/bank.csv"),
        ];
        let request = startup_request_from_args(&args);

        assert_eq!(request.transaction_search, None);
        assert_eq!(request.opened_uris.len(), 1);
        assert!(request.opened_uris[0].ends_with("/tmp/bank.csv"));
    }
}
