use super::*;
use calamine::{open_workbook_auto, Data, Reader};

pub(crate) const TRANSACTION_IMPORT_EXTENSIONS: &[&str] =
    &["csv", "ods", "xls", "xlsb", "xlsm", "xlsx"];

pub(in crate::data) fn is_spreadsheet(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            TRANSACTION_IMPORT_EXTENSIONS[1..]
                .iter()
                .any(|supported| extension.eq_ignore_ascii_case(supported))
        })
        .unwrap_or(false)
}

pub(crate) fn is_transaction_import_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            TRANSACTION_IMPORT_EXTENSIONS
                .iter()
                .any(|supported| extension.eq_ignore_ascii_case(supported))
        })
        .unwrap_or(false)
}

pub(in crate::data) fn convert_spreadsheet_to_csv_files(
    source: &Path,
    target_dir: &Path,
) -> Result<Vec<PathBuf>> {
    fs::create_dir_all(target_dir).with_context(|| {
        format!(
            "Could not create spreadsheet conversion folder: {}",
            target_dir.display()
        )
    })?;

    let mut workbook = open_workbook_auto(source)
        .with_context(|| format!("Could not open spreadsheet: {}", source.display()))?;
    let sheet_names = workbook.sheet_names().to_vec();
    let multiple_sheets = sheet_names.len() > 1;
    let mut converted = Vec::new();

    for (index, sheet_name) in sheet_names.iter().enumerate() {
        let range = workbook.worksheet_range(sheet_name).with_context(|| {
            format!(
                "Could not read sheet “{sheet_name}” from {}",
                source.display()
            )
        })?;
        if sheet_is_empty(&range) {
            continue;
        }

        let file_name = converted_sheet_file_name(source, sheet_name, index, multiple_sheets);
        let target = super::copy::unique_inbox_target(target_dir, Path::new(&file_name));
        write_range_as_csv(&range, &target).with_context(|| {
            format!(
                "Could not convert sheet “{sheet_name}” to {}",
                target.display()
            )
        })?;
        mark_transaction_csv_readonly(&target)?;
        converted.push(target);
    }

    Ok(converted)
}

pub(in crate::data) fn live_spreadsheet_csv_dir() -> Result<PathBuf> {
    let root = app_cache_dir()?
        .join("spreadsheet-csv")
        .join(std::process::id().to_string());
    fs::create_dir_all(&root).with_context(|| {
        format!(
            "Could not create temporary spreadsheet CSV folder: {}",
            root.display()
        )
    })?;
    Ok(root)
}

fn sheet_is_empty(range: &calamine::Range<Data>) -> bool {
    range
        .rows()
        .all(|row| row.iter().all(|cell| matches!(cell, Data::Empty)))
}

fn write_range_as_csv(range: &calamine::Range<Data>, target: &Path) -> Result<()> {
    let mut writer = csv::WriterBuilder::new().from_path(target)?;
    for row in range.rows() {
        let cells = row.iter().map(cell_to_string).collect::<Vec<_>>();
        writer.write_record(cells)?;
    }
    writer.flush()?;
    Ok(())
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.clone(),
        Data::Float(value) => spreadsheet_float_to_string(*value),
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) | Data::DurationIso(value) => value.clone(),
        Data::Error(value) => value.to_string(),
    }
}

fn spreadsheet_float_to_string(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn converted_sheet_file_name(
    source: &Path,
    sheet_name: &str,
    index: usize,
    multiple_sheets: bool,
) -> String {
    let stem = source
        .file_stem()
        .map(|name| sanitize_file_stem(&name.to_string_lossy()))
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "spreadsheet".to_string());
    if multiple_sheets {
        let sheet = sanitize_file_stem(sheet_name);
        format!("{stem}-{:02}-{sheet}.csv", index + 1)
    } else {
        format!("{stem}.csv")
    }
}

fn sanitize_file_stem(value: &str) -> String {
    let mut sanitized = String::new();
    let mut previous_dash = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            sanitized.push(character.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash {
            sanitized.push('-');
            previous_dash = true;
        }
    }
    sanitized.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spreadsheet_extensions_are_supported_case_insensitively() {
        for extension in ["ods", "xls", "xlsb", "xlsm", "xlsx", "XLSX"] {
            assert!(is_spreadsheet(Path::new(&format!("bank.{extension}"))));
        }
        assert!(!is_spreadsheet(Path::new("bank.csv")));
        assert!(!is_spreadsheet(Path::new("bank.txt")));
        assert!(is_transaction_import_file(Path::new("bank.csv")));
        assert!(is_transaction_import_file(Path::new("bank.xlsx")));
        assert!(!is_transaction_import_file(Path::new("bank.txt")));
    }

    #[test]
    fn empty_ranges_are_skipped() {
        assert!(sheet_is_empty(&calamine::Range::<Data>::empty()));
    }

    #[test]
    fn ranges_are_written_as_csv() {
        let range = calamine::Range::from_sparse(vec![
            calamine::Cell::new((0, 0), Data::String("Date".to_string())),
            calamine::Cell::new((0, 1), Data::String("Amount".to_string())),
            calamine::Cell::new((0, 2), Data::String("Description".to_string())),
            calamine::Cell::new((1, 0), Data::String("2026-01-01".to_string())),
            calamine::Cell::new((1, 1), Data::Float(-12.5)),
            calamine::Cell::new((1, 2), Data::String("Cafe".to_string())),
        ]);
        let target = std::env::temp_dir().join(format!(
            "bank-files-spreadsheet-test-{}.csv",
            std::process::id()
        ));

        write_range_as_csv(&range, &target).expect("range should write as csv");

        let contents = fs::read_to_string(&target).expect("csv should be readable");
        let _ = fs::remove_file(target);
        assert!(contents.contains("Date,Amount,Description"));
        assert!(contents.contains("2026-01-01,-12.5,Cafe"));
    }

    #[test]
    fn converted_sheet_names_are_stable_and_safe() {
        assert_eq!(
            converted_sheet_file_name(Path::new("My Bank Export.xlsx"), "Sheet 1", 0, true),
            "my-bank-export-01-sheet-1.csv"
        );
        assert_eq!(
            converted_sheet_file_name(Path::new("My Bank Export.xlsx"), "Sheet 1", 0, false),
            "my-bank-export.csv"
        );
    }
}
