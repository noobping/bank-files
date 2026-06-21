use super::navigation::update_header_navigation_button;
use super::*;

pub(in crate::app) fn begin_background_operation(ui: &UiHandles) {
    let active = ui.loading_count.get();
    let next = active.saturating_add(1);
    ui.loading_count.set(next);
    if active == 0 {
        set_background_loading(ui, true);
    }
}

pub(in crate::app) fn finish_background_operation(ui: &UiHandles) {
    let active = ui.loading_count.get();
    if active <= 1 {
        ui.loading_count.set(0);
        set_background_loading(ui, false);
    } else {
        let next = active - 1;
        ui.loading_count.set(next);
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
