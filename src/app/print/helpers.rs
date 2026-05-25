use super::*;

pub(in crate::app) fn transaction_print_columns() -> Vec<PrintColumn> {
    columns(&[
        ("Date", 0.7, PrintAlign::Left),
        ("Amount", 0.75, PrintAlign::Right),
        ("Category", 1.0, PrintAlign::Left),
        ("Budget", 0.55, PrintAlign::Left),
        ("Description", 1.75, PrintAlign::Left),
    ])
}

pub(in crate::app) fn transaction_print_row(tx: &Transaction) -> Vec<PrintCell> {
    vec![
        cell(tx.date.to_string()),
        tone_cell(signed_money(tx.amount), tone_for_amount(tx.amount)),
        cell(truncate(&tx.category, 24)),
        cell(truncate(&tx.budget_code, 10)),
        cell(truncate(&tx.description, 48)),
    ]
}

pub(in crate::app) fn columns(spec: &[(&str, f64, PrintAlign)]) -> Vec<PrintColumn> {
    spec.iter()
        .map(|(title, width, align)| PrintColumn {
            title: tr(title),
            width: *width,
            align: *align,
        })
        .collect()
}

pub(in crate::app) fn cell(text: impl Into<String>) -> PrintCell {
    PrintCell {
        text: text.into(),
        tone: PrintTone::Normal,
    }
}

pub(in crate::app) fn tone_cell(text: impl Into<String>, tone: PrintTone) -> PrintCell {
    PrintCell {
        text: text.into(),
        tone,
    }
}

pub(in crate::app) fn metric(
    label: impl Into<String>,
    value: impl Into<String>,
    detail: impl Into<String>,
    tone: PrintTone,
) -> PrintMetric {
    let label = label.into();
    let detail = detail.into();
    PrintMetric {
        label: tr(&label),
        value: value.into(),
        detail: tr(&detail),
        tone,
    }
}

pub(in crate::app) fn tone_for_amount(amount: Decimal) -> PrintTone {
    if amount > Decimal::ZERO {
        PrintTone::Positive
    } else if amount < Decimal::ZERO {
        PrintTone::Negative
    } else {
        PrintTone::Muted
    }
}

pub(in crate::app) fn print_generated_at() -> String {
    trf(
        "Created on {date}",
        &[(
            "date",
            chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
        )],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_print_row_omits_counterparty_and_notes() {
        let tx = Transaction {
            date: chrono::NaiveDate::from_ymd_opt(2025, 4, 12).unwrap(),
            amount: Decimal::new(-1234, 2),
            description: "Monthly supermarket run".to_string(),
            counterparty: "Hidden Counterparty BV".to_string(),
            tags: String::new(),
            account: String::new(),
            transaction_id: String::new(),
            currency: "EUR".to_string(),
            source_file: String::new(),
            source_row: 1,
            category: "Groceries".to_string(),
            budget_code: "FOOD".to_string(),
            notes: "Receipt checked".to_string(),
            strict_key: String::new(),
            loose_key: String::new(),
            rule_match: None,
        };

        let columns = transaction_print_columns();
        let row = transaction_print_row(&tx);

        assert_eq!(columns.len(), row.len());
        assert!(columns
            .iter()
            .all(|column| { column.title != tr("Counterparty") && column.title != tr("Note") }));
        assert!(row.iter().all(|cell| {
            !cell.text.contains("Hidden Counterparty") && !cell.text.contains("Receipt checked")
        }));
    }
}
