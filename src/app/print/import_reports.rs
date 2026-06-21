use crate::model::ImportReport;

pub(in crate::app) fn import_report_field_text(report: &ImportReport, separator: &str) -> String {
    [
        report.guessed_fields.date.as_deref(),
        report.guessed_fields.amount.as_deref(),
        report.guessed_fields.description.as_deref(),
        report.guessed_fields.counterparty.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(separator)
}
