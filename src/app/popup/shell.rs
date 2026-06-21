use super::*;

const LIST_PAGE: &str = "list";
const FORM_PAGE: &str = "form";
const PREFERENCES_DIALOG_MAX_HEIGHT: i32 = 720;

struct PopupTemplateIds {
    resource: &'static str,
    root: &'static str,
    header: &'static str,
    title: &'static str,
    search_button: &'static str,
    search_bar: &'static str,
    search_entry: &'static str,
}

const ACTION_POPUP_TEMPLATE: PopupTemplateIds = PopupTemplateIds {
    resource: "action-dialog.ui",
    root: "action_root",
    header: "action_header",
    title: "action_title",
    search_button: "action_search_button",
    search_bar: "action_search_bar",
    search_entry: "action_search_entry",
};

const SETTINGS_POPUP_TEMPLATE: PopupTemplateIds = PopupTemplateIds {
    resource: "settings-dialog.ui",
    root: "settings_root",
    header: "settings_header",
    title: "settings_title",
    search_button: "settings_search_button",
    search_bar: "settings_search_bar",
    search_entry: "settings_search_entry",
};

struct PopupTemplate {
    builder: gtk::Builder,
    root: gtk::Box,
    header: adw::HeaderBar,
    search_bar: gtk::SearchBar,
    search_entry: gtk::SearchEntry,
}

pub(in crate::app) struct ActionDialogShell {
    pub(in crate::app) root: gtk::Box,
    pub(in crate::app) back_button: gtk::Button,
    pub(in crate::app) submit_button: gtk::Button,
    pub(in crate::app) search_bar: gtk::SearchBar,
    pub(in crate::app) search_entry: gtk::SearchEntry,
    pub(in crate::app) stack: gtk::Stack,
    start_stack: gtk::Stack,
}

impl ActionDialogShell {
    pub(in crate::app) fn set_list_page(&self) {
        self.stack.set_visible_child_name(LIST_PAGE);
        self.start_stack.set_visible_child_name("search");
    }

    pub(in crate::app) fn set_form_only(&self) {
        self.search_bar.set_search_mode(false);
        self.start_stack.set_visible_child_name("empty");
    }

    pub(in crate::app) fn add_list_page(&self, child: &impl IsA<gtk::Widget>) {
        self.stack.add_named(child, Some(LIST_PAGE));
    }

    pub(in crate::app) fn add_form_page(&self, child: &impl IsA<gtk::Widget>) {
        self.stack.add_named(child, Some(FORM_PAGE));
    }

    pub(in crate::app) fn page_handle(&self) -> ActionDialogPageHandle {
        ActionDialogPageHandle {
            stack: self.stack.clone(),
            start_stack: self.start_stack.clone(),
            search_bar: self.search_bar.clone(),
        }
    }
}

#[derive(Clone)]
pub(in crate::app) struct ActionDialogPageHandle {
    stack: gtk::Stack,
    start_stack: gtk::Stack,
    search_bar: gtk::SearchBar,
}

impl ActionDialogPageHandle {
    pub(in crate::app) fn set_list_page(&self) {
        self.stack.set_visible_child_name(LIST_PAGE);
        self.start_stack.set_visible_child_name("search");
    }

    pub(in crate::app) fn set_form_page(&self) {
        self.search_bar.set_search_mode(false);
        self.stack.set_visible_child_name(FORM_PAGE);
        self.start_stack.set_visible_child_name("back");
    }
}

pub(in crate::app) struct SettingsDialogShell {
    pub(in crate::app) root: gtk::Box,
    pub(in crate::app) header: adw::HeaderBar,
    pub(in crate::app) search_bar: gtk::SearchBar,
    pub(in crate::app) search_entry: gtk::SearchEntry,
}

pub(in crate::app) struct ActionFormDialog {
    pub(in crate::app) dialog: adw::Dialog,
    pub(in crate::app) page: gtk::Box,
    pub(in crate::app) submit_button: gtk::Button,
    pub(in crate::app) status: gtk::Label,
}

