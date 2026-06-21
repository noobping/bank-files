use super::*;

pub(in crate::updater::standalone) fn download_release(
    release: &SelectedRelease,
    progress: &mpsc::Sender<DownloadMessage>,
    canceled: &AtomicBool,
) -> Result<DownloadOutcome, String> {
    if download_canceled(canceled) {
        return Ok(DownloadOutcome::Canceled);
    }

    let target = download_target_path(release)?;
    if use_cached_downloads() && cached_download_matches_release(&target, &release.asset)? {
        let _ = progress.send(DownloadMessage::Progress {
            downloaded: release.asset.size,
            total: release.asset.size,
        });
        return Ok(DownloadOutcome::Completed(DownloadedUpdate {
            path: target,
        }));
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "Failed to create update download directory '{}': {error}",
                parent.display()
            )
        })?;
    }

    let temp_path = target.with_extension("partial");
    if download_canceled(canceled) {
        return Ok(DownloadOutcome::Canceled);
    }

    let mut response = http_client()
        .get(&release.asset.browser_download_url)
        .headers(github_headers()?)
        .send()
        .map_err(|error| format!("Failed to download update: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Update download returned an error: {error}"))?;
    if download_canceled(canceled) {
        return Ok(DownloadOutcome::Canceled);
    }

    let mut file = File::create(&temp_path).map_err(|error| {
        format!(
            "Failed to create partial update file '{}': {error}",
            temp_path.display()
        )
    })?;
    let mut hasher = Sha256::new();
    let mut downloaded = 0u64;
    let mut buffer = [0u8; 64 * 1024];

    loop {
        if download_canceled(canceled) {
            return Ok(cancel_partial_download(file, &temp_path));
        }

        let read = response
            .read(&mut buffer)
            .map_err(|error| format!("Failed to read update bytes: {error}"))?;
        if read == 0 {
            break;
        }
        if download_canceled(canceled) {
            return Ok(cancel_partial_download(file, &temp_path));
        }

        file.write_all(&buffer[..read])
            .map_err(|error| format!("Failed to write update bytes: {error}"))?;
        hasher.update(&buffer[..read]);
        downloaded += read as u64;
        let _ = progress.send(DownloadMessage::Progress {
            downloaded,
            total: release.asset.size,
        });
    }
    file.flush()
        .map_err(|error| format!("Failed to flush update file: {error}"))?;

    if download_canceled(canceled) {
        return Ok(cancel_partial_download(file, &temp_path));
    }

    validate_downloaded_update(&temp_path, downloaded, &hasher.finalize(), &release.asset)?;
    if download_canceled(canceled) {
        return Ok(cancel_partial_download(file, &temp_path));
    }

    fs::rename(&temp_path, &target).map_err(|error| {
        format!(
            "Failed to finalize update file '{}': {error}",
            target.display()
        )
    })?;
    Ok(DownloadOutcome::Completed(DownloadedUpdate {
        path: target,
    }))
}

fn download_canceled(canceled: &AtomicBool) -> bool {
    canceled.load(Ordering::Relaxed)
}

fn cancel_partial_download(file: File, path: &Path) -> DownloadOutcome {
    drop(file);
    remove_partial_download(path);
    DownloadOutcome::Canceled
}

fn remove_partial_download(path: &Path) {
    let _ = fs::remove_file(path);
}

fn download_target_path(release: &SelectedRelease) -> Result<PathBuf, String> {
    #[cfg(target_os = "linux")]
    {
        Ok(create_update_dir()?.join(&release.asset.name))
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok(cached_download_path(release))
    }
}

#[cfg(not(target_os = "linux"))]
fn cached_download_path(release: &SelectedRelease) -> PathBuf {
    let base = dirs_next::cache_dir()
        .or_else(dirs_next::data_local_dir)
        .unwrap_or_else(std::env::temp_dir);
    base.join(env!("CARGO_PKG_NAME"))
        .join("updates")
        .join(format!(
            "{}-{}",
            release.version,
            sanitize_filename(&release.asset.name)
        ))
}

#[cfg(not(target_os = "linux"))]
fn sanitize_filename(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            ch if ch.is_control() => '_',
            ch => ch,
        })
        .collect::<String>();
    let sanitized = sanitized
        .trim_matches(|ch| ch == ' ' || ch == '.')
        .to_string();
    if sanitized.is_empty() {
        "download".to_string()
    } else {
        sanitized
    }
}

fn use_cached_downloads() -> bool {
    #[cfg(target_os = "linux")]
    {
        false
    }

    #[cfg(not(target_os = "linux"))]
    {
        true
    }
}
