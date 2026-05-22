use super::mapping::{diagnostic_field_items, show_field_mapping_dialog, DiagnosticField};
use super::*;

const TRANSACTION_PATTERN_VALUE_PREVIEW_LIMIT: usize = 6;

pub(in crate::app) fn diagnostic_file_card(
    report: &ImportReport,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
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

    let file_actions = ui::linked_button_group();
    file_actions.set_valign(gtk::Align::Start);
    let reload_button = ui::icon_button("view-refresh-symbolic", "Force reload stored CSV");
    reload_button.add_css_class("flat");
    register_loading_sensitive_widget(ui_handles, &reload_button);
    let unload_button = ui::icon_button("user-trash-symbolic", "Unload stored CSV");
    unload_button.add_css_class("destructive-action");
    unload_button.add_css_class("flat");
    register_loading_sensitive_widget(ui_handles, &unload_button);
    match data_write_availability(ui_handles.as_ref()) {
        ActionAvailability::Available => {}
        availability => apply_action_availability(&unload_button, &availability),
    }

    let path_for_reload = report.source.clone();
    let name_for_reload = name.clone();
    let state_for_reload = Rc::clone(state);
    let ui_for_reload = Rc::clone(ui_handles);
    reload_button.connect_clicked(move |button| {
        force_reload_csv_file(
            &path_for_reload,
            &name_for_reload,
            &state_for_reload,
            &ui_for_reload,
            button,
        );
    });

    let path_for_unload = report.source.clone();
    let name_for_unload = name.clone();
    let state_for_unload = Rc::clone(state);
    let ui_for_unload = Rc::clone(ui_handles);
    unload_button.connect_clicked(move |button| {
        if !csv_file_action_available(ui_for_unload.loading_count.get()) {
            show_status(&ui_for_unload, "Data is still loading.");
            return;
        }
        if !ui_for_unload.storage_capabilities.borrow().data_writable {
            show_status(
                &ui_for_unload,
                ui_for_unload
                    .storage_capabilities
                    .borrow()
                    .data_write_reason(),
            );
            return;
        }
        let button = button.clone();
        let path = path_for_unload.clone();
        let name = name_for_unload.clone();
        let mode = state_for_unload.borrow().dedupe_mode;
        let auto_clean_config = ui_for_unload.preferences.auto_clean_config();
        let scope =
            current_transaction_load_scope(&state_for_unload.borrow(), ui_for_unload.as_ref());
        let state_for_unload = Rc::clone(&state_for_unload);
        let ui_for_unload = Rc::clone(&ui_for_unload);
        button.set_sensitive(false);
        show_status(&ui_for_unload, "CSV unloaded. Updating the overview...");

        gtk::glib::MainContext::default().spawn_local(async move {
            let task = gtk::gio::spawn_blocking(move || {
                data::remove_inbox_file(&path)?;
                data::load_app_data_read_only_aware(mode, auto_clean_config, scope)
            });

            match task.await {
                Ok(Ok((new_data, capabilities))) => {
                    *state_for_unload.borrow_mut() = new_data;
                    set_storage_capabilities(&ui_for_unload, capabilities);
                    request_render_views(&ui_for_unload, &state_for_unload);
                    refresh_menu(&ui_for_unload, &state_for_unload.borrow());
                    show_status(
                        &ui_for_unload,
                        &trf(
                            "{name} was unloaded. The original CSV remains where you chose it.",
                            &[("name", name.clone())],
                        ),
                    );
                }
                Ok(Err(err)) => {
                    show_status(
                        &ui_for_unload,
                        &trf(
                            "Could not unload {name}: {error}",
                            &[("name", name.clone()), ("error", format!("{err:#}"))],
                        ),
                    );
                    button.set_sensitive(true);
                }
                Err(_) => {
                    show_status(
                        &ui_for_unload,
                        "CSV unload canceled: the background task stopped unexpectedly.",
                    );
                    button.set_sensitive(true);
                }
            }
        });
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

    content.append(&detected_fields_toggle(report, state, ui_handles));

    card
}

fn force_reload_csv_file(
    path: &std::path::Path,
    name: &str,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    button: &gtk::Button,
) {
    if !csv_file_action_available(ui_handles.loading_count.get()) {
        show_status(ui_handles, "Data is still loading.");
        return;
    }

    let path = path.to_path_buf();
    let name = name.to_string();
    let mode = state.borrow().dedupe_mode;
    let auto_clean_config = ui_handles.preferences.auto_clean_config();
    let data = state.borrow().clone();
    let state_for_reload = Rc::clone(state);
    let ui_for_reload = Rc::clone(ui_handles);
    let button = button.clone();
    button.set_sensitive(false);
    show_status(&ui_for_reload, "Force reloading CSV file...");
    begin_background_operation(ui_for_reload.as_ref());

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            data::reload_inbox_file(data, &path, mode, auto_clean_config)
        });

        match task.await {
            Ok(Ok(new_data)) => {
                *state_for_reload.borrow_mut() = new_data;
                set_storage_capabilities(&ui_for_reload, data::current_storage_capabilities());
                render_views(
                    &state_for_reload.borrow(),
                    &ui_for_reload,
                    &state_for_reload,
                );
                refresh_menu(&ui_for_reload, &state_for_reload.borrow());
                show_status(
                    &ui_for_reload,
                    &trf(
                        "{name} was reloaded from app storage.",
                        &[("name", name.clone())],
                    ),
                );
            }
            Ok(Err(error)) => {
                show_status(
                    &ui_for_reload,
                    &trf(
                        "Could not reload {name}: {error}",
                        &[("name", name.clone()), ("error", format!("{error:#}"))],
                    ),
                );
                button.set_sensitive(true);
            }
            Err(_) => {
                show_status(
                    &ui_for_reload,
                    "CSV reload canceled: the background task stopped unexpectedly.",
                );
                button.set_sensitive(true);
            }
        }
        finish_background_operation(ui_for_reload.as_ref());
    });
}

