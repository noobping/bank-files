use super::*;

pub fn copy_files_to_app_storage(files: &[PathBuf]) -> Result<CsvCopyResult> {
    let dirs = prepare_import_storage()?;
    let mut outcome = CsvCopyResult::default();

    for file in files {
        let source = adw::gtk::gio::File::for_path(file);
        match copy_gio_file_to_app_storage(&source, &dirs) {
            Ok(CsvCopyTarget::Transactions) => outcome.transaction_csvs += 1,
            Ok(CsvCopyTarget::Config) => outcome.config_csvs += 1,
            Ok(CsvCopyTarget::Skipped) => outcome.skipped += 1,
            Err(err) => {
                return Err(err).with_context(|| format!("Could not import {}", file.display()));
            }
        }
    }
    Ok(outcome)
}

pub fn copy_uris_to_app_storage(uris: &[String]) -> Result<CsvCopyResult> {
    let dirs = prepare_import_storage()?;
    let mut outcome = CsvCopyResult::default();

    for uri in uris {
        let source = adw::gtk::gio::File::for_uri(uri);
        match copy_gio_file_to_app_storage(&source, &dirs) {
            Ok(CsvCopyTarget::Transactions) => outcome.transaction_csvs += 1,
            Ok(CsvCopyTarget::Config) => outcome.config_csvs += 1,
            Ok(CsvCopyTarget::Skipped) => outcome.skipped += 1,
            Err(err) => return Err(err).with_context(|| format!("Could not import {uri}")),
        }
    }
    Ok(outcome)
}

fn prepare_import_storage() -> Result<AppDirs> {
    let dirs = app_dirs()?;
    fs::create_dir_all(&dirs.data)
        .with_context(|| format!("Could not create app data folder: {}", dirs.data.display()))?;
    fs::create_dir_all(&dirs.inbox).with_context(|| {
        format!(
            "Could not create CSV storage folder: {}",
            dirs.inbox.display()
        )
    })?;
    Ok(dirs)
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::data) enum CsvCopyTarget {
    Transactions,
    Config,
    Skipped,
}

pub(in crate::data) fn copy_gio_file_to_app_storage(
    source: &adw::gtk::gio::File,
    dirs: &AppDirs,
) -> Result<CsvCopyTarget> {
    if !source.query_exists(adw::gtk::gio::Cancellable::NONE) {
        return Ok(CsvCopyTarget::Skipped);
    }

    let Some(name) = source.basename() else {
        return Ok(CsvCopyTarget::Skipped);
    };
    if !is_csv(&name) {
        return Ok(CsvCopyTarget::Skipped);
    }

    let temp = unique_temp_csv_target(dirs, &name);
    let destination = adw::gtk::gio::File::for_path(&temp);
    source
        .copy(
            &destination,
            adw::gtk::gio::FileCopyFlags::NONE,
            adw::gtk::gio::Cancellable::NONE,
            None,
        )
        .with_context(|| format!("Could not copy {} to {}", source.uri(), temp.display()))?;

    let target = match config_csv_name(&name).or_else(|| detect_config_csv_file(&temp)) {
        Some(config_name) => {
            fs::create_dir_all(&dirs.config).with_context(|| {
                format!(
                    "Could not create configuration folder: {}",
                    dirs.config.display()
                )
            })?;
            fs::copy(&temp, dirs.config.join(config_name))
                .with_context(|| format!("Could not save configuration CSV: {}", config_name))?;
            CsvCopyTarget::Config
        }
        None => {
            let target = unique_inbox_target(&dirs.inbox, &name);
            fs::rename(&temp, &target).with_context(|| {
                format!("Could not move CSV to app storage: {}", target.display())
            })?;
            mark_transaction_csv_readonly(&target)?;
            return Ok(CsvCopyTarget::Transactions);
        }
    };

    let _ = fs::remove_file(temp);
    Ok(target)
}

pub(in crate::data) fn unique_temp_csv_target(dirs: &AppDirs, name: &Path) -> PathBuf {
    let stem = name
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S%.f");
    dirs.data
        .join(format!(".incoming-{stem}-{}-{ts}.tmp", std::process::id()))
}

pub(in crate::data) fn unique_inbox_target(inbox: &Path, name: &Path) -> PathBuf {
    let mut target = inbox.join(name);
    if target.exists() {
        let stem = name
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());
        let ext = name
            .extension()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "csv".to_string());
        let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
        target = inbox.join(format!("{stem}-{ts}.{ext}"));
    }
    target
}

pub(in crate::data) fn config_csv_name(name: &Path) -> Option<&'static str> {
    match name
        .file_name()?
        .to_string_lossy()
        .to_ascii_lowercase()
        .as_str()
    {
        "rules.csv" => Some("rules.csv"),
        "budgetcodes.csv" | "budgets.csv" => Some("budgetcodes.csv"),
        "field_aliases.csv" | "field-aliases.csv" | "aliases.csv" => Some("field_aliases.csv"),
        _ => None,
    }
}

pub(in crate::data) fn detect_config_csv_file(path: &Path) -> Option<&'static str> {
    config_csv_from_headers(&read_csv_header_sample(path).ok()?)
}

pub(in crate::data) fn config_csv_from_headers(sample: &str) -> Option<&'static str> {
    for delimiter in [b',', b';', b'\t'] {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(sample.as_bytes());
        let Ok(headers) = reader.headers() else {
            continue;
        };
        let normalized = headers
            .iter()
            .map(normalize_key)
            .collect::<HashSet<String>>();

        if has_headers(&normalized, &["canonical", "alias"]) {
            return Some("field_aliases.csv");
        }
        if has_headers(&normalized, &["code", "category", "monthly budget"]) {
            return Some("budgetcodes.csv");
        }
        if has_headers(&normalized, &["pattern", "category", "budget code"]) {
            return Some("rules.csv");
        }
    }

    None
}

pub(in crate::data) fn has_headers(headers: &HashSet<String>, required: &[&str]) -> bool {
    required.iter().all(|header| headers.contains(*header))
}

pub(in crate::data) fn read_csv_header_sample(path: &Path) -> Result<String> {
    let mut file =
        fs::File::open(path).with_context(|| format!("Could not inspect {}", path.display()))?;
    let mut bytes = Vec::new();
    file.by_ref().take(64 * 1024).read_to_end(&mut bytes)?;
    if let Ok(text) = String::from_utf8(bytes.clone()) {
        Ok(text)
    } else {
        let (text, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
        Ok(text.into_owned())
    }
}
