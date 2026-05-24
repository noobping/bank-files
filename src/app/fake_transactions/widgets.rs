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

    let builder = ui::builder_from_resource("fake-transactions-dialog.ui");
    let root = fake_transactions_object::<gtk::Box>(&builder, "fake_transactions_root");
    let title = fake_transactions_object::<adw::WindowTitle>(&builder, "fake_transactions_title");
    let summary = fake_transactions_object::<gtk::Label>(&builder, "fake_transactions_summary");
    let busy_box = fake_transactions_object::<gtk::Box>(&builder, "fake_transactions_busy_box");
    let busy_label =
        fake_transactions_object::<gtk::Label>(&builder, "fake_transactions_busy_label");
    let start_stack =
        fake_transactions_object::<gtk::Stack>(&builder, "fake_transactions_start_stack");
    let back_button =
        fake_transactions_object::<gtk::Button>(&builder, "fake_transactions_back_button");
    let add_button =
        fake_transactions_object::<gtk::Button>(&builder, "fake_transactions_add_button");
    let save_button =
        fake_transactions_object::<gtk::Button>(&builder, "fake_transactions_save_button");
    let save_content =
        fake_transactions_object::<adw::ButtonContent>(&builder, "fake_transactions_save_content");
    let clear_button =
        fake_transactions_object::<gtk::Button>(&builder, "fake_transactions_clear_button");
    let search_bar =
        fake_transactions_object::<gtk::SearchBar>(&builder, "fake_transactions_search_bar");
    let search_entry =
        fake_transactions_object::<gtk::SearchEntry>(&builder, "fake_transactions_search_entry");
    let stack = fake_transactions_object::<gtk::Stack>(&builder, "fake_transactions_stack");
    let list = fake_transactions_object::<gtk::ListBox>(&builder, "fake_transactions_list");
    let form_box = fake_transactions_object::<gtk::Box>(&builder, "fake_transactions_form_box");

    title.set_title(&tr("Fake Transactions"));
    title.set_subtitle(&tr("Runtime preview transactions"));
    back_button.set_tooltip_text(Some(&tr("Back to fake transactions")));
    add_button.set_tooltip_text(Some(&tr("New fake transaction")));
    save_content.set_label(&tr("Save"));
    save_content.set_icon_name("document-save-symbolic");
    save_button.set_tooltip_text(Some(&tr("Save fake transaction")));
    clear_button.set_tooltip_text(Some(&tr("Clear fake transactions")));
    search_entry.set_placeholder_text(Some(&tr("Search fake transactions")));
    search_bar.connect_entry(&search_entry);

    let dialog = ui::content_dialog(tr("Fake Transactions"), &root)
        .content_width(560)
        .content_height(560)
        .default_widget(&add_button)
        .build();
    ui::connect_search_shortcut(&dialog, &search_bar, &search_entry);
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

fn fake_transactions_object<T: IsA<gtk::glib::Object>>(builder: &gtk::Builder, id: &str) -> T {
    ui::builder_object(builder, id, "fake-transactions-dialog.ui")
}

pub(in crate::app) fn focus_fake_transaction_search(ui: &UiHandles) -> bool {
    let widgets = &ui.fake_transaction_widgets;
    if !widgets.dialog.is_mapped() {
        return false;
    }

    ui::toggle_search_bar(&widgets.search_bar, &widgets.search_entry);
    true
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
