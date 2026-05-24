use super::super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageCapabilities {
    pub data_readable: bool,
    pub data_writable: bool,
    pub config_readable: bool,
    pub config_writable: bool,
    pub data_reason: String,
    pub config_reason: String,
}

impl Default for StorageCapabilities {
    fn default() -> Self {
        Self {
            data_readable: true,
            data_writable: true,
            config_readable: true,
            config_writable: true,
            data_reason: String::new(),
            config_reason: String::new(),
        }
    }
}

impl StorageCapabilities {
    pub fn data_write_reason(&self) -> &str {
        if self.data_reason.is_empty() {
            "CSV storage is read-only."
        } else {
            &self.data_reason
        }
    }

    pub fn config_write_reason(&self) -> &str {
        if self.config_reason.is_empty() {
            "Configuration storage is read-only."
        } else {
            &self.config_reason
        }
    }
}

pub fn storage_capabilities(dirs: &AppDirs) -> StorageCapabilities {
    let (data_readable, data_writable, data_reason) =
        directory_capability(&dirs.inbox, "CSV storage");
    let (config_readable, config_writable, config_reason) =
        directory_capability(&dirs.config, "Configuration storage");
    StorageCapabilities {
        data_readable,
        data_writable,
        config_readable,
        config_writable,
        data_reason,
        config_reason,
    }
}

pub fn current_storage_capabilities() -> StorageCapabilities {
    app_dirs()
        .map(|dirs| crate::data::storage_capabilities(&dirs))
        .unwrap_or_else(|error| StorageCapabilities {
            data_readable: false,
            data_writable: false,
            config_readable: false,
            config_writable: false,
            data_reason: format!("Could not find app storage folders: {error:#}"),
            config_reason: format!("Could not find configuration folder: {error:#}"),
        })
}

fn directory_capability(path: &Path, label: &str) -> (bool, bool, String) {
    if path.exists() {
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) => {
                return (
                    false,
                    false,
                    format!("{label} cannot be inspected: {error}"),
                );
            }
        };
        if !metadata.is_dir() {
            return (false, false, format!("{label} is not a folder."));
        }
        let readable = fs::read_dir(path).is_ok();
        let writable = !metadata.permissions().readonly();
        let reason = if writable {
            String::new()
        } else {
            format!("{label} is read-only.")
        };
        return (readable, writable, reason);
    }

    match nearest_existing_parent(path) {
        Some(parent) => match fs::metadata(&parent) {
            Ok(metadata) if metadata.is_dir() && !metadata.permissions().readonly() => {
                (true, true, String::new())
            }
            Ok(_) => (
                true,
                false,
                format!("{label} cannot be created because its parent is read-only."),
            ),
            Err(error) => (true, false, format!("{label} cannot be inspected: {error}")),
        },
        None => (
            true,
            false,
            format!("{label} cannot be created because no parent folder exists."),
        ),
    }
}

fn nearest_existing_parent(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent();
    while let Some(parent) = current {
        if parent.exists() {
            return Some(parent.to_path_buf());
        }
        current = parent.parent();
    }
    None
}

pub fn mark_transaction_csv_readonly(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Could not inspect imported CSV: {}", path.display()))?;
    let mut permissions = metadata.permissions();
    if !permissions.readonly() {
        permissions.set_readonly(true);
        fs::set_permissions(path, permissions).with_context(|| {
            format!("Could not mark imported CSV read-only: {}", path.display())
        })?;
    }
    Ok(())
}

pub(super) fn mark_existing_transaction_csvs_readonly(dirs: &AppDirs) -> Vec<String> {
    let mut warnings = Vec::new();
    let Ok(entries) = fs::read_dir(&dirs.inbox) else {
        return warnings;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && is_csv(&path) {
            if let Err(error) = mark_transaction_csv_readonly(&path) {
                warnings.push(format!(
                    "Could not mark imported CSV read-only: {}: {error:#}",
                    path.display()
                ));
            }
        }
    }
    warnings
}
