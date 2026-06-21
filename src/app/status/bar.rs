use super::*;

pub(in crate::app) struct StatusBar {
    pub(in crate::app) container: gtk::Box,
    pub(in crate::app) icon: gtk::Image,
    pub(in crate::app) spinner: adw::Spinner,
    pub(in crate::app) label: gtk::Label,
    pub(in crate::app) action_group: gtk::Box,
    pub(in crate::app) history_button: gtk::Button,
    pub(in crate::app) search_preset_button: gtk::MenuButton,
    pub(in crate::app) page_actions_button: gtk::MenuButton,
    pub(in crate::app) hide_button: gtk::Button,
}

#[derive(Clone)]
pub(in crate::app) struct StatusHandle {
    icon: gtk::Image,
    spinner: adw::Spinner,
    label: gtk::Label,
}

impl StatusHandle {
    pub(in crate::app) fn from_status_bar(status_bar: &StatusBar) -> Self {
        Self {
            icon: status_bar.icon.clone(),
            spinner: status_bar.spinner.clone(),
            label: status_bar.label.clone(),
        }
    }

    pub(in crate::app) fn set_text(&self, message: &str) {
        self.label.set_text(message);
    }

    pub(in crate::app) fn set_loading(&self, loading: bool) {
        self.icon.set_visible(!loading);
        self.spinner.set_visible(loading);
    }
}

pub(in crate::app) fn build_status_bar() -> StatusBar {
    let builder = ui::builder_from_resource("status-bar.ui");
    let container = ui::builder_object::<gtk::Box>(&builder, "status_bar", "status-bar.ui");
    let status_icon = ui::builder_object::<gtk::Image>(&builder, "status_icon", "status-bar.ui");
    let spinner = ui::builder_object::<adw::Spinner>(&builder, "status_spinner", "status-bar.ui");
    let label = ui::builder_object::<gtk::Label>(&builder, "status_label", "status-bar.ui");
    install_status_scroll_guard(&container, &status_icon, &spinner, &label);
    let action_group =
        ui::builder_object::<gtk::Box>(&builder, "status_action_group", "status-bar.ui");
    let history_button =
        ui::builder_object::<gtk::Button>(&builder, "status_history_button", "status-bar.ui");
    history_button.set_tooltip_text(Some(&tr("Show message history")));
    let search_preset_button = ui::builder_object::<gtk::MenuButton>(
        &builder,
        "status_search_preset_button",
        "status-bar.ui",
    );
    set_search_preset_menu_model(&search_preset_button);
    let page_actions_button = ui::builder_object::<gtk::MenuButton>(
        &builder,
        "status_page_actions_button",
        "status-bar.ui",
    );
    set_page_actions_menu_namespace(&page_actions_button, "app");
    let hide_button =
        ui::builder_object::<gtk::Button>(&builder, "status_hide_button", "status-bar.ui");
    hide_button.set_tooltip_text(Some(&tr("Hide message")));

    StatusBar {
        container,
        icon: status_icon,
        spinner,
        label,
        action_group,
        history_button,
        search_preset_button,
        page_actions_button,
        hide_button,
    }
}

fn set_search_preset_menu_model(menu_button: &gtk::MenuButton) {
    menu_button.set_tooltip_text(Some(&tr("Search filters")));

    let menu = gtk::gio::Menu::new();
    for section in [
        SearchPresetSection::General,
        SearchPresetSection::Transactions,
        SearchPresetSection::Diagnostics,
    ] {
        let section_menu = gtk::gio::Menu::new();
        for preset in search_preset_specs()
            .iter()
            .filter(|preset| preset.section == section)
        {
            append_search_preset(&section_menu, preset.label, preset.id);
        }
        let label = section.label().map(tr);
        menu.append_section(label.as_deref(), &section_menu);
    }

    menu_button.set_menu_model(Some(&menu));
}

fn append_search_preset(menu: &gtk::gio::Menu, label: &str, preset: &str) {
    let item = gtk::gio::MenuItem::new(Some(&tr(label)), Some(SEARCH_PRESET_DETAILED_ACTION));
    item.set_attribute_value("target", Some(&preset.to_variant()));
    menu.append_item(&item);
}

pub(super) fn set_page_actions_menu_namespace(
    menu_button: &gtk::MenuButton,
    action_namespace: &str,
) {
    menu_button.set_tooltip_text(Some(&tr("Page actions")));

    let menu = gtk::gio::Menu::new();
    menu.append(
        Some(&tr("Copy Page")),
        Some(&format!("{action_namespace}.copy-page")),
    );
    menu.append(
        Some(&tr("Print Page")),
        Some(&format!("{action_namespace}.print-page")),
    );
    menu.append(
        Some(&tr("Export CSV")),
        Some(&format!("{action_namespace}.export-csv")),
    );
    menu_button.set_menu_model(Some(&menu));
}

pub(super) fn status_button(icon_name: &str, tooltip: &str) -> gtk::Button {
    let button = ui::icon_button(icon_name, tooltip);
    button.add_css_class("flat");
    button
}

fn install_status_scroll_guard(
    container: &gtk::Box,
    icon: &gtk::Image,
    spinner: &adw::Spinner,
    label: &gtk::Label,
) {
    let container_for_visible = container.clone();
    container.connect_visible_notify(move |_| {
        ui::preserve_descendant_scroll_positions(&container_for_visible);
    });

    let container_for_icon = container.clone();
    icon.connect_visible_notify(move |_| {
        ui::preserve_descendant_scroll_positions(&container_for_icon);
    });

    let container_for_spinner = container.clone();
    spinner.connect_visible_notify(move |_| {
        ui::preserve_descendant_scroll_positions(&container_for_spinner);
    });

    let container_for_label = container.clone();
    label.connect_label_notify(move |_| {
        ui::preserve_descendant_scroll_positions(&container_for_label);
    });
}
