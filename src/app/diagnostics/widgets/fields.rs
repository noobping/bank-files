use super::super::mapping::{diagnostic_field_items, show_field_mapping_dialog, DiagnosticField};
use super::*;

pub(in crate::app) fn detected_fields_toggle(
    report: &ImportReport,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    fields_visibility: DetectedFieldsVisibility,
) -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let reveal_initially = fields_visibility.reveal_initially(ui_handles.show_all.get());
    let button_content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let button = ui::flat_custom_button(
        if reveal_initially {
            "Hide detected fields"
        } else {
            "Show detected fields"
        },
        &button_content,
    );

    let icon = gtk::Image::from_icon_name("format-justify-left-symbolic");
    button_content.append(&icon);
    let label = ui::wrapped_label(&tr("Detected fields"));
    label.add_css_class("caption");
    label.set_hexpand(true);
    button_content.append(&label);
    let indicator = gtk::Image::from_icon_name(if reveal_initially {
        "go-up-symbolic"
    } else {
        "go-down-symbolic"
    });
    button_content.append(&indicator);
    container.append(&button);

    let revealer = gtk::Revealer::builder()
        .transition_type(gtk::RevealerTransitionType::SlideDown)
        .reveal_child(reveal_initially)
        .child(&diagnostic_field_flow(report, state, ui_handles))
        .build();
    container.append(&revealer);

    let revealer_for_toggle = revealer.clone();
    let indicator_for_toggle = indicator.clone();
    button.connect_clicked(move |button| {
        let reveal = !revealer_for_toggle.reveals_child();
        revealer_for_toggle.set_reveal_child(reveal);
        indicator_for_toggle.set_icon_name(Some(if reveal {
            "go-up-symbolic"
        } else {
            "go-down-symbolic"
        }));
        button.set_tooltip_text(Some(&tr(if reveal {
            "Hide detected fields"
        } else {
            "Show detected fields"
        })));
    });

    container
}

pub(in crate::app) fn diagnostic_field_flow(
    report: &ImportReport,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> gtk::FlowBox {
    let flow = gtk::FlowBox::builder()
        .column_spacing(8)
        .row_spacing(8)
        .homogeneous(true)
        .selection_mode(gtk::SelectionMode::None)
        .min_children_per_line(1)
        .max_children_per_line(3)
        .build();
    flow.set_hexpand(true);

    for field in diagnostic_field_items(&report.guessed_fields) {
        flow.insert(&diagnostic_field_chip(field, report, state, ui_handles), -1);
    }

    flow
}

pub(in crate::app) fn diagnostic_field_chip(
    field: DiagnosticField<'_>,
    report: &ImportReport,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> gtk::Box {
    let chip = gtk::Box::new(gtk::Orientation::Vertical, 3);
    chip.set_margin_top(4);
    chip.set_margin_bottom(4);
    chip.set_margin_start(4);
    chip.set_margin_end(4);

    let title = gtk::Label::new(Some(&tr(field.label)));
    title.add_css_class("caption");
    title.add_css_class("dim-label");
    title.set_xalign(0.0);
    chip.append(&title);

    let value_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let icon = gtk::Image::from_icon_name(if field.value.is_some() {
        "dialog-information-symbolic"
    } else {
        "dialog-warning-symbolic"
    });
    value_row.append(&icon);

    let value_text = field
        .value
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| tr("Not detected"));
    let value_label = ui::selectable_wrapped_label(&value_text);
    value_label.set_hexpand(true);
    if field.value.is_none() {
        value_label.add_css_class("dim-label");
    }
    value_row.append(&value_label);

    let map_button = ui::icon_button("document-edit-symbolic", "Map CSV header to this field");
    map_button.add_css_class("flat");
    map_button.set_sensitive(!report.headers.is_empty());
    register_config_widget(ui_handles, &map_button);
    let headers = report.headers.clone();
    let canonical = field.canonical.to_string();
    let label = field.label.to_string();
    let current_value = field.value.map(ToOwned::to_owned);
    let state_for_map = Rc::clone(state);
    let ui_for_map = Rc::clone(ui_handles);
    map_button.connect_clicked(move |_| {
        show_field_mapping_dialog(
            &ui_for_map.window,
            &headers,
            &canonical,
            &label,
            current_value.as_deref(),
            &state_for_map,
            &ui_for_map,
        );
    });
    value_row.append(&map_button);
    chip.append(&value_row);

    chip
}
