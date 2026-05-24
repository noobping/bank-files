use super::navigation::connect_navigation_history;
use super::preference_sync::connect_preference_sync;
use super::session::{ActiveSession, ACTIVE_SESSION};
use super::startup::{apply_transaction_search, StartupRequest};
use super::*;

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

pub(super) fn build_ui_with_startup_request(
    app: &adw::Application,
    startup_request: StartupRequest,
) {
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