fn csv_file_action_available(loading_count: u32) -> bool {
    loading_count == 0
}

pub(in crate::app) fn detected_fields_toggle(
    report: &ImportReport,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let reveal_initially = ui_handles.show_all.get();
    let button = gtk::Button::builder()
        .tooltip_text(tr(if reveal_initially {
            "Hide detected fields"
        } else {
            "Show detected fields"
        }))
        .build();
    button.add_css_class("flat");

    let button_content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
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
    button.set_child(Some(&button_content));
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

pub(in crate::app) fn transaction_pattern_card<F>(
    pattern: &analytics::TransactionPattern,
    badges: &[String],
    on_activate: F,
) -> gtk::Box
where
    F: Fn() + 'static,
{
    let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
    card.add_css_class("card");
    card.set_margin_top(4);
    card.set_margin_bottom(4);
    card.set_margin_start(4);
    card.set_margin_end(4);
    card.set_hexpand(true);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    header.set_hexpand(true);

    let icon = gtk::Image::from_icon_name(transaction_pattern_icon(pattern.kind));
    icon.add_css_class("dim-label");
    icon.set_valign(gtk::Align::Start);
    header.append(&icon);

    let title_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    title_box.set_hexpand(true);

    let title_label = pattern_label(&transaction_pattern_title(pattern));
    title_label.add_css_class("heading");
    title_box.append(&title_label);

    let label = pattern_label(&pattern.label);
    label.add_css_class("dim-label");
    title_box.append(&label);
    header.append(&title_box);

    let count = gtk::Label::new(Some(&pattern.count.to_string()));
    count.add_css_class("title-3");
    count.set_valign(gtk::Align::Start);
    count.set_tooltip_text(Some(&tr("Transactions")));
    header.append(&count);
    content.append(&header);

    if !badges.is_empty() {
        let badge_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        for badge in badges {
            let badge_label = gtk::Label::new(Some(badge));
            badge_label.add_css_class("caption");
            badge_label.add_css_class("dim-label");
            badge_label.set_xalign(0.0);
            badge_box.append(&badge_label);
        }
        content.append(&badge_box);
    }

    let period = trf(
        "{count} transactions total from {first} to {last}.",
        &[
            ("count", pattern.count.to_string()),
            ("first", pattern.first_date.to_string()),
            ("last", pattern.last_date.to_string()),
        ],
    );
    let period_label = pattern_label(&period);
    period_label.add_css_class("caption");
    content.append(&period_label);

    let values = pattern_label(&transaction_pattern_value_stats(pattern));
    values.add_css_class("caption");
    values.add_css_class("dim-label");
    content.append(&values);

    if matches!(
        pattern.kind,
        analytics::TransactionPatternKind::FullRefund
            | analytics::TransactionPatternKind::BillSplit
    ) {
        let net = pattern_label(&trf(
            "Net effect: {net}.",
            &[("net", signed_money(pattern.net))],
        ));
        net.add_css_class("caption");
        net.add_css_class(if pattern.net == rust_decimal::Decimal::ZERO {
            "dim-label"
        } else if pattern.net > Decimal::ZERO {
            "success"
        } else {
            "error"
        });
        content.append(&net);
    }

    card.append(&content);
    ui::activatable_card(card, on_activate)
}

fn pattern_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label.set_width_chars(1);
    label.set_max_width_chars(72);
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    label
}

