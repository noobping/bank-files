use super::super::*;
use super::form::FakeTransactionFormState;
use super::{FAKE_TRANSACTIONS_FORM_PAGE, FAKE_TRANSACTIONS_LIST_PAGE};

#[derive(Clone)]
pub(in crate::app) struct FakeTransactionWidgets {
    pub(in crate::app) button: gtk::Button,
    pub(super) badge: gtk::Label,
    pub(super) summary: gtk::Label,
    pub(super) busy_box: gtk::Box,
    pub(super) busy_label: gtk::Label,
    pub(super) start_stack: gtk::Stack,
    pub(super) back_button: gtk::Button,
    pub(super) add_button: gtk::Button,
    pub(super) save_button: gtk::Button,
    pub(super) clear_button: gtk::Button,
    pub(super) search_bar: gtk::SearchBar,
    pub(super) search_entry: gtk::SearchEntry,
    pub(super) stack: gtk::Stack,
    pub(super) list: gtk::ListBox,
    pub(super) form_box: gtk::Box,
    pub(super) dialog: adw::Dialog,
    pub(super) form_state: Rc<RefCell<Option<FakeTransactionFormState>>>,
}

pub(in crate::app) fn build_fake_transaction_widgets() -> FakeTransactionWidgets {
    let badge = gtk::Label::new(None);
    badge.add_css_class("caption");
    badge.set_visible(false);
    badge.set_halign(gtk::Align::Center);
    badge.set_valign(gtk::Align::Center);

    let icon = gtk::Image::from_icon_name("document-new-symbolic");
    let button_content = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    button_content.append(&badge);
    button_content.append(&icon);

    let button = ui::flat_custom_button("Fake transactions", &button_content);
    button.set_focus_on_click(false);

    let header = ui::cancelable_dialog_header("Fake Transactions", "Runtime preview transactions");

    let clear_button = ui::icon_button("edit-clear-symbolic", "Clear fake transactions");
    clear_button.add_css_class("flat");
    clear_button.set_valign(gtk::Align::Start);

    let back_button = ui::icon_button("go-previous-symbolic", "Back to fake transactions");
    back_button.add_css_class("flat");
    let add_button = ui::icon_button("list-add-symbolic", "New fake transaction");
    add_button.add_css_class("flat");

    let start_stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::Crossfade)
        .build();
    start_stack.add_named(&add_button, Some("list"));
    start_stack.add_named(&back_button, Some("form"));
    start_stack.set_visible_child_name("list");
    header.pack_start(&start_stack);

    let save_button =
        ui::primary_text_icon_button("document-save-symbolic", "Save", "Save fake transaction");
    save_button.set_visible(false);
    header.pack_end(&save_button);

    let search_bar = gtk::SearchBar::builder()
        .show_close_button(true)
        .search_mode_enabled(false)
        .build();
    let search_entry = gtk::SearchEntry::builder()
        .placeholder_text(tr("Search fake transactions"))
        .hexpand(true)
        .build();
    let search_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    search_box.set_margin_top(8);
    search_box.set_margin_bottom(8);
    search_box.set_margin_start(12);
    search_box.set_margin_end(12);
    search_box.append(&search_entry);
    search_bar.set_child(Some(&search_box));
    search_bar.connect_entry(&search_entry);

    let summary = gtk::Label::new(None);
    summary.add_css_class("dim-label");
    summary.set_selectable(false);
    summary.set_xalign(0.0);

    let busy_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    busy_box.add_css_class("dim-label");
    busy_box.set_visible(false);
    let busy_spinner = ui::loading_spinner();
    busy_spinner.set_size_request(16, 16);
    let busy_label = gtk::Label::new(None);
    busy_label.set_selectable(false);
    busy_label.set_xalign(0.0);
    busy_label.set_hexpand(true);
    busy_box.append(&busy_spinner);
    busy_box.append(&busy_label);

    let summary_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    summary_row.set_hexpand(true);
    summary.set_hexpand(true);
    summary_row.append(&summary);
    summary_row.append(&clear_button);

    let list_page = ui::page_box();
    list_page.append(&summary_row);
    list_page.append(&busy_box);

    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    list.set_selection_mode(gtk::SelectionMode::None);
    list.set_hexpand(true);
    list_page.append(&list);

    let form_box = ui::page_box();
    let stack = gtk::Stack::builder()
        .hhomogeneous(false)
        .vhomogeneous(false)
        .transition_type(gtk::StackTransitionType::SlideLeftRight)
        .hexpand(true)
        .build();
    stack.add_named(&list_page, Some(FAKE_TRANSACTIONS_LIST_PAGE));
    stack.add_named(&form_box, Some(FAKE_TRANSACTIONS_FORM_PAGE));
    stack.set_visible_child_name(FAKE_TRANSACTIONS_LIST_PAGE);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.append(&search_bar);
    root.append(&ui::action_dialog_scroll_with_min(&stack, 320));
    let view = ui::dialog_toolbar_view(&header, &root);

    let dialog = ui::content_dialog(tr("Fake Transactions"), &view)
        .content_width(560)
        .content_height(560)
        .default_widget(&add_button)
        .build();
    let focus_search = {
        let search_bar = search_bar.clone();
        let search_entry = search_entry.clone();
        move || focus_fake_transaction_search_bar(&search_bar, &search_entry)
    };
    ui::connect_primary_f_shortcut(&dialog, focus_search.clone());
    ui::connect_primary_f_shortcut(&view, focus_search);
    search_bar.set_key_capture_widget(Some(&dialog));

    FakeTransactionWidgets {
        button,
        badge,
        summary,
        busy_box,
        busy_label,
        start_stack,
        back_button,
        add_button,
        save_button,
        clear_button,
        search_bar,
        search_entry,
        stack,
        list,
        form_box,
        dialog,
        form_state: Rc::new(RefCell::new(None)),
    }
}

pub(in crate::app) fn focus_fake_transaction_search(ui: &UiHandles) -> bool {
    let widgets = &ui.fake_transaction_widgets;
    if !widgets.dialog.is_mapped() {
        return false;
    }

    focus_fake_transaction_search_bar(&widgets.search_bar, &widgets.search_entry);
    true
}

pub(super) fn focus_fake_transaction_search_bar(
    search_bar: &gtk::SearchBar,
    search_entry: &gtk::SearchEntry,
) {
    search_bar.set_search_mode(true);
    search_entry.grab_focus();
    search_entry.select_region(0, -1);
}

pub(super) fn show_fake_transaction_list(widgets: &FakeTransactionWidgets) {
    widgets.search_bar.set_search_mode(false);
    widgets
        .stack
        .set_visible_child_name(FAKE_TRANSACTIONS_LIST_PAGE);
    widgets.start_stack.set_visible_child_name("list");
    widgets.save_button.set_visible(false);
    widgets.dialog.set_default_widget(Some(&widgets.add_button));
    widgets.form_state.borrow_mut().take();
    if !widgets.search_bar.is_search_mode() && widgets.add_button.is_sensitive() {
        widgets.add_button.grab_focus();
    }
}

pub(super) fn show_fake_transaction_form_page(widgets: &FakeTransactionWidgets) {
    widgets.search_bar.set_search_mode(false);
    widgets
        .stack
        .set_visible_child_name(FAKE_TRANSACTIONS_FORM_PAGE);
    widgets.start_stack.set_visible_child_name("form");
    widgets.save_button.set_visible(true);
    widgets
        .dialog
        .set_default_widget(Some(&widgets.save_button));
}
