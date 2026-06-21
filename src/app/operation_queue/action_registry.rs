use super::super::*;
use super::messages::operation_already_queued_tooltip;
use super::model::{
    OperationQueue, OperationQueueActionRegistration, OperationQueueActionTarget,
    QueuedOperationKind,
};

pub(in crate::app) fn update_operation_queue_action_widgets(ui: &UiHandles) {
    ui.operation_queue.update_registered_actions(ui);
}

pub(in crate::app) fn register_operation_queue_widget<W: IsA<gtk::Widget>>(
    ui: &Rc<UiHandles>,
    widget: &W,
    kind: QueuedOperationKind,
) {
    let widget = widget.clone().upcast::<gtk::Widget>();
    let registration = OperationQueueActionRegistration {
        owner: widget.clone(),
        target: OperationQueueActionTarget::Widget(widget.clone()),
        kind,
        base_enabled: widget.is_sensitive(),
        base_visible: widget.is_visible(),
        base_tooltip: widget.tooltip_text().map(|text| text.to_string()),
        was_rooted: Rc::new(Cell::new(widget.root().is_some())),
    };
    apply_operation_queue_action_state(ui.as_ref(), &registration);
    ui.operation_queue
        .action_registrations
        .borrow_mut()
        .push(registration);
}

pub(in crate::app) fn register_operation_queue_menu_action<W: IsA<gtk::Widget>>(
    ui: &Rc<UiHandles>,
    owner: &W,
    action: &gtk::gio::SimpleAction,
    kind: QueuedOperationKind,
) {
    let owner = owner.clone().upcast::<gtk::Widget>();
    let registration = OperationQueueActionRegistration {
        owner: owner.clone(),
        target: OperationQueueActionTarget::MenuAction(action.clone()),
        kind,
        base_enabled: action.is_enabled(),
        base_visible: true,
        base_tooltip: None,
        was_rooted: Rc::new(Cell::new(owner.root().is_some())),
    };
    apply_operation_queue_action_state(ui.as_ref(), &registration);
    ui.operation_queue
        .action_registrations
        .borrow_mut()
        .push(registration);
}

fn operation_queue_action_registration_is_live(
    registration: &OperationQueueActionRegistration,
) -> bool {
    let rooted = registration.owner.root().is_some();
    if rooted {
        registration.was_rooted.set(true);
    }
    rooted || !registration.was_rooted.get()
}

pub(super) fn operation_queue_action_enabled(
    base_enabled: bool,
    queued: bool,
    config_enabled: bool,
) -> bool {
    base_enabled && config_enabled && !queued
}

fn apply_operation_queue_action_state(
    ui: &UiHandles,
    registration: &OperationQueueActionRegistration,
) {
    let queued = ui.operation_queue.contains_kind(&registration.kind);
    let config_idle = !ui.management_dialog_active.get() && ui.loading_count.get() == 0;
    match &registration.target {
        OperationQueueActionTarget::Widget(widget) => match config_write_availability(ui) {
            ActionAvailability::Available => {
                let enabled =
                    operation_queue_action_enabled(registration.base_enabled, queued, config_idle);
                widget.set_visible(registration.base_visible);
                widget.set_sensitive(enabled);
                if queued {
                    widget.set_tooltip_text(Some(&tr(operation_already_queued_tooltip())));
                } else {
                    widget.set_tooltip_text(registration.base_tooltip.as_deref());
                }
            }
            ActionAvailability::Hidden => {
                widget.set_visible(false);
                widget.set_sensitive(false);
                widget.set_tooltip_text(None);
            }
            ActionAvailability::Disabled(reason) => {
                widget.set_visible(registration.base_visible);
                widget.set_sensitive(false);
                widget.set_tooltip_text(Some(&tr(&reason)));
            }
        },
        OperationQueueActionTarget::MenuAction(action) => {
            let config_enabled =
                matches!(config_write_availability(ui), ActionAvailability::Available)
                    && config_idle;
            action.set_enabled(operation_queue_action_enabled(
                registration.base_enabled,
                queued,
                config_enabled,
            ));
        }
    }
}

impl OperationQueue {
    fn update_registered_actions(&self, ui: &UiHandles) {
        let mut registrations = self.action_registrations.borrow_mut();
        registrations.retain(operation_queue_action_registration_is_live);
        for registration in registrations.iter() {
            apply_operation_queue_action_state(ui, registration);
        }
    }
}