fn transaction_pattern_icon(kind: analytics::TransactionPatternKind) -> &'static str {
    match kind {
        analytics::TransactionPatternKind::Repeating(_) => "view-refresh-symbolic",
        analytics::TransactionPatternKind::FullRefund => "edit-undo-symbolic",
        analytics::TransactionPatternKind::BillSplit => "view-list-symbolic",
        analytics::TransactionPatternKind::Transfer => "folder-transfer-symbolic",
    }
}

fn transaction_pattern_value_stats(pattern: &analytics::TransactionPattern) -> String {
    let mut values = pattern
        .amount_stats
        .iter()
        .take(TRANSACTION_PATTERN_VALUE_PREVIEW_LIMIT)
        .map(|stat| {
            trf(
                "{count} times {amount}",
                &[
                    ("count", stat.count.to_string()),
                    ("amount", money(stat.amount)),
                ],
            )
        })
        .collect::<Vec<_>>();
    let hidden = pattern
        .amount_stats
        .len()
        .saturating_sub(TRANSACTION_PATTERN_VALUE_PREVIEW_LIMIT);
    if hidden > 0 {
        values.push(trf(
            "{count} more value groups",
            &[("count", hidden.to_string())],
        ));
    }
    trf("Values: {values}", &[("values", values.join(", "))])
}

pub(in crate::app) fn transaction_pattern_matches(
    pattern: &analytics::TransactionPattern,
    filter: &SearchFilter,
) -> bool {
    filter.matches(&format!(
        "{} {} {} {} {} {} {} {}",
        transaction_pattern_title(pattern),
        pattern.label,
        pattern.count,
        money(pattern.amount),
        pattern
            .amount_stats
            .iter()
            .map(|stat| format!("{} {}", stat.count, money(stat.amount)))
            .collect::<Vec<_>>()
            .join(" "),
        signed_money(pattern.net),
        pattern.first_date,
        pattern.last_date,
    ))
}

fn transaction_pattern_title(pattern: &analytics::TransactionPattern) -> String {
    match pattern.kind {
        analytics::TransactionPatternKind::Repeating(cadence) => trf(
            "Repeating {cadence} transaction",
            &[("cadence", transaction_pattern_cadence(cadence))],
        ),
        analytics::TransactionPatternKind::FullRefund => tr("Possible refund"),
        analytics::TransactionPatternKind::BillSplit => tr("Possible offsetting group"),
        analytics::TransactionPatternKind::Transfer => tr("Possible transfer"),
    }
}

fn transaction_pattern_cadence(cadence: analytics::RepeatingCadence) -> String {
    tr(match cadence {
        analytics::RepeatingCadence::Weekly => "weekly",
        analytics::RepeatingCadence::Biweekly => "every two weeks",
        analytics::RepeatingCadence::Monthly => "monthly",
        analytics::RepeatingCadence::Quarterly => "quarterly",
        analytics::RepeatingCadence::Yearly => "yearly",
        analytics::RepeatingCadence::Recurring => "recurring",
    })
}

pub(in crate::app) fn delimiter_label(delimiter: char) -> String {
    match delimiter {
        ';' => tr("Semicolon delimiter (;)"),
        ',' => tr("Comma delimiter (,)"),
        '\t' => tr("Tab delimiter"),
        other => trf(
            "Delimiter {delimiter}",
            &[("delimiter", format!("{other:?}"))],
        ),
    }
}

pub(in crate::app) fn diagnostic_error_text(errors: usize) -> String {
    match errors {
        0 => tr("no sample errors"),
        1 => tr("1 sample error"),
        count => trf("{count} sample errors", &[("count", count.to_string())]),
    }
}

pub(in crate::app) fn empty_page(
    icon_name: &str,
    title: &str,
    description: &str,
) -> adw::StatusPage {
    adw::StatusPage::builder()
        .icon_name(icon_name)
        .title(tr(title))
        .description(tr(description))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_file_actions_are_disabled_while_loading() {
        assert!(csv_file_action_available(0));
        assert!(!csv_file_action_available(1));
        assert!(!csv_file_action_available(3));
    }
}
