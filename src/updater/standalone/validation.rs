use super::*;

pub(in crate::updater::standalone) fn cached_download_matches_release(
    path: &Path,
    asset: &ReleaseAsset,
) -> Result<bool, String> {
    let Ok(metadata) = fs::metadata(path) else {
        return Ok(false);
    };
    if !metadata.is_file() || metadata.len() != asset.size {
        return Ok(false);
    }
    validate_update_file(path, asset).map(|_| true)
}

pub(in crate::updater::standalone) fn validate_downloaded_update(
    path: &Path,
    downloaded: u64,
    digest: &[u8],
    asset: &ReleaseAsset,
) -> Result<(), String> {
    if downloaded != asset.size {
        return Err(format!(
            "Update size mismatch after download (expected {}, got {}).",
            asset.size, downloaded
        ));
    }
    if let Some(expected) = expected_sha256(asset) {
        let actual = hex::encode(digest);
        if actual != expected {
            return Err(format!("Update SHA-256 mismatch for '{}'.", path.display()));
        }
    }
    Ok(())
}

pub(in crate::updater::standalone) fn validate_update_file(
    path: &Path,
    asset: &ReleaseAsset,
) -> Result<(), String> {
    if let Some(expected) = expected_sha256(asset) {
        let actual = hash_file(path)?;
        if actual != expected {
            return Err(format!("Update SHA-256 mismatch for '{}'.", path.display()));
        }
    }
    Ok(())
}

fn expected_sha256(asset: &ReleaseAsset) -> Option<String> {
    asset
        .sha256_digest
        .as_deref()
        .map(|digest| digest.strip_prefix("sha256:").unwrap_or(digest))
        .map(|digest| digest.to_ascii_lowercase())
}

fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(|error| format!("Failed to open update file '{}': {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("Failed to hash update file '{}': {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub(in crate::updater::standalone) fn http_client() -> Client {
    Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| Client::new())
}

pub(in crate::updater::standalone) fn github_headers() -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static(GITHUB_API_ACCEPT));
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static(GITHUB_API_VERSION),
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&format!("{APP_NAME}/{}", env!("CARGO_PKG_VERSION")))
            .map_err(|error| format!("Failed to build update request headers: {error}"))?,
    );
    Ok(headers)
}

pub(in crate::updater::standalone) fn human_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    let bytes = bytes as f64;
    if bytes >= MB {
        format!("{:.1} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes / KB)
    } else {
        format!("{bytes:.0} B")
    }
}
