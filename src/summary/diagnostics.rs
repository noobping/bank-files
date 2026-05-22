use super::*;

pub fn render_debug(data: &AppData) -> String {
    let mut out = String::new();
    out.push_str("Diagnostics and import quality\n");
    out.push_str("==============================\n\n");

    if data.reports.is_empty() {
        out.push_str("No CSV files opened.\n");
    }

    for report in &data.reports {
        let name = report
            .source
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| report.source.display().to_string());
        out.push_str(&format!(
            "File: {}\nDelimiter: {:?}\nRows seen: {} · imported: {} · skipped: {}\n",
            name, report.delimiter, report.rows_seen, report.rows_imported, report.rows_skipped
        ));
        out.push_str("Detected fields:\n");
        for line in field_debug_lines(&report.guessed_fields) {
            out.push_str("  ");
            out.push_str(&line);
            out.push('\n');
        }
        if !report.errors.is_empty() {
            out.push_str("Errors/samples:\n");
            for err in &report.errors {
                out.push_str("  - ");
                out.push_str(err);
                out.push('\n');
            }
        }
        out.push('\n');
    }

    if !data.warnings.is_empty() {
        out.push_str("Warnings\n--------\n");
        for warning in &data.warnings {
            out.push_str("- ");
            out.push_str(warning);
            out.push('\n');
        }
    }

    out
}

fn field_debug_lines(fields: &FieldMap) -> Vec<String> {
    field_debug_items(fields)
        .into_iter()
        .map(|(label, value)| format!("{}: {}", gettext(label), value.unwrap_or("-")))
        .collect()
}

fn field_debug_items(fields: &FieldMap) -> [(&'static str, Option<&str>); 11] {
    [
        ("date", fields.date.as_deref()),
        ("amount", fields.amount.as_deref()),
        ("debit", fields.debit.as_deref()),
        ("credit", fields.credit.as_deref()),
        ("description", fields.description.as_deref()),
        ("counterparty", fields.counterparty.as_deref()),
        ("tags", fields.tags.as_deref()),
        ("account", fields.account.as_deref()),
        ("transaction id", fields.transaction_id.as_deref()),
        ("currency", fields.currency.as_deref()),
        ("direction", fields.direction.as_deref()),
    ]
}
