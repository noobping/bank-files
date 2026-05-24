use super::csv_io::{
    column_index, csv_reader, date_from_byte_record, decode_record, read_sample_text,
    sniff_delimiter,
};
use super::scope::{scope_bounds, should_skip_row, should_stop_before_row, DateSortOrder};
use super::*;
use chrono::Datelike;

#[derive(Debug, Clone)]
pub(super) struct CsvPlan {
    pub(super) path: PathBuf,
    delimiter: char,
    headers: Vec<String>,
    field_map: FieldMap,
    pub(super) available_months: Vec<MonthKey>,
    records_total: usize,
    sort_order: DateSortOrder,
}

pub(super) fn plan_csv(path: &Path, aliases: &FieldAliases) -> Result<CsvPlan> {
    let sample_text = read_sample_text(path)?;
    let delimiter = sniff_delimiter(&sample_text);
    let mut rdr = csv_reader(path, delimiter)?;
    let headers_record = rdr
        .byte_headers()
        .context("CSV has no usable header")?
        .clone();
    let headers = decode_record(&headers_record);
    let sample = rdr
        .byte_records()
        .take(50)
        .filter_map(Result::ok)
        .map(|record| decode_record(&record))
        .collect::<Vec<_>>();
    let field_map = guess_field_map(&headers, &sample, aliases);
    let (available_months, records_total, sort_order) =
        scan_available_months(path, delimiter, &headers, &field_map)?;

    Ok(CsvPlan {
        path: path.to_path_buf(),
        delimiter,
        headers,
        field_map,
        available_months,
        records_total,
        sort_order,
    })
}

pub(super) fn import_planned_csv(
    plan: &CsvPlan,
    scope: TransactionLoadScope,
) -> Result<(Vec<Transaction>, ImportReport)> {
    let bounds = scope_bounds(scope);
    let date_index = column_index(&plan.headers, &plan.field_map.date);
    let mut rdr = csv_reader(&plan.path, plan.delimiter)?;
    let _ = rdr.byte_headers().context("CSV has no usable header")?;
    let mut report = ImportReport {
        source: plan.path.clone(),
        delimiter: plan.delimiter,
        headers: plan.headers.clone(),
        records_total: plan.records_total,
        guessed_fields: plan.field_map.clone(),
        ..Default::default()
    };
    let mut transactions = Vec::new();

    for (idx, result) in rdr.byte_records().enumerate() {
        let record = result?;
        report.rows_seen += 1;
        let source_row = idx + 2;
        let row_date = date_index.and_then(|index| date_from_byte_record(&record, index));

        if should_stop_before_row(row_date, bounds, plan.sort_order) {
            break;
        }
        if should_skip_row(row_date, bounds, plan.sort_order) {
            continue;
        }
        if let Some(date) = row_date {
            if !bounds.contains(date) {
                continue;
            }
        } else if !matches!(bounds, super::scope::ScopeBounds::All) {
            continue;
        }

        let decoded = decode_record(&record);
        match parse_record(
            &plan.path,
            &plan.headers,
            &decoded,
            &plan.field_map,
            source_row,
        ) {
            Some(tx) => {
                report.rows_imported += 1;
                transactions.push(tx);
            }
            None => {
                report.rows_skipped += 1;
                if report.errors.len() < 20 {
                    report.errors.push(i18n::format(
                        "Row {row}: missing date or amount",
                        &[("row", source_row.to_string())],
                    ));
                }
            }
        }
    }

    Ok((transactions, report))
}

fn scan_available_months(
    path: &Path,
    delimiter: char,
    headers: &[String],
    field_map: &FieldMap,
) -> Result<(Vec<MonthKey>, usize, DateSortOrder)> {
    let Some(date_index) = column_index(headers, &field_map.date) else {
        return Ok((Vec::new(), 0, DateSortOrder::Unknown));
    };
    let mut months = BTreeSet::new();
    let mut records_total = 0;
    let mut last_date = None;
    let mut saw_ascending = false;
    let mut saw_descending = false;
    let mut rdr = csv_reader(path, delimiter)?;
    let _ = rdr.byte_headers().context("CSV has no usable header")?;

    for result in rdr.byte_records() {
        let record = result?;
        records_total += 1;
        let Some(date) = date_from_byte_record(&record, date_index) else {
            continue;
        };
        months.insert(MonthKey::new(date.year(), date.month()));
        if let Some(previous) = last_date {
            if date > previous {
                saw_ascending = true;
            } else if date < previous {
                saw_descending = true;
            }
        }
        last_date = Some(date);
    }

    let sort_order = match (saw_ascending, saw_descending) {
        (true, false) => DateSortOrder::Ascending,
        (false, true) => DateSortOrder::Descending,
        (false, false) => DateSortOrder::Unknown,
        (true, true) => DateSortOrder::Unsorted,
    };
    Ok((months.into_iter().collect(), records_total, sort_order))
}
