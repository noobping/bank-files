use super::*;

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct UpdateStagingRoot {
    path: PathBuf,
    needs_creation: bool,
}

#[cfg(target_os = "linux")]
pub(in crate::updater::standalone) fn create_update_dir() -> Result<PathBuf, String> {
    create_update_dir_in(&update_staging_root())
}

#[cfg(target_os = "linux")]
fn update_staging_root() -> UpdateStagingRoot {
    if let Some(base) = dirs_next::cache_dir().or_else(dirs_next::data_local_dir) {
        UpdateStagingRoot {
            path: base.join(env!("CARGO_PKG_NAME")).join("updates"),
            needs_creation: true,
        }
    } else {
        UpdateStagingRoot {
            path: std::env::temp_dir(),
            needs_creation: false,
        }
    }
}

#[cfg(target_os = "linux")]
fn create_update_dir_in(root: &UpdateStagingRoot) -> Result<PathBuf, String> {
    ensure_update_staging_root(root)?;

    for attempt in 0..UPDATE_DIR_ATTEMPTS {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let candidate = root
            .path
            .join(format!("update-{}-{nanos}-{attempt}", process::id()));
        match create_private_update_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "Failed to create Linux update staging directory '{}': {error}",
                    candidate.display()
                ));
            }
        }
    }

    Err("Failed to allocate a private Linux update staging directory.".to_string())
}

#[cfg(target_os = "linux")]
fn ensure_update_staging_root(root: &UpdateStagingRoot) -> Result<(), String> {
    if !root.needs_creation {
        return Ok(());
    }

    fs::create_dir_all(&root.path).map_err(|error| {
        format!(
            "Failed to create Linux update staging root '{}': {error}",
            root.path.display()
        )
    })?;
    let mut perms = fs::metadata(&root.path)
        .map_err(|error| {
            format!(
                "Failed to read Linux update staging root metadata '{}': {error}",
                root.path.display()
            )
        })?
        .permissions();
    perms.set_mode(UPDATE_STAGING_DIR_MODE);
    fs::set_permissions(&root.path, perms).map_err(|error| {
        format!(
            "Failed to secure Linux update staging root '{}': {error}",
            root.path.display()
        )
    })
}

#[cfg(target_os = "linux")]
fn create_private_update_dir(path: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.mode(UPDATE_STAGING_DIR_MODE);
    builder.create(path)
}
