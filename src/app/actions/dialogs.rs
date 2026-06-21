use super::*;

pub(super) fn register_dialog_actions(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    register_configuration_action(app, window, state, ui);
    register_preferences_action(app, window, state, ui);
    register_management_actions(app, window, state, ui);
    register_popup_actions(app, state, ui);
    register_shortcuts_action(app, window);
    register_about_action(app, window);
    register_quit_action(app);
}

fn register_configuration_action(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let window_for_configuration = window.clone();
    let state_for_configuration = Rc::clone(state);
    let ui_for_configuration = Rc::clone(ui);
    let configuration_action = gtk::gio::SimpleAction::new("configuration", None);
    configuration_action.connect_activate(move |_, _| {
        show_configuration_dialog(
            &window_for_configuration,
            &state_for_configuration,
            &ui_for_configuration,
        );
    });
    app.add_action(&configuration_action);
}

fn register_preferences_action(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let window_for_preferences = window.clone();
    let ui_for_preferences = Rc::clone(ui);
    let preferences_action = gtk::gio::SimpleAction::new("preferences", None);
    let state_for_preferences = Rc::clone(state);
    preferences_action.connect_activate(move |_, _| {
        show_preferences_dialog(
            &window_for_preferences,
            &state_for_preferences,
            &ui_for_preferences,
        );
    });
    app.add_action(&preferences_action);
}

fn register_management_actions(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let manage_rules_action = gtk::gio::SimpleAction::new("manage-rules", None);
    let manage_budgets_action = gtk::gio::SimpleAction::new("manage-budgets", None);
    let manage_aliases_action = gtk::gio::SimpleAction::new("manage-aliases", None);
    *ui.management_actions.borrow_mut() = vec![
        manage_rules_action.clone(),
        manage_budgets_action.clone(),
        manage_aliases_action.clone(),
    ];

    let state_for_rules = Rc::clone(state);
    let ui_for_rules = Rc::clone(ui);
    let window_for_rules = window.clone();
    manage_rules_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        show_management_dialog(&window_for_rules, &state_for_rules, &ui_for_rules, "rules");
    });
    app.add_action(&manage_rules_action);

    let state_for_budgets = Rc::clone(state);
    let ui_for_budgets = Rc::clone(ui);
    let window_for_budgets = window.clone();
    manage_budgets_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        show_management_dialog(
            &window_for_budgets,
            &state_for_budgets,
            &ui_for_budgets,
            "budgets",
        );
    });
    app.add_action(&manage_budgets_action);

    let state_for_aliases = Rc::clone(state);
    let ui_for_aliases = Rc::clone(ui);
    let window_for_aliases = window.clone();
    manage_aliases_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        show_management_dialog(
            &window_for_aliases,
            &state_for_aliases,
            &ui_for_aliases,
            "aliases",
        );
    });
    app.add_action(&manage_aliases_action);
}

fn register_popup_actions(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_fake_transactions = Rc::clone(state);
    let ui_for_fake_transactions = Rc::clone(ui);
    let fake_transactions_action = gtk::gio::SimpleAction::new("fake-transactions", None);
    fake_transactions_action.connect_activate(move |_, _| {
        show_fake_transactions_dialog(&state_for_fake_transactions, &ui_for_fake_transactions);
    });
    app.add_action(&fake_transactions_action);

    let state_for_operation_queue = Rc::clone(state);
    let ui_for_operation_queue = Rc::clone(ui);
    let operation_queue_action = gtk::gio::SimpleAction::new("operation-queue", None);
    operation_queue_action.connect_activate(move |_, _| {
        show_operation_queue_dialog(&state_for_operation_queue, &ui_for_operation_queue);
    });
    app.add_action(&operation_queue_action);
}

fn register_shortcuts_action(app: &adw::Application, window: &adw::ApplicationWindow) {
    let window_for_shortcuts = window.clone();
    let shortcuts_action = gtk::gio::SimpleAction::new("shortcuts", None);
    shortcuts_action.connect_activate(move |_, _| {
        let shortcuts = build_shortcuts_dialog();
        shortcuts.present(Some(&window_for_shortcuts));
    });
    app.add_action(&shortcuts_action);
}

fn register_about_action(app: &adw::Application, window: &adw::ApplicationWindow) {
    let window_for_about = window.clone();
    let about_action = gtk::gio::SimpleAction::new("about", None);
    about_action.connect_activate(move |_, _| {
        let app_name = app_info::display_name();
        let summary = app_info::summary();
        let dialog = ui::about_dialog(ui::AboutDialogDetails {
            application_name: &app_name,
            application_icon: APP_ID,
            developer_name: "Nick",
            version: env!("CARGO_PKG_VERSION"),
            comments: &summary,
            copyright: "Copyright 2026 Nick",
            website: env!("CARGO_PKG_HOMEPAGE"),
            issue_url: "https://github.com/noobping/bank-files/issues",
            license_type: gtk::License::MitX11,
        });
        dialog.present(Some(&window_for_about));
    });
    app.add_action(&about_action);
}

fn register_quit_action(app: &adw::Application) {
    let app_for_quit = app.clone();
    let quit_action = gtk::gio::SimpleAction::new("quit", None);
    quit_action.connect_activate(move |_, _| {
        app_for_quit.quit();
    });
    app.add_action(&quit_action);
}
