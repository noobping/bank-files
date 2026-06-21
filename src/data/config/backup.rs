use super::super::*;
use super::files::config_file_path;
use super::{CONFIG_ARCHIVE_DIR, CONFIG_FILE_NAMES};
use std::time::{SystemTime, UNIX_EPOCH};

const BACKUP_PREFIX: &str = "backup-";
const CONFIG_ARCHIVE_KEEP_COUNT: usize = 5;
const LEGACY_ARCHIVE_ID: &str = "legacy";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConfigurationArchive {
    pub id: String,
    pub label: String,
    pub path: PathBuf,
}

struct ConfigurationArchiveEntry {
    archive: ConfigurationArchive,
    sort_key: u128,
}

pub(in crate::data) fn archive_configuration_in(dirs: &AppDirs) -> Result<PathBuf> {
    ensure_layout(dirs)?;
    ensure_default_files(dirs)?;

    let root = configuration_archive_dir(dirs);
    prepare_archive_root(&root)?;
    let archive = create_unique_archive_dir(&root)?;

    copy_config_files_to_archive(dirs, &archive)?;
    prune_old_archives(dirs)?;

    Ok(archive)
}

pub(in crate::data) fn restore_configuration_archive_in(dirs: &AppDirs) -> Result<PathBuf> {
    let archive = latest_configuration_archive_in(dirs)?;
    restore_archive_to_config(dirs, &archive)
}

pub(in crate::data) fn restore_configuration_archive_by_id_in(
    dirs: &AppDirs,
    id: &str,
) -> Result<PathBuf> {
    let archive = configuration_archive_by_id(dirs, id)?;
    restore_archive_to_config(dirs, &archive)
}

pub(in crate::data) fn remove_configuration_archive_in(
    dirs: &AppDirs,
    id: &str,
) -> Result<PathBuf> {
    let archive = configuration_archive_by_id(dirs, id)?;
    remove_archive_path(dirs, &archive)?;
    Ok(archive.path)
}

pub(in crate::data) fn configuration_archives_in(
    dirs: &AppDirs,
) -> Result<Vec<ConfigurationArchive>> {
    Ok(configuration_archive_entries_in(dirs)?
        .into_iter()
        .map(|entry| entry.archive)
        .collect())
}

pub(in crate::data) fn configuration_archive_exists_in(dirs: &AppDirs) -> bool {
    configuration_archives_in(dirs)
        .map(|archives| !archives.is_empty())
        .unwrap_or(false)
}

fn latest_configuration_archive_in(dirs: &AppDirs) -> Result<ConfigurationArchive> {
    configuration_archives_in(dirs)?
        .into_iter()
        .next()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No configuration backup exists in {}",
                configuration_archive_dir(dirs).display()
            )
        })
}

fn configuration_archive_by_id(dirs: &AppDirs, id: &str) -> Result<ConfigurationArchive> {
    configuration_archives_in(dirs)?
        .into_iter()
        .find(|archive| archive.id == id)
        .ok_or_else(|| anyhow::anyhow!("Configuration backup not found: {id}"))
}

fn configuration_archive_entries_in(dirs: &AppDirs) -> Result<Vec<ConfigurationArchiveEntry>> {
    let root = configuration_archive_dir(dirs);
    let mut entries = Vec::new();
    if !root.is_dir() {
        return Ok(entries);
    }

    if configuration_archive_files_exist(&root) {
        entries.push(configuration_archive_entry(LEGACY_ARCHIVE_ID, root.clone()));
    }

    for entry in
        fs::read_dir(&root).with_context(|| format!("Could not read {}", root.display()))?
    {
        let path = entry
            .with_context(|| format!("Could not read {}", root.display()))?
            .path();
        if path.is_dir() && configuration_archive_files_exist(&path) {
            let id = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string();
            entries.push(configuration_archive_entry(&id, path));
        }
    }

    entries.sort_by(|left, right| {
        right
            .sort_key
            .cmp(&left.sort_key)
            .then_with(|| right.archive.id.cmp(&left.archive.id))
    });
    Ok(entries)
}

