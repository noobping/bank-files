use super::super::*;
use super::form::FakeTransactionFormState;
use super::presentation::FAKE_TRANSACTIONS_TITLE;
use super::{FAKE_TRANSACTIONS_FORM_PAGE, FAKE_TRANSACTIONS_LIST_PAGE};

#[derive(Clone)]
pub(in crate::app) struct FakeTransactionWidgets {
    pub(in crate::app) button: gtk::Button,
    pub(super) badge: gtk::Label,
    pub(super) summary_row: gtk::Box,
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
    let status_button = ui::flat_badge_icon_button("document-new-symbolic", "Fake transactions");
    let button = status_button.button;
    let badge = status_button.badge;

    let builder = ui::builder_from_resource("fake-transactions-dialog.ui");
    let root = fake_transactions_object::<gtk::Box>(&builder, "fake_transactions_root");
    let summary_row =
        fake_transactions_object::<gtk::Box>(&builder, "fake_transactions_summary_row");
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
    let clear_button =
        fake_transactions_object::<gtk::Button>(&builder, "fake_transactions_clear_button");
    let search_bar =
        fake_transactions_object::<gtk::SearchBar>(&builder, "fake_transactions_search_bar");
    let search_entry =
        fake_transactions_object::<gtk::SearchEntry>(&builder, "fake_transactions_search_entry");
    let stack = fake_transactions_object::<gtk::Stack>(&builder, "fake_transactions_stack");
    let list = fake_transactions_object::<gtk::ListBox>(&builder, "fake_transactions_list");
    let form_box = fake_transactions_object::<gtk::Box>(&builder, "fake_transactions_form_box");

    let dialog = ui::content_dialog(tr(FAKE_TRANSACTIONS_TITLE), &root)
        .content_width(560)
        .content_height(560)
        .default_widget(&add_button)
        .build();
    ui::bind_search_bar(&dialog, &dialog, &search_bar, &search_entry);

    FakeTransactionWidgets {
        button,
        badge,
        summary_row,
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
