use super::super::*;
use sha2::{Digest, Sha256};
use std::io::ErrorKind;

pub(super) fn write_cache_status(cache_key: &str, data: &mut AppData) {
    match write_cached_app_data(cache_key, data) {
        Ok(()) => data.cache_status = DataCacheStatus::Updated,
        Err(error) => {
            data.cache_status = DataCacheStatus::Failed(format!("{error:#}"));
            data.warnings.push(format!(
                "Data and analytics cache was not updated: {error:#}"
            ));
        }
    }
}

pub(super) fn read_cached_app_data(cache_key: &str) -> Result<Option<AppData>> {
    let path = app_data_cache_path(cache_key)?;
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "Could not read data and analytics cache: {}",
            path.display()
        )
    })?;
    let data = serde_json::from_str(&content).with_context(|| {
        format!(
            "Could not parse data and analytics cache: {}",
            path.display()
        )
    })?;
    Ok(Some(data))
}

pub fn clear_processed_app_data_cache() -> Result<bool> {
    let path = app_data_cache_root()?;
    match fs::remove_dir_all(&path) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| {
            format!(
                "Could not remove data and analytics cache folder: {}",
                path.display()
            )
        }),
    }
}

pub(super) fn write_cached_app_data(cache_key: &str, data: &AppData) -> Result<()> {
    let path = app_data_cache_path(cache_key)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create data and analytics cache folder: {}",
                parent.display()
            )
        })?;
    }
    let temp = path.with_extension("json.tmp");
    let content =
        serde_json::to_string(data).context("Could not serialize data and analytics cache")?;
    fs::write(&temp, content).with_context(|| {
        format!(
            "Could not write data and analytics cache: {}",
            temp.display()
        )
    })?;
    fs::rename(&temp, &path).with_context(|| {
        format!(
            "Could not replace data and analytics cache: {}",
            path.display()
        )
    })?;
    Ok(())
}

fn app_data_cache_path(cache_key: &str) -> Result<PathBuf> {
    Ok(app_data_cache_root()?.join(format!("{cache_key}.json")))
}

fn app_data_cache_root() -> Result<PathBuf> {
    Ok(app_cache_dir()?.join("processed"))
}

pub(super) fn app_data_cache_key(
    dirs: &AppDirs,
    mode: DedupeMode,
    scope: TransactionLoadScope,
    remember_mode: RememberMode,
    sources: &[TransactionSource],
    smart_insights_enabled: bool,
) -> Result<String> {
    let mut digest = Sha256::new();
    digest.update(env!("CARGO_PKG_VERSION").as_bytes());
    digest.update(
        format!("{mode:?}|{scope:?}|{remember_mode:?}|smart:{smart_insights_enabled}").as_bytes(),
    );
    for source in effective_cache_sources(dirs, remember_mode, sources)? {
        digest.update(format!("{:?}|{}|", source.kind, source.path.display()).as_bytes());
        match fs::metadata(&source.path) {
            Ok(metadata) => {
                digest.update(metadata.len().to_string().as_bytes());
                if let Ok(modified) = metadata.modified() {
                    digest.update(format!("{modified:?}").as_bytes());
                }
            }
            Err(error) => digest.update(format!("missing:{error}").as_bytes()),
        }
    }
    for name in ["rules.csv", "budgetcodes.csv", "field_aliases.csv"] {
        let path = dirs.config.join(name);
        digest.update(path.display().to_string().as_bytes());
        match fs::metadata(&path) {
            Ok(metadata) => {
                digest.update(metadata.len().to_string().as_bytes());
                if let Ok(modified) = metadata.modified() {
                    digest.update(format!("{modified:?}").as_bytes());
                }
            }
            Err(_) => digest.update(b"missing"),
        }
    }
    Ok(hex::encode(digest.finalize()))
}

fn effective_cache_sources(
    dirs: &AppDirs,
    remember_mode: RememberMode,
    sources: &[TransactionSource],
) -> Result<Vec<TransactionSource>> {
    if !sources.is_empty() || remember_mode.opens_live_files() {
        let mut current = sources.to_vec();
        current.sort_by(|left, right| left.path.cmp(&right.path));
        return Ok(current);
    }

    let mut inbox_sources = inbox_csv_files(dirs)?
        .into_iter()
        .map(TransactionSource::inbox_file)
        .collect::<Vec<_>>();
    inbox_sources.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(inbox_sources)
}

fn inbox_csv_files(dirs: &AppDirs) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if dirs.inbox.exists() {
        for entry in fs::read_dir(&dirs.inbox)
            .with_context(|| format!("Could not read CSV storage: {}", dirs.inbox.display()))?
        {
            let path = entry?.path();
            if path.is_file() && is_csv(&path) {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}
