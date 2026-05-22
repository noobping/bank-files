use super::*;

pub(in crate::app) fn diagnostic_report_matches(
    report: &crate::model::ImportReport,
    search: Option<&SearchFilter>,
) -> bool {
    search
        .map(|filter| filter.matches(&diagnostic_report_search_text(report)))
        .unwrap_or(true)
}

fn diagnostic_report_search_text(report: &crate::model::ImportReport) -> String {
    let fields = import_report_field_text(report, " ");

    format!(
        "{} {} {} {} {} {} {}",
        report.source.display(),
        delimiter_label(report.delimiter),
        report.headers.join(" "),
        fields,
        report.rows_seen,
        report.rows_imported,
        report.errors.join(" "),
    )
}
