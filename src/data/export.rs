use super::*;

pub fn export_transactions_to_path(transactions: &[Transaction], path: &Path) -> Result<PathBuf> {
    let mut wtr = csv::WriterBuilder::new()
        .from_path(path)
        .with_context(|| format!("Could not write export: {}", path.display()))?;
    for tx in transactions {
        wtr.serialize(tx)
            .with_context(|| format!("Could not export transaction to {}", path.display()))?;
    }
    wtr.flush()
        .with_context(|| format!("Could not finish export: {}", path.display()))?;
    Ok(path.to_path_buf())
}

pub fn remove_inbox_file(path: &Path) -> Result<()> {
    let dirs = app_dirs()?;
    remove_inbox_file_with_dirs(path, &dirs)
}

fn remove_inbox_file_with_dirs(path: &Path, dirs: &AppDirs) -> Result<()> {
    let inbox = dirs
        .inbox
        .canonicalize()
        .with_context(|| format!("Could not find app CSV folder: {}", dirs.inbox.display()))?;
    let file = path
        .canonicalize()
        .with_context(|| format!("Could not find CSV: {}", path.display()))?;

    if !file.starts_with(&inbox) {
        bail!(
            "This file is not in app storage and will not be removed: {}",
            path.display()
        );
    }

    if !is_csv(&file) {
        bail!("Only CSV files can be unloaded");
    }

    if let Ok(metadata) = fs::metadata(&file) {
        let mut permissions = metadata.permissions();
        if permissions.readonly() {
            permissions.set_readonly(false);
            fs::set_permissions(&file, permissions).with_context(|| {
                format!("Could not prepare CSV for removal: {}", file.display())
            })?;
        }
    }

    fs::remove_file(&file).with_context(|| format!("Could not remove CSV: {}", file.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn remove_inbox_file_removes_readonly_csv_when_data_folder_is_writable() {
        let root = unique_test_dir("remove-readonly-csv");
        let dirs = AppDirs {
            config: root.join("config"),
            data: root.join("data"),
            inbox: root.join("data"),
        };
        ensure_layout(&dirs).expect("test app dirs should be created");
        let csv = dirs.inbox.join("transactions.csv");
        fs::write(&csv, "Date,Description,Amount\n2026-01-01,Coffee,-2.50\n")
            .expect("test csv should be written");
        mark_transaction_csv_readonly(&csv).expect("test csv should be read-only");

        remove_inbox_file_with_dirs(&csv, &dirs).expect("readonly csv should be removed");

        assert!(!csv.exists());
        fs::remove_dir_all(root).ok();
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("bank-files-{name}-{suffix}"))
    }
}
