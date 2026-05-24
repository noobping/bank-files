use super::*;

pub(super) fn append_reports_section(
    data: &AppData,
    search: Option<&SearchFilter>,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> bool {
    if data.reports.is_empty() && search.is_none() {
        ui_handles.debug.append(&empty_page(
            "dialog-information-symbolic",
            "No CSV files opened",
            "Choose CSV files or drop bank files onto the window to see import diagnostics.",
        ));
        return false;
    }

    let report_section_matches = search.map(import_report_section_matches).unwrap_or(true);
    let fields_visibility = import_report_field_visibility(search);
    let reports = data
        .reports
        .iter()
        .filter(|report| {
            report_section_matches
                || search
                    .map(|filter| import_report_matches(report, filter))
                    .unwrap_or(true)
        })
        .collect::<Vec<_>>();
    if reports.is_empty() {
        return false;
    }

    let reload_all_button = ui::plain_text_icon_button(
        "view-refresh-symbolic",
        "Reload All",
        "Force reload all CSV files",
    );
    reload_all_button.set_action_name(Some("app.reload-all"));
    register_loading_sensitive_widget(ui_handles, &reload_all_button);
    ui_handles.debug.append(&ui::section_title_with_action(
        "CSV files",
        "These are remembered app copies or live CSV files for this session. Removing a live CSV only forgets it for this session.",
        &reload_all_button,
    ));
    let files = gtk::Box::new(gtk::Orientation::Vertical, 8);
    for report in reports {
        files.append(&diagnostic_file_card(
            report,
            state,
            ui_handles,
            fields_visibility,
        ));
    }
    ui_handles.debug.append(&files);
    true
}

fn import_report_section_matches(filter: &SearchFilter) -> bool {
    filter.matches(
        "csv files imports import reports field mappings detected fields guessed fields headers",
    )
}

fn import_report_field_visibility(search: Option<&SearchFilter>) -> DetectedFieldsVisibility {
    search
        .map(|filter| {
            let query = filter.raw.trim();
            if query.eq_ignore_ascii_case("fields") || query.eq_ignore_ascii_case("field mappings")
            {
                DetectedFieldsVisibility::Expanded
            } else if query.eq_ignore_ascii_case("imports")
                || query.eq_ignore_ascii_case("import reports")
            {
                DetectedFieldsVisibility::Collapsed
            } else {
                DetectedFieldsVisibility::FollowShowAll
            }
        })
        .unwrap_or(DetectedFieldsVisibility::FollowShowAll)
}

pub(in crate::app) fn import_report_matches(report: &ImportReport, filter: &SearchFilter) -> bool {
    let field_text = diagnostic_field_items(&report.guessed_fields)
        .into_iter()
        .map(|field| format!("{} {}", field.label, field.value.unwrap_or("Not detected")))
        .collect::<Vec<_>>()
        .join(" ");
    let header_text = report.headers.join(" ");
    let error_text = report.errors.join(" ");
    filter.matches(&format!(
        "{} {} {} {} {} {} {} {} {}",
        report.source.display(),
        delimiter_label(report.delimiter),
        header_text,
        report.rows_seen,
        report.rows_imported,
        report.rows_skipped,
        diagnostic_error_text(report.errors.len()),
        field_text,
        error_text,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_and_field_presets_match_import_report_section() {
        let import_search = SearchFilter::from_text("imports").unwrap();
        let field_search = SearchFilter::from_text("fields").unwrap();
        let unrelated_search = SearchFilter::from_text("groceries").unwrap();

        assert!(import_report_section_matches(&import_search));
        assert!(import_report_section_matches(&field_search));
        assert!(!import_report_section_matches(&unrelated_search));
    }

    #[test]
    fn import_and_field_presets_control_detected_field_visibility() {
        let import_search = SearchFilter::from_text("imports").unwrap();
        let import_report_search = SearchFilter::from_text("Import Reports").unwrap();
        let field_search = SearchFilter::from_text("fields").unwrap();
        let field_mapping_search = SearchFilter::from_text("Field Mappings").unwrap();
        let unrelated_search = SearchFilter::from_text("groceries").unwrap();

        assert_eq!(
            import_report_field_visibility(Some(&import_search)),
            DetectedFieldsVisibility::Collapsed
        );
        assert_eq!(
            import_report_field_visibility(Some(&import_report_search)),
            DetectedFieldsVisibility::Collapsed
        );
        assert_eq!(
            import_report_field_visibility(Some(&field_search)),
            DetectedFieldsVisibility::Expanded
        );
        assert_eq!(
            import_report_field_visibility(Some(&field_mapping_search)),
            DetectedFieldsVisibility::Expanded
        );
        assert_eq!(
            import_report_field_visibility(Some(&unrelated_search)),
            DetectedFieldsVisibility::FollowShowAll
        );
        assert_eq!(
            import_report_field_visibility(None),
            DetectedFieldsVisibility::FollowShowAll
        );
    }
}
