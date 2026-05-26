use super::fields::detected_fields_toggle;
use super::file_actions::{
    force_reload_csv_file, forget_or_unload_csv_file, transaction_source_for_report,
};
use super::helpers::{delimiter_label, diagnostic_error_text};
use super::*;

pub(in crate::app) fn diagnostic_file_card(
    report: &ImportReport,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    fields_visibility: DetectedFieldsVisibility,
) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 10);
    card.add_css_class("card");
    card.set_margin_top(4);
    card.set_margin_bottom(4);
    card.set_margin_start(4);
    card.set_margin_end(4);
    let content = gtk::Box::new(gtk::Orientation::Vertical, 10);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    card.append(&content);

    let heading = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let icon = gtk::Image::from_icon_name("text-x-generic-symbolic");
    icon.set_valign(gtk::Align::Start);
    heading.append(&icon);

    let text = gtk::Box::new(gtk::Orientation::Vertical, 3);
    text.set_hexpand(true);
    let name = report
        .source
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| report.source.display().to_string());
    let title = ui::wrapped_label(&name);
    title.add_css_class("heading");
    let subtitle = ui::wrapped_label(&format!(
        "{} · {} · {}",
        delimiter_label(report.delimiter),
        diagnostic_error_text(report.errors.len()),
        format_size(file_size(&report.source))
    ));
    subtitle.add_css_class("dim-label");
    text.append(&title);
    text.append(&subtitle);
    heading.append(&text);

    let count = gtk::Label::new(Some(&format!(
        "{} / {}",
        report.rows_imported, report.rows_seen
    )));
    count.add_css_class("caption");
    count.set_valign(gtk::Align::Start);
    heading.append(&count);

    let source = transaction_source_for_report(report, &state.borrow());
    let live_source = source.is_live();

    let file_actions = ui::linked_button_group();
    file_actions.set_valign(gtk::Align::Start);
    let reload_button = ui::icon_button(
        "view-refresh-symbolic",
        if live_source {
            "Force reload live bank file"
        } else {
            "Force reload stored CSV"
        },
    );
    reload_button.add_css_class("flat");
    register_loading_sensitive_widget(ui_handles, &reload_button);
    let unload_button = ui::icon_button(
        "user-trash-symbolic",
        if live_source {
            "Forget live bank file for this session"
        } else {
            "Unload stored CSV"
        },
    );
    unload_button.add_css_class("destructive-action");
    unload_button.add_css_class("flat");
    register_loading_sensitive_widget(ui_handles, &unload_button);
    if !live_source {
        match data_write_availability(ui_handles.as_ref()) {
            ActionAvailability::Available => {}
            availability => apply_action_availability(&unload_button, &availability),
        }
    }

    let source_for_reload = source.clone();
    let name_for_reload = name.clone();
    let state_for_reload = Rc::clone(state);
    let ui_for_reload = Rc::clone(ui_handles);
    reload_button.connect_clicked(move |button| {
        force_reload_csv_file(
            &source_for_reload,
            &name_for_reload,
            &state_for_reload,
            &ui_for_reload,
            button,
        );
    });

    let source_for_unload = source.clone();
    let name_for_unload = name.clone();
    let state_for_unload = Rc::clone(state);
    let ui_for_unload = Rc::clone(ui_handles);
    unload_button.connect_clicked(move |button| {
        forget_or_unload_csv_file(
            &source_for_unload,
            &name_for_unload,
            &state_for_unload,
            &ui_for_unload,
            button,
        );
    });
    file_actions.append(&reload_button);
    file_actions.append(&unload_button);
    heading.append(&file_actions);
    content.append(&heading);

    let progress = gtk::ProgressBar::new();
    let fraction = if report.rows_seen == 0 {
        0.0
    } else {
        report.rows_imported as f64 / report.rows_seen as f64
    };
    progress.set_fraction(fraction.clamp(0.0, 1.0));
    content.append(&progress);

    let row_text = format!(
        "{} usable · {} skipped",
        report.rows_imported, report.rows_skipped
    );
    let row_label = ui::wrapped_label(&row_text);
    row_label.add_css_class("dim-label");
    content.append(&row_label);

    content.append(&detected_fields_toggle(
        report,
        state,
        ui_handles,
        fields_visibility,
    ));

    card
}
