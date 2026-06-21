use super::*;

pub(super) fn register_page_actions(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    register_copy_page_action(app, state, ui);
    register_print_page_action(app, state, ui);
    register_export_action(app, state, ui);
}

fn register_copy_page_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_copy_page = Rc::clone(state);
    let ui_for_copy_page = Rc::clone(ui);
    let copy_page_action = gtk::gio::SimpleAction::new("copy-page", None);
    copy_page_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        let snapshot = current_page_snapshot(&state_for_copy_page.borrow(), &ui_for_copy_page);
        ui_for_copy_page.window.clipboard().set_text(&snapshot.text);
        show_page_copy_feedback(&ui_for_copy_page);
    });
    app.add_action(&copy_page_action);
}

fn register_print_page_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_print_page = Rc::clone(state);
    let ui_for_print_page = Rc::clone(ui);
    let print_page_action = gtk::gio::SimpleAction::new("print-page", None);
    print_page_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        let report = current_print_report(&state_for_print_page.borrow(), &ui_for_print_page);
        print_report(&ui_for_print_page, report);
    });
    app.add_action(&print_page_action);
}

fn register_export_action(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let state_for_export = Rc::clone(state);
    let ui_for_export = Rc::clone(ui);
    let export_action = gtk::gio::SimpleAction::new("export-csv", None);
    export_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        let snapshot = current_real_page_snapshot(&state_for_export.borrow(), &ui_for_export);
        export_transactions_from_action(&ui_for_export, action, &snapshot.transactions);
    });
    app.add_action(&export_action);
}
