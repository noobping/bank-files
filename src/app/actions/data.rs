use super::*;

pub(super) fn register_data_actions(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    import_button: gtk::Button,
) {
    register_import_action(app, window, state, ui, import_button);
    register_reload_action(app, state, ui);
    register_reload_all_action(app, state, ui);
    register_clear_cache_action(app, state, ui);
}

fn register_import_action(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    import_button: gtk::Button,
) {
    let state_for_import = Rc::clone(state);
    let ui_for_import = Rc::clone(ui);
    let window_for_import = window.clone();
    let import_action = gtk::gio::SimpleAction::new("import-csv", None);
    import_action.connect_activate(move |action, _| {
        if !action.is_enabled() || ui_for_import.loading_count.get() > 0 {
            return;
        }
        action.set_enabled(false);
        show_status(&ui_for_import, "Opening the file portal for bank files...");

        let action_for_import = action.clone();
        let state_for_import = Rc::clone(&state_for_import);
        let ui_for_import = Rc::clone(&ui_for_import);
        let window_for_import = window_for_import.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            let handles = rfd::AsyncFileDialog::new()
                .set_title(tr("Choose one or more bank files"))
                .add_filter(tr("Bank files"), crate::data::TRANSACTION_IMPORT_EXTENSIONS)
                .pick_files()
                .await;

            let Some(handles) = handles.filter(|handles| !handles.is_empty()) else {
                action_for_import.set_enabled(true);
                show_status(&ui_for_import, "Bank file opening canceled.");
                return;
            };
            let files = handles
                .into_iter()
                .map(|handle| handle.path().to_path_buf())
                .collect::<Vec<_>>();

            show_status(&ui_for_import, "Opening bank files...");
            open_paths_in_background(
                files,
                Rc::clone(&state_for_import),
                Rc::clone(&ui_for_import),
            )
            .await;
            action_for_import.set_enabled(true);
            window_for_import.present();
        });
    });
    app.add_action(&import_action);
    import_button.set_action_name(Some("app.import-csv"));
}

fn register_reload_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_reload = Rc::clone(state);
    let ui_for_reload = Rc::clone(ui);
    let reload_action = gtk::gio::SimpleAction::new("reload", None);
    reload_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        reload_state(&state_for_reload, &ui_for_reload);
    });
    app.add_action(&reload_action);
}

fn register_reload_all_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_reload_all = Rc::clone(state);
    let ui_for_reload_all = Rc::clone(ui);
    let reload_all_action = gtk::gio::SimpleAction::new("reload-all", None);
    reload_all_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        reload_state_with_scope(
            &state_for_reload_all,
            &ui_for_reload_all,
            TransactionLoadScope::All,
            "Reloading all bank data...",
            tr("All bank data reloaded."),
            "Reload error: {error}",
            Vec::new(),
        );
    });
    app.add_action(&reload_all_action);
}

fn register_clear_cache_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_clear_cache = Rc::clone(state);
    let ui_for_clear_cache = Rc::clone(ui);
    let clear_cache_action = gtk::gio::SimpleAction::new("clear-cache-and-reload", None);
    clear_cache_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        clear_cache_and_reload_state(&state_for_clear_cache, &ui_for_clear_cache);
    });
    app.add_action(&clear_cache_action);
}