fn configuration_archive_entry(id: &str, path: PathBuf) -> ConfigurationArchiveEntry {
    ConfigurationArchiveEntry {
        sort_key: archive_sort_key(id, &path),
        archive: ConfigurationArchive {
            id: id.to_string(),
            label: archive_label(&path),
            path,
        },
    }
}

fn archive_sort_key(id: &str, path: &Path) -> u128 {
    id.strip_prefix(BACKUP_PREFIX)
        .and_then(|value| value.split('-').next())
        .and_then(|value| value.parse::<u128>().ok())
        .unwrap_or_else(|| metadata_modified_key(path))
}

fn metadata_modified_key(path: &Path) -> u128 {
    path.metadata()
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

fn archive_label(path: &Path) -> String {
    path.metadata()
        .and_then(|metadata| metadata.modified())
        .map(|modified| {
            let local: chrono::DateTime<chrono::Local> = modified.into();
            local.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|_| {
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string()
        })
}

fn prepare_archive_root(root: &Path) -> Result<()> {
    if root.exists() && !root.is_dir() {
        fs::remove_file(root).with_context(|| format!("Could not replace {}", root.display()))?;
    }
    fs::create_dir_all(root).with_context(|| format!("Could not create {}", root.display()))
}

fn create_unique_archive_dir(root: &Path) -> Result<PathBuf> {
    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .with_context(|| "System clock is before the Unix epoch".to_string())?
        .as_nanos();

    for suffix in 0..1000 {
        let name = if suffix == 0 {
            format!("{BACKUP_PREFIX}{created}")
        } else {
            format!("{BACKUP_PREFIX}{created}-{suffix}")
        };
        let archive = root.join(name);
        match fs::create_dir(&archive) {
            Ok(()) => return Ok(archive),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("Could not create {}", archive.display()))
            }
        }
    }

    anyhow::bail!("Could not create a unique configuration backup folder")
}

fn copy_config_files_to_archive(dirs: &AppDirs, archive: &Path) -> Result<()> {
    for name in CONFIG_FILE_NAMES {
        let source = config_file_path(dirs, name)?;
        let target = archive.join(name);
        fs::copy(&source, &target).with_context(|| {
            format!(
                "Could not back up {} to {}",
                source.display(),
                target.display()
            )
        })?;
    }
    Ok(())
}

fn restore_archive_to_config(dirs: &AppDirs, archive: &ConfigurationArchive) -> Result<PathBuf> {
    ensure_layout(dirs)?;

    for name in CONFIG_FILE_NAMES {
        let source = archive.path.join(name);
        let target = config_file_path(dirs, name)?;
        fs::copy(&source, &target).with_context(|| {
            format!(
                "Could not restore {} to {}",
                source.display(),
                target.display()
            )
        })?;
    }

    Ok(archive.path.clone())
}

fn prune_old_archives(dirs: &AppDirs) -> Result<()> {
    let entries = configuration_archive_entries_in(dirs)?;
    for entry in entries.into_iter().skip(CONFIG_ARCHIVE_KEEP_COUNT) {
        remove_archive_path(dirs, &entry.archive)?;
    }
    Ok(())
}

fn remove_archive_path(dirs: &AppDirs, archive: &ConfigurationArchive) -> Result<()> {
    if archive.path == configuration_archive_dir(dirs) {
        remove_legacy_archive_files(&archive.path)
    } else {
        fs::remove_dir_all(&archive.path)
            .with_context(|| format!("Could not remove {}", archive.path.display()))
    }
}

fn remove_legacy_archive_files(root: &Path) -> Result<()> {
    for name in CONFIG_FILE_NAMES {
        let path = root.join(name);
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Could not remove {}", path.display()))?;
        }
    }
    Ok(())
}

fn configuration_archive_files_exist(path: &Path) -> bool {
    CONFIG_FILE_NAMES
        .iter()
        .all(|name| path.join(name).is_file())
}

fn configuration_archive_dir(dirs: &AppDirs) -> PathBuf {
    dirs.config.join(CONFIG_ARCHIVE_DIR)
}
