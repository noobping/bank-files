use super::navigation::update_header_navigation_button;
use super::types::LoadingSensitiveWidget;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) enum ActionAvailability {
    Available,
    Hidden,
    Disabled(String),
}

pub(in crate::app) fn write_action_available(
    writable: bool,
    advanced_features: bool,
    reason: &str,
) -> ActionAvailability {
    if writable {
        ActionAvailability::Available
    } else if advanced_features {
        ActionAvailability::Disabled(reason.to_string())
    } else {
        ActionAvailability::Hidden
    }
}

pub(in crate::app) fn apply_action_availability<W: IsA<gtk::Widget>>(
    widget: &W,
    availability: &ActionAvailability,
) {
    let widget = widget.as_ref();
    match availability {
        ActionAvailability::Available => {
            widget.set_visible(true);
            widget.set_sensitive(true);
            widget.set_tooltip_text(None);
        }
        ActionAvailability::Hidden => {
            widget.set_visible(false);
            widget.set_sensitive(false);
        }
        ActionAvailability::Disabled(reason) => {
            widget.set_visible(true);
            widget.set_sensitive(false);
            widget.set_tooltip_text(Some(&tr(reason)));
        }
    }
}

pub(in crate::app) fn data_write_availability(ui: &UiHandles) -> ActionAvailability {
    let capabilities = ui.storage_capabilities.borrow();
    write_action_available(
        capabilities.data_writable,
        ui.advanced_features.get(),
        capabilities.data_write_reason(),
    )
}

pub(in crate::app) fn config_write_availability(ui: &UiHandles) -> ActionAvailability {
    let capabilities = ui.storage_capabilities.borrow();
    write_action_available(
        capabilities.config_writable,
        ui.advanced_features.get(),
        capabilities.config_write_reason(),
    )
}

pub(in crate::app) fn set_storage_capabilities(
    ui: &Rc<UiHandles>,
    capabilities: data::StorageCapabilities,
) {
    *ui.storage_capabilities.borrow_mut() = capabilities;
    refresh_write_actions(ui.as_ref());
    update_header_navigation_button(ui.as_ref());
}

pub(in crate::app) fn refresh_write_actions(ui: &UiHandles) {
    let capabilities = ui.storage_capabilities.borrow().clone();
    let not_loading = loading_sensitive_items_enabled(ui.loading_count.get());
    let idle = not_loading && !ui.management_dialog_active.get();
    set_app_action_enabled(ui, "reload", not_loading);
    set_app_action_enabled(ui, "reload-all", not_loading);
    set_app_action_enabled(ui, "clear-cache-and-reload", not_loading);
    set_app_action_enabled(ui, "copy-page", not_loading);
    set_app_action_enabled(ui, "print-page", not_loading);
    set_app_action_enabled(ui, "export-csv", not_loading);
    set_app_action_enabled(ui, "import-csv", idle);
    set_app_action_enabled(ui, "configuration", capabilities.config_writable && idle);
    set_app_action_enabled(ui, "manage-rules", capabilities.config_writable && idle);
    set_app_action_enabled(ui, "manage-budgets", capabilities.config_writable && idle);
    set_app_action_enabled(ui, "manage-aliases", capabilities.config_writable && idle);
    set_app_action_enabled(ui, "preferences", ui.preferences.any_writable());
    set_app_action_enabled(
        ui,
        "remember-mode",
        ui.preferences.action_is_writable("remember-mode") && not_loading,
    );
    update_config_action_widgets(ui);
    update_operation_queue_action_widgets(ui);
    update_loading_sensitive_widgets(ui);
}

fn loading_sensitive_items_enabled(loading_count: u32) -> bool {
    loading_count == 0
}

pub(in crate::app) fn register_loading_sensitive_widget<W: IsA<gtk::Widget>>(
    ui: &Rc<UiHandles>,
    widget: &W,
) {
    let widget = widget.clone().upcast::<gtk::Widget>();
    let base_sensitive = widget.is_sensitive();
    let was_rooted = widget.root().is_some();
    widget.set_sensitive(base_sensitive && loading_sensitive_items_enabled(ui.loading_count.get()));
    ui.loading_sensitive_widgets
        .borrow_mut()
        .push(LoadingSensitiveWidget {
            widget,
            base_sensitive,
            was_rooted: Rc::new(Cell::new(was_rooted)),
        });
}

fn update_loading_sensitive_widgets(ui: &UiHandles) {
    let sensitive = loading_sensitive_items_enabled(ui.loading_count.get());
    let mut widgets = ui.loading_sensitive_widgets.borrow_mut();
    widgets.retain(loading_sensitive_widget_should_remain_registered);
    for item in widgets.iter() {
        item.widget.set_sensitive(item.base_sensitive && sensitive);
    }
}

fn loading_sensitive_widget_should_remain_registered(item: &LoadingSensitiveWidget) -> bool {
    let rooted = item.widget.root().is_some();
    if rooted {
        item.was_rooted.set(true);
    }
    widget_registration_is_live(rooted, item.was_rooted.get())
}

fn widget_registration_is_live(rooted: bool, was_rooted: bool) -> bool {
    rooted || !was_rooted
}

fn set_app_action_enabled(ui: &UiHandles, name: &str, enabled: bool) {
    if let Some(action) = ui
        .window
        .application()
        .and_then(|app| app.lookup_action(name))
        .and_then(|action| action.downcast::<gtk::gio::SimpleAction>().ok())
    {
        action.set_enabled(enabled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_action_availability_follows_storage_and_mode() {
        assert_eq!(
            write_action_available(true, false, "Locked"),
            ActionAvailability::Available
        );
        assert_eq!(
            write_action_available(false, false, "Locked"),
            ActionAvailability::Hidden
        );
        assert_eq!(
            write_action_available(false, true, "Locked"),
            ActionAvailability::Disabled("Locked".to_string())
        );
    }

    #[test]
    fn loading_sensitive_items_are_enabled_only_when_idle() {
        assert!(loading_sensitive_items_enabled(0));
        assert!(!loading_sensitive_items_enabled(1));
        assert!(!loading_sensitive_items_enabled(3));
    }

    #[test]
    fn unrooted_loading_widgets_stay_registered_until_first_root() {
        assert!(widget_registration_is_live(false, false));
        assert!(widget_registration_is_live(true, false));
        assert!(!widget_registration_is_live(false, true));
    }
}
