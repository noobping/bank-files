use super::*;

pub(super) fn period_controls_box(ui_handles: &UiHandles) -> gtk::Box {
    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    controls.add_css_class("period-controls");
    controls.set_sensitive(!period_controls_are_loading(ui_handles));
    controls
}

pub(super) fn period_controls_are_loading(ui_handles: &UiHandles) -> bool {
    ui_handles.loading_count.get() > 0
}
