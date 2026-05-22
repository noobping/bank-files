use super::*;
use anyhow::anyhow;
use chrono::{Datelike, NaiveDate};
use std::ops::Range;

#[derive(Debug, Clone)]
struct CsvPlan {
    path: PathBuf,
    delimiter: char,
    headers: Vec<String>,
    field_map: FieldMap,
    available_months: Vec<MonthKey>,
    sort_order: DateSortOrder,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum DateSortOrder {
    Ascending,
    Descending,
    Unsorted,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
enum ScopeBounds {
    All,
    Empty,
    Window { start: NaiveDate, end: NaiveDate },
}

impl ScopeBounds {
    fn contains(self, date: NaiveDate) -> bool {
        match self {
            Self::All => true,
            Self::Empty => false,
            Self::Window { start, end } => date >= start && date < end,
        }
    }
}

pub fn import_inbox(dirs: &AppDirs, scope: TransactionLoadScope) -> Result<ImportOutcome> {
    let aliases = FieldAliases::load(&dirs.config)?;
    let mut files: Vec<PathBuf> = Vec::new();
    if dirs.inbox.exists() {
        for entry in WalkDir::new(&dirs.inbox).max_depth(1).into_iter().flatten() {
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .map(|e| e.eq_ignore_ascii_case("csv"))
                    .unwrap_or(false)
            {
                files.push(path.to_path_buf());
            }
        }
    }
    files.sort();
    import_files(&files, &aliases, scope)
}

pub fn import_files(
    files: &[PathBuf],
    aliases: &FieldAliases,
    requested_scope: TransactionLoadScope,
) -> Result<ImportOutcome> {
    let mut outcome = ImportOutcome::default();
    let mut plans = Vec::new();
    let mut available_months = BTreeSet::new();

    for (index, result) in run_indexed_in_parallel(
        files.len(),
        |index| plan_csv(&files[index], aliases),
        "CSV planning worker stopped unexpectedly",
    )? {
        match result {
            Ok(plan) => {
                available_months.extend(plan.available_months.iter().copied());
                plans.push(plan);
            }
            Err(err) => outcome
                .warnings
                .push(format!("{}: {err:#}", files[index].display())),
        }
    }

    outcome.available_months = available_months.into_iter().collect();
    let default_month = default_month_from_available_months(&outcome.available_months);
    let resolved_scope = requested_scope.resolve(default_month);
    outcome.loaded_scope = resolved_scope;

    for (index, result) in run_indexed_in_parallel(
        plans.len(),
        |index| import_planned_csv(&plans[index], resolved_scope),
        "CSV import worker stopped unexpectedly",
    )? {
        match result {
            Ok((transactions, report)) => {
                outcome.transactions.extend(transactions);
                outcome.reports.push(report);
            }
            Err(err) => outcome
                .warnings
                .push(format!("{}: {err:#}", plans[index].path.display())),
        }
    }

    Ok(outcome)
}

fn run_indexed_in_parallel<T, F>(
    item_count: usize,
    operation: F,
    panic_message: &'static str,
) -> Result<Vec<(usize, Result<T>)>>
where
    T: Send,
    F: Fn(usize) -> Result<T> + Sync,
{
    let worker_count = parallel_worker_count(item_count);
    if worker_count <= 1 {
        return Ok((0..item_count)
            .map(|index| (index, operation(index)))
            .collect());
    }

    std::thread::scope(|scope| {
        let operation = &operation;
        let mut handles = Vec::new();
        for range in parallel_chunk_ranges(item_count, worker_count) {
            handles.push(scope.spawn(move || {
                range
                    .map(|index| (index, operation(index)))
                    .collect::<Vec<_>>()
            }));
        }

        let mut results = Vec::with_capacity(item_count);
        for handle in handles {
            results.extend(handle.join().map_err(|_| anyhow!(panic_message))?);
        }
        results.sort_by_key(|(index, _)| *index);
        Ok(results)
    })
}

fn parallel_worker_count(item_count: usize) -> usize {
    let available = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    item_count.min(available).max(1)
}

pub(super) fn parallel_chunk_ranges(item_count: usize, worker_count: usize) -> Vec<Range<usize>> {
    if item_count == 0 || worker_count == 0 {
        return Vec::new();
    }

    let worker_count = worker_count.min(item_count);
    let base = item_count / worker_count;
    let remainder = item_count % worker_count;
    let mut ranges = Vec::with_capacity(worker_count);
    let mut start = 0;
    for worker_index in 0..worker_count {
        let extra = usize::from(worker_index < remainder);
        let end = start + base + extra;
        ranges.push(start..end);
        start = end;
    }
    ranges
}

fn plan_csv(path: &Path, aliases: &FieldAliases) -> Result<CsvPlan> {
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
    let (available_months, sort_order) =
        scan_available_months(path, delimiter, &headers, &field_map)?;

    Ok(CsvPlan {
        path: path.to_path_buf(),
        delimiter,
        headers,
        field_map,
        available_months,
        sort_order,
    })
}

fn import_planned_csv(
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
        } else if !matches!(bounds, ScopeBounds::All) {
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

fn read_sample_text(path: &Path) -> Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("Could not read file: {}", path.display()))?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(64 * 1024)
        .read_to_end(&mut bytes)
        .with_context(|| format!("Could not read file: {}", path.display()))?;
    Ok(decode_bytes(&bytes))
}

fn csv_reader(path: &Path, delimiter: char) -> Result<csv::Reader<BufReader<File>>> {
    let file =
        File::open(path).with_context(|| format!("Could not read file: {}", path.display()))?;
    Ok(ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .flexible(true)
        .trim(Trim::All)
        .from_reader(BufReader::new(file)))
}

fn scan_available_months(
    path: &Path,
    delimiter: char,
    headers: &[String],
    field_map: &FieldMap,
) -> Result<(Vec<MonthKey>, DateSortOrder)> {
    let Some(date_index) = column_index(headers, &field_map.date) else {
        return Ok((Vec::new(), DateSortOrder::Unknown));
    };
    let mut months = BTreeSet::new();
    let mut last_date = None;
    let mut saw_ascending = false;
    let mut saw_descending = false;
    let mut rdr = csv_reader(path, delimiter)?;
    let _ = rdr.byte_headers().context("CSV has no usable header")?;

    for result in rdr.byte_records() {
        let record = result?;
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
    Ok((months.into_iter().collect(), sort_order))
}

fn column_index(headers: &[String], column: &Option<String>) -> Option<usize> {
    let column = column.as_ref()?;
    headers.iter().position(|header| header == column)
}

fn date_from_byte_record(record: &ByteRecord, index: usize) -> Option<NaiveDate> {
    record
        .get(index)
        .and_then(|cell| parse_date(&decode_bytes(cell)))
}

fn decode_record(record: &ByteRecord) -> Vec<String> {
    record.iter().map(decode_bytes).collect()
}

fn decode_bytes(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        text.to_string()
    } else {
        let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
        cow.into_owned()
    }
}

fn default_month_from_available_months(months: &[MonthKey]) -> Option<MonthKey> {
    let latest = months.last().copied()?;
    let today = chrono::Local::now().date_naive();
    let current = MonthKey::new(today.year(), today.month());
    if months.contains(&current) {
        Some(current.previous())
    } else {
        Some(latest)
    }
}

fn scope_bounds(scope: TransactionLoadScope) -> ScopeBounds {
    match scope {
        TransactionLoadScope::Unloaded => ScopeBounds::Empty,
        TransactionLoadScope::All => ScopeBounds::All,
        TransactionLoadScope::Year(Some(year)) => year_bounds(year, false),
        TransactionLoadScope::YearWithPrevious(Some(year)) => year_bounds(year, true),
        TransactionLoadScope::Month(Some(month)) => month_bounds(month, false),
        TransactionLoadScope::MonthWithPrevious(Some(month)) => month_bounds(month, true),
        TransactionLoadScope::Year(None)
        | TransactionLoadScope::YearWithPrevious(None)
        | TransactionLoadScope::Month(None)
        | TransactionLoadScope::MonthWithPrevious(None) => ScopeBounds::Empty,
    }
}

fn year_bounds(year: i32, include_previous: bool) -> ScopeBounds {
    let start_year = if include_previous { year - 1 } else { year };
    let Some(start) = NaiveDate::from_ymd_opt(start_year, 1, 1) else {
        return ScopeBounds::Empty;
    };
    let Some(end) = NaiveDate::from_ymd_opt(year + 1, 1, 1) else {
        return ScopeBounds::Empty;
    };
    ScopeBounds::Window { start, end }
}

fn month_bounds(month: MonthKey, include_previous: bool) -> ScopeBounds {
    let start_month = if include_previous {
        month.previous()
    } else {
        month
    };
    let end_month = month.next();
    let Some(start) = NaiveDate::from_ymd_opt(start_month.year, start_month.month, 1) else {
        return ScopeBounds::Empty;
    };
    let Some(end) = NaiveDate::from_ymd_opt(end_month.year, end_month.month, 1) else {
        return ScopeBounds::Empty;
    };
    ScopeBounds::Window { start, end }
}

fn should_skip_row(
    date: Option<NaiveDate>,
    bounds: ScopeBounds,
    sort_order: DateSortOrder,
) -> bool {
    let ScopeBounds::Window { start, end } = bounds else {
        return matches!(bounds, ScopeBounds::Empty);
    };
    let Some(date) = date else {
        return false;
    };
    match sort_order {
        DateSortOrder::Ascending => date < start,
        DateSortOrder::Descending => date >= end,
        DateSortOrder::Unsorted | DateSortOrder::Unknown => false,
    }
}

fn should_stop_before_row(
    date: Option<NaiveDate>,
    bounds: ScopeBounds,
    sort_order: DateSortOrder,
) -> bool {
    let ScopeBounds::Window { start, end } = bounds else {
        return false;
    };
    let Some(date) = date else {
        return false;
    };
    match sort_order {
        DateSortOrder::Ascending => date >= end,
        DateSortOrder::Descending => date < start,
        DateSortOrder::Unsorted | DateSortOrder::Unknown => false,
    }
}

fn sniff_delimiter(content: &str) -> char {
    let candidates = [';', ',', '\t', '|'];
    let lines: Vec<&str> = content.lines().take(12).collect();
    let mut best = (';', 0usize);
    for delimiter in candidates {
        let score = lines
            .iter()
            .map(|line| line.matches(delimiter).count())
            .filter(|count| *count > 0)
            .sum::<usize>();
        if score > best.1 {
            best = (delimiter, score);
        }
    }
    best.0
}