pub(in crate::app) fn build_action_dialog_shell(
    title: &str,
    subtitle: &str,
    submit_label: &str,
    submit_icon_name: &str,
    submit_tooltip: &str,
    search_placeholder: &str,
) -> ActionDialogShell {
    let template =
        build_popup_template(&ACTION_POPUP_TEMPLATE, title, subtitle, search_placeholder);

    let start_stack = popup_object::<gtk::Stack>(
        &template.builder,
        "action_start_stack",
        ACTION_POPUP_TEMPLATE.resource,
    );
    let back_button = popup_object::<gtk::Button>(
        &template.builder,
        "action_back_button",
        ACTION_POPUP_TEMPLATE.resource,
    );
    let submit_button = popup_object::<gtk::Button>(
        &template.builder,
        "action_submit_button",
        ACTION_POPUP_TEMPLATE.resource,
    );
    let submit_content = popup_object::<adw::ButtonContent>(
        &template.builder,
        "action_submit_content",
        ACTION_POPUP_TEMPLATE.resource,
    );
    submit_content.set_label(&tr(submit_label));
    submit_content.set_icon_name(submit_icon_name);
    submit_button.set_tooltip_text(Some(&tr(submit_tooltip)));

    let stack = popup_object::<gtk::Stack>(
        &template.builder,
        "action_stack",
        ACTION_POPUP_TEMPLATE.resource,
    );

    ActionDialogShell {
        root: template.root,
        back_button,
        submit_button,
        search_bar: template.search_bar,
        search_entry: template.search_entry,
        stack,
        start_stack,
    }
}

pub(in crate::app) fn build_settings_dialog_shell(
    title: &str,
    search_placeholder: &str,
) -> SettingsDialogShell {
    let template = build_popup_template(&SETTINGS_POPUP_TEMPLATE, title, "", search_placeholder);
    SettingsDialogShell {
        root: template.root,
        header: template.header,
        search_bar: template.search_bar,
        search_entry: template.search_entry,
    }
}

pub(in crate::app) fn settings_dialog_scroll(child: &impl IsA<gtk::Widget>) -> gtk::ScrolledWindow {
    ui::action_dialog_scroll(child)
}

pub(in crate::app) fn preferences_dialog_scroll(
    child: &impl IsA<gtk::Widget>,
) -> gtk::ScrolledWindow {
    ui::action_dialog_scroll_with_limits(child, 0, PREFERENCES_DIALOG_MAX_HEIGHT)
}

pub(in crate::app) fn settings_content_dialog(
    title: &str,
    root: &impl IsA<gtk::Widget>,
    content_width: i32,
) -> adw::Dialog {
    ui::content_dialog(tr(title), root)
        .content_width(content_width)
        .build()
}

pub(in crate::app) fn build_action_form_dialog(
    title: &str,
    subtitle: &str,
    submit_label: &str,
    submit_icon_name: &str,
    submit_tooltip: &str,
    search_placeholder: &str,
    content_width: i32,
) -> ActionFormDialog {
    let shell = build_action_dialog_shell(
        title,
        subtitle,
        submit_label,
        submit_icon_name,
        submit_tooltip,
        search_placeholder,
    );
    shell.set_form_only();

    let page = ui::page_box();
    shell.add_form_page(&ui::action_dialog_scroll(&page));

    let status = ui::wrapped_label("");
    status.add_css_class("dim-label");

    let submit_button = shell.submit_button.clone();
    let dialog = ui::content_dialog(tr(title), &shell.root)
        .content_width(content_width)
        .default_widget(&submit_button)
        .build();

    ActionFormDialog {
        dialog,
        page,
        submit_button,
        status,
    }
}

fn build_popup_template(
    ids: &PopupTemplateIds,
    title: &str,
    subtitle: &str,
    search_placeholder: &str,
) -> PopupTemplate {
    let builder = ui::builder_from_resource(ids.resource);
    let root = popup_object::<gtk::Box>(&builder, ids.root, ids.resource);
    let header = popup_object::<adw::HeaderBar>(&builder, ids.header, ids.resource);
    let title_widget = popup_object::<adw::WindowTitle>(&builder, ids.title, ids.resource);
    title_widget.set_title(&tr(title));
    title_widget.set_subtitle(&tr(subtitle));

    let search_button = popup_object::<gtk::Button>(&builder, ids.search_button, ids.resource);

    let search_bar = popup_object::<gtk::SearchBar>(&builder, ids.search_bar, ids.resource);
    let search_entry = popup_object::<gtk::SearchEntry>(&builder, ids.search_entry, ids.resource);
    search_entry.set_placeholder_text(Some(&tr(search_placeholder)));
    ui::connect_search_button(&search_button, &search_bar, &search_entry);

    PopupTemplate {
        builder,
        root,
        header,
        search_bar,
        search_entry,
    }
}

fn popup_object<T: IsA<gtk::glib::Object>>(builder: &gtk::Builder, id: &str, resource: &str) -> T {
    ui::builder_object(builder, id, resource)
}
