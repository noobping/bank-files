use super::*;

#[derive(Clone, Copy)]
pub(in crate::app) struct DiagnosticField<'a> {
    pub(in crate::app) canonical: &'static str,
    pub(in crate::app) label: &'static str,
    pub(in crate::app) value: Option<&'a str>,
}

pub(in crate::app) fn diagnostic_field_items(fields: &FieldMap) -> [DiagnosticField<'_>; 11] {
    FIELD_ALIAS_SPECS.map(|spec| DiagnosticField {
        canonical: spec.canonical,
        label: spec.label,
        value: field_value(fields, spec.canonical),
    })
}

fn field_value<'a>(fields: &'a FieldMap, canonical: &str) -> Option<&'a str> {
    match canonical {
        "date" => fields.date.as_deref(),
        "amount" => fields.amount.as_deref(),
        "debit" => fields.debit.as_deref(),
        "credit" => fields.credit.as_deref(),
        "description" => fields.description.as_deref(),
        "counterparty" => fields.counterparty.as_deref(),
        "tags" => fields.tags.as_deref(),
        "account" => fields.account.as_deref(),
        "transaction_id" => fields.transaction_id.as_deref(),
        "currency" => fields.currency.as_deref(),
        "direction" => fields.direction.as_deref(),
        _ => None,
    }
}

pub(in crate::app) fn show_field_mapping_dialog(
    parent: &adw::ApplicationWindow,
    headers: &[String],
    canonical: &str,
    label: &str,
    current_value: Option<&str>,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let dialog = field_mapping_dialog(headers, canonical, label, current_value, state, ui_handles);
    dialog.present(Some(parent));
}

fn field_mapping_dialog(
    headers: &[String],
    canonical: &str,
    label: &str,
    current_value: Option<&str>,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> adw::Dialog {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header = ui::cancelable_dialog_header("Map CSV Field", label);

    let save_button =
        ui::primary_text_icon_button("document-save-symbolic", "Save", "Save field mapping");
    save_button.set_sensitive(!headers.is_empty());
    register_config_widget(ui_handles, &save_button);
    header.pack_end(&save_button);
    root.append(&header);

    let page = ui::page_box();
    page.append(&ui::section_title(
        "CSV Field Mapping",
        &trf(
            "Map one of this file's CSV headers to {field}.",
            &[("field", tr(label))],
        ),
    ));

    let field = gtk::ComboBoxText::new();
    for header in headers {
        field.append(Some(header), header);
    }
    if let Some(value) = current_value {
        field.set_active_id(Some(value));
    }
    if field.active_id().is_none() {
        field.set_active(Some(0));
    }

    let grid = ui::form_grid();
    ui::add_labeled(&grid, 0, "App field", &ui::wrapped_label(label));
    ui::add_labeled(&grid, 1, "CSV header", &field);
    page.append(&grid);

    let status_text = if headers.is_empty() {
        tr("This CSV report has no headers to map.")
    } else {
        tr("This is saved to field_aliases.csv and applies to future imports.")
    };
    let status = ui::wrapped_label(&status_text);
    status.add_css_class("dim-label");
    page.append(&status);
    root.append(&ui::scroll(&page));

    let dialog = adw::Dialog::builder()
        .title(tr("Map CSV Field"))
        .content_width(620)
        .content_height(620)
        .default_widget(&save_button)
        .child(&root)
        .build();

    let canonical = canonical.to_string();
    let label = label.to_string();
    let state_for_save = Rc::clone(state);
    let ui_for_save = Rc::clone(ui_handles);
    let dialog_for_save = dialog.clone();
    save_button.connect_clicked(move |button| {
        let alias = field
            .active_id()
            .map(|value| value.to_string())
            .unwrap_or_default();
        if alias.trim().is_empty() {
            status.set_text(&tr("Choose a CSV header first."));
            return;
        }

        let button = button.clone();
        let canonical_for_save = canonical.clone();
        let alias_for_save = alias.clone();
        let alias_for_message = alias.clone();
        let label_for_message = label.clone();
        let borrowed = state_for_save.borrow();
        let mode = borrowed.dedupe_mode;
        let remember_mode = ui_for_save.remember_mode.get();
        let sources = current_sources_for_reload(&borrowed, remember_mode);
        let scope = current_transaction_load_scope(&borrowed, ui_for_save.as_ref());
        drop(borrowed);
        let auto_clean_config = ui_for_save.preferences.auto_clean_config();
        let state_for_save = Rc::clone(&state_for_save);
        let ui_for_save = Rc::clone(&ui_for_save);
        let dialog_for_save = dialog_for_save.clone();
        let status_for_save = status.clone();
        button.set_sensitive(false);
        status_for_save.set_text(&tr("Saving field mapping..."));

        gtk::glib::MainContext::default().spawn_local(async move {
            let task = gtk::gio::spawn_blocking(move || {
                let saved = data::upsert_editable_alias(&canonical_for_save, &alias_for_save)?;
                let new_data = data::load_app_data_with_sources(mode, auto_clean_config, scope, remember_mode, &sources)?.0;
                anyhow::Ok((saved, new_data))
            });

            match task.await {
                Ok(Ok((saved, new_data))) => {
                    *state_for_save.borrow_mut() = new_data;
                    render_views(&state_for_save.borrow(), &ui_for_save, &state_for_save);
                    let success_message = if saved {
                        trf(
                            "{alias} is now mapped to {field}. Reloaded imports use the new field name.",
                            &[
                                ("alias", alias_for_message.clone()),
                                ("field", label_for_message.clone()),
                            ],
                        )
                    } else {
                        trf(
                            "{alias} was already mapped to {field}.",
                            &[
                                ("alias", alias_for_message.clone()),
                                ("field", label_for_message.clone()),
                            ],
                        )
                    };
                    show_status(&ui_for_save, &success_message);
                    dialog_for_save.close();
                }
                Ok(Err(err)) => {
                    status_for_save.set_text(&trf(
                        "Could not save field mapping: {error}",
                        &[("error", format!("{err:#}"))],
                    ));
                    button.set_sensitive(true);
                }
                Err(_) => {
                    status_for_save.set_text(&tr("Field mapping save canceled: the background task stopped unexpectedly."));
                    button.set_sensitive(true);
                }
            }
        });
    });

    dialog
}
