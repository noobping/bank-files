use super::*;

const ACTION_DIALOG_MAX_HEIGHT: i32 = 620;

pub fn month_label(month: MonthKey) -> String {
    format!("{} {}", month_name(month.month), month.year)
}

pub fn month_name(month: u32) -> String {
    let name = match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => return month.to_string(),
    };
    gettext(name)
}

pub fn icon_button(icon_name: &str, tooltip: &str) -> gtk::Button {
    gtk::Button::builder()
        .icon_name(icon_name)
        .tooltip_text(gettext(tooltip))
        .build()
}

pub fn set_button_icon(button: &gtk::Button, icon_name: &str) {
    button.set_icon_name(icon_name);
}

pub fn flat_custom_button(tooltip: &str, child: &impl IsA<gtk::Widget>) -> gtk::Button {
    let button = gtk::Button::builder()
        .tooltip_text(gettext(tooltip))
        .child(child)
        .build();
    button.add_css_class("flat");
    button
}

pub fn primary_text_icon_button(icon_name: &str, label: &str, tooltip: &str) -> gtk::Button {
    let button = plain_text_icon_button(icon_name, label, tooltip);
    button.add_css_class("suggested-action");
    button
}

pub fn plain_text_icon_button(icon_name: &str, label: &str, tooltip: &str) -> gtk::Button {
    let content = adw::ButtonContent::builder()
        .icon_name(icon_name)
        .label(gettext(label))
        .build();

    gtk::Button::builder()
        .tooltip_text(gettext(tooltip))
        .child(&content)
        .build()
}

pub fn loading_spinner() -> adw::Spinner {
    adw::Spinner::new()
}

pub fn action_dialog_header() -> adw::HeaderBar {
    let header = adw::HeaderBar::new();
    header.set_show_start_title_buttons(false);
    header.set_show_end_title_buttons(true);
    header
}

pub fn cancelable_dialog_header(title: &str, subtitle: &str) -> adw::HeaderBar {
    let header = action_dialog_header();
    header.set_title_widget(Some(&adw::WindowTitle::new(
        &gettext(title),
        &gettext(subtitle),
    )));
    header
}

pub fn dialog_toolbar_view(
    header: &impl IsA<gtk::Widget>,
    content: &impl IsA<gtk::Widget>,
) -> adw::ToolbarView {
    let view = adw::ToolbarView::new();
    view.add_top_bar(header);
    view.set_content(Some(content));
    view
}

pub fn wrapped_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    label
}

pub fn selectable_wrapped_label(text: &str) -> gtk::Label {
    let label = wrapped_label(text);
    label.set_selectable(true);
    label
}

pub fn scroll(child: &impl IsA<gtk::Widget>) -> gtk::ScrolledWindow {
    let clamp = adw::Clamp::builder()
        .maximum_size(1080)
        .tightening_threshold(640)
        .child(child)
        .build();
    gtk::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&clamp)
        .build()
}

pub fn action_dialog_scroll(child: &impl IsA<gtk::Widget>) -> gtk::ScrolledWindow {
    action_dialog_scroll_with_min(child, 0)
}

pub fn action_dialog_scroll_with_min(
    child: &impl IsA<gtk::Widget>,
    min_content_height: i32,
) -> gtk::ScrolledWindow {
    action_dialog_scroll_with_limits(child, min_content_height, ACTION_DIALOG_MAX_HEIGHT)
}

pub fn action_dialog_scroll_with_limits(
    child: &impl IsA<gtk::Widget>,
    min_content_height: i32,
    max_content_height: i32,
) -> gtk::ScrolledWindow {
    let min_content_height = min_content_height.max(0);
    let max_content_height = max_content_height.max(min_content_height).max(1);
    let clamp = adw::Clamp::builder()
        .maximum_size(1080)
        .tightening_threshold(640)
        .child(child)
        .build();
    gtk::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(false)
        .min_content_height(min_content_height)
        .max_content_height(max_content_height)
        .propagate_natural_height(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&clamp)
        .build()
}

pub fn linked_button_group() -> gtk::Box {
    let group = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    group.add_css_class("linked");
    group
}

pub fn text_list_row(text: &str) -> adw::ActionRow {
    let row = adw::ActionRow::builder().title(text).build();
    row.set_activatable(false);
    row.set_selectable(false);
    row.set_title_lines(3);
    row
}

pub struct MoreListRow {
    pub container: adw::PreferencesGroup,
    pub row: adw::ActionRow,
}

pub fn more_list_row(title: &str, tooltip: &str) -> MoreListRow {
    let container = adw::PreferencesGroup::new();
    container.set_margin_top(4);
    container.set_margin_bottom(18);
    container.set_margin_start(4);
    container.set_margin_end(4);

    let row = adw::ActionRow::builder().title(gettext(title)).build();
    row.set_tooltip_text(Some(&gettext(tooltip)));
    row.set_activatable(true);
    row.add_prefix(&gtk::Image::from_icon_name("view-more-symbolic"));
    row.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));
    container.add(&row);

    MoreListRow { container, row }
}

pub fn page_box() -> gtk::Box {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 18);
    page.set_margin_top(18);
    page.set_margin_bottom(18);
    page.set_margin_start(18);
    page.set_margin_end(18);
    page
}

pub fn clear_box(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

pub fn clear_list_box(container: &gtk::ListBox) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

pub fn toggle_search_bar(search_bar: &gtk::SearchBar, search_entry: &gtk::SearchEntry) {
    let enabled = !search_bar.is_search_mode();
    search_bar.set_search_mode(enabled);
    if enabled {
        search_entry.grab_focus();
        search_entry.select_region(0, -1);
    }
}

pub fn connect_search_button(
    search_button: &gtk::Button,
    search_bar: &gtk::SearchBar,
    search_entry: &gtk::SearchEntry,
) {
    let search_bar_for_button = search_bar.clone();
    let search_entry_for_button = search_entry.clone();
    search_button.connect_clicked(move |_| {
        toggle_search_bar(&search_bar_for_button, &search_entry_for_button);
    });
}

pub fn connect_primary_f_shortcut(widget: &impl IsA<gtk::Widget>, on_find: impl Fn() + 'static) {
    let key_controller = gtk::EventControllerKey::new();
    key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
    key_controller.connect_key_pressed(move |_, key, _, modifier| {
        let is_find_shortcut = modifier.contains(gtk::gdk::ModifierType::CONTROL_MASK)
            && !modifier.contains(gtk::gdk::ModifierType::ALT_MASK)
            && matches!(key.to_unicode(), Some('f') | Some('F'));
        if !is_find_shortcut {
            return gtk::glib::Propagation::Proceed;
        }

        on_find();
        gtk::glib::Propagation::Stop
    });
    widget.add_controller(key_controller);
}

pub fn connect_search_shortcut(
    widget: &impl IsA<gtk::Widget>,
    search_bar: &gtk::SearchBar,
    search_entry: &gtk::SearchEntry,
) {
    let search_bar_for_shortcut = search_bar.clone();
    let search_entry_for_shortcut = search_entry.clone();
    connect_primary_f_shortcut(widget, move || {
        toggle_search_bar(&search_bar_for_shortcut, &search_entry_for_shortcut);
    });
}
