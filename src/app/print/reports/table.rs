use super::*;

pub(in crate::app) fn table_print_report(
    title: &str,
    subtitle: &str,
    column_titles: &[String],
    rows: &[Vec<String>],
) -> PrintReport {
    let section = if rows.is_empty() {
        PrintSection::Paragraph {
            title: title.to_string(),
            body: tr("No rows to print."),
        }
    } else {
        PrintSection::Table {
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            columns: column_titles
                .iter()
                .map(|title| PrintColumn {
                    title: tr(title),
                    width: 1.0,
                    align: PrintAlign::Left,
                })
                .collect(),
            rows: rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|value| cell(truncate(value, 90)))
                        .collect::<Vec<_>>()
                })
                .collect(),
        }
    };

    PrintReport {
        title: title.to_string(),
        subtitle: subtitle.to_string(),
        generated: print_generated_at(),
        metrics: Vec::new(),
        sections: vec![section],
    }
}
