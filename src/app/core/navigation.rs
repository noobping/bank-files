use super::session::ACTIVE_SESSION;
use super::types::NavigationEntry;
use super::*;

pub(super) fn connect_navigation_history(ui: &Rc<UiHandles>) {
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

pub(super) fn update_header_navigation_button(ui: &UiHandles) {
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
                .set_tooltip_text(Some(&tr("Open bank files")));
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
