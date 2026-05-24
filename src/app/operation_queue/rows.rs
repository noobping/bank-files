use super::super::*;
use super::apply::apply_one;
use super::details::{
    operation_details, operation_details_icon_name, operation_status_text, point_is_inside_child,
    toggle_operation_details,
};
use super::model::{QueuedOperation, QueuedOperationStatus};
use super::presentation::{operation_apply_button_sensitive, operation_subtitle, operation_title};
use super::widgets::refresh_operation_queue_ui;

pub(super) fn operation_row(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    operation: QueuedOperation,
) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::builder()
        .activatable(true)
        .selectable(false)
        .build();
    row.set_tooltip_text(Some(&tr("Show operation details")));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.set_hexpand(true);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(10);
    content.set_margin_end(10);

    let summary = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    summary.set_hexpand(true);

    let details_revealer = gtk::Revealer::new();
    details_revealer.set_transition_type(gtk::RevealerTransitionType::SlideDown);
    details_revealer.set_reveal_child(false);

    let expand_icon = gtk::Image::from_icon_name(operation_details_icon_name(false));
    expand_icon.add_css_class("dim-label");
    expand_icon.set_valign(gtk::Align::Start);
    summary.append(&expand_icon);

    let labels = gtk::Box::new(gtk::Orientation::Vertical, 2);
    labels.set_hexpand(true);
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    header.set_hexpand(true);
    let title = gtk::Label::new(Some(&operation_title(&operation.kind)));
    title.set_selectable(false);
    title.set_xalign(0.0);
    title.set_width_chars(1);
    title.set_max_width_chars(20);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title.set_hexpand(true);
    header.append(&title);

    if matches!(operation.status, QueuedOperationStatus::Applying) {
        let spinner = ui::loading_spinner();
        spinner.set_size_request(16, 16);
        header.append(&spinner);
    }
    let status = gtk::Label::new(Some(&operation_status_text(&operation.status)));
    status.add_css_class("dim-label");
    status.set_selectable(false);
    status.set_xalign(1.0);
    status.set_width_chars(1);
    status.set_max_width_chars(10);
    status.set_ellipsize(gtk::pango::EllipsizeMode::End);
    header.append(&status);
    labels.append(&header);

    let subtitle = gtk::Label::new(Some(&operation_subtitle(&operation.kind)));
    subtitle.set_selectable(false);
    subtitle.add_css_class("dim-label");
    subtitle.set_xalign(0.0);
    subtitle.set_width_chars(1);
    subtitle.set_max_width_chars(34);
    subtitle.set_ellipsize(gtk::pango::EllipsizeMode::End);
    labels.append(&subtitle);

    if let QueuedOperationStatus::Failed(message) = &operation.status {
        let error = gtk::Label::new(Some(message));
        error.set_selectable(false);
        error.add_css_class("error");
        error.set_xalign(0.0);
        error.set_width_chars(1);
        error.set_max_width_chars(34);
        error.set_ellipsize(gtk::pango::EllipsizeMode::End);
        labels.append(&error);
    }
    summary.append(&labels);

    let actions = ui::linked_button_group();
    actions.set_halign(gtk::Align::End);
    actions.set_valign(gtk::Align::Start);
    let apply_button = ui::icon_button("object-select-symbolic", "Apply this operation");
    match config_write_availability(ui.as_ref()) {
        ActionAvailability::Available => {
            apply_button.set_visible(true);
            apply_button.set_sensitive(operation_apply_button_sensitive(
                &operation.status,
                ui.operation_queue.is_processing(),
                ui.loading_count.get(),
            ));
        }
        availability => apply_action_availability(&apply_button, &availability),
    }
    let remove_button = ui::icon_button("user-trash-symbolic", "Remove this operation");
    remove_button.set_sensitive(operation.status.can_remove());

    let id = operation.id;
    let state_for_apply = Rc::clone(state);
    let ui_for_apply = Rc::clone(ui);
    apply_button.connect_clicked(move |_| apply_one(&state_for_apply, &ui_for_apply, id));

    let state_for_remove = Rc::clone(state);
    let ui_for_remove = Rc::clone(ui);
    remove_button.connect_clicked(move |_| {
        if ui_for_remove.operation_queue.remove(id) {
            refresh_operation_queue_ui(&state_for_remove, &ui_for_remove);
            show_status(&ui_for_remove, "Queued operation removed.");
        }
    });

    actions.append(&apply_button);
    actions.append(&remove_button);
    summary.append(&actions);
    content.append(&summary);

    let details = ui::wrapped_label(&operation_details(&operation.kind, &operation.status));
    details.add_css_class("dim-label");
    details.set_width_chars(1);
    details.set_max_width_chars(44);
    details.set_margin_top(6);
    details.set_margin_start(32);
    details_revealer.set_child(Some(&details));
    content.append(&details_revealer);

    let details_revealer_for_activate = details_revealer.clone();
    let expand_icon_for_activate = expand_icon.clone();
    row.connect_activate(move |row| {
        toggle_operation_details(
            row,
            &details_revealer_for_activate,
            &expand_icon_for_activate,
        );
    });

    let click = gtk::GestureClick::new();
    click.set_button(0);
    let row_for_click = row.clone();
    let content_for_click = content.clone();
    let actions_for_click = actions.clone();
    let details_revealer_for_click = details_revealer.clone();
    let expand_icon_for_click = expand_icon.clone();
    click.connect_released(move |_, _, x, y| {
        if !point_is_inside_child(&actions_for_click, &content_for_click, x, y) {
            toggle_operation_details(
                &row_for_click,
                &details_revealer_for_click,
                &expand_icon_for_click,
            );
        }
    });
    content.add_controller(click);

    row.set_child(Some(&content));
    row
}
