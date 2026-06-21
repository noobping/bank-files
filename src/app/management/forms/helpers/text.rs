use super::*;

const RULE_SEARCH_MIN_HEIGHT: i32 = 96;

pub(in crate::app) fn rule_search_text_view(text: &str) -> gtk::TextView {
    let buffer = gtk::TextBuffer::new(None);
    buffer.set_text(text);

    gtk::TextView::builder()
        .buffer(&buffer)
        .wrap_mode(gtk::WrapMode::WordChar)
        .accepts_tab(false)
        .top_margin(6)
        .bottom_margin(6)
        .left_margin(8)
        .right_margin(8)
        .hexpand(true)
        .vexpand(false)
        .build()
}

pub(in crate::app) fn rule_search_text_area(text_view: &gtk::TextView) -> gtk::ScrolledWindow {
    gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .min_content_height(RULE_SEARCH_MIN_HEIGHT)
        .propagate_natural_height(true)
        .child(text_view)
        .build()
}

pub(in crate::app) fn rule_search_text(text_view: &gtk::TextView) -> String {
    let buffer = text_view.buffer();
    let (start, end) = buffer.bounds();
    buffer.text(&start, &end, true).trim().to_string()
}

pub(in crate::app) fn set_rule_search_text(text_view: &gtk::TextView, text: &str) {
    text_view.buffer().set_text(text);
}

pub(in crate::app) fn connect_text_view_summary(text_view: &gtk::TextView, update: &Rc<dyn Fn()>) {
    let update_for_buffer = Rc::clone(update);
    text_view
        .buffer()
        .connect_changed(move |_| update_for_buffer());
}
