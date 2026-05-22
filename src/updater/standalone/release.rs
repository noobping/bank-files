use super::*;

pub(in crate::updater::standalone) fn fetch_update_release(
) -> Result<Option<SelectedRelease>, String> {
    let (owner, repo) = repository_owner_and_name()?;
    let url = format!(
        "https://api.github.com/repos/{owner}/{repo}/releases?per_page={GITHUB_RELEASES_PER_PAGE}"
    );
    let releases: Vec<GitHubReleaseResponse> = http_client()
        .get(url)
        .headers(github_headers()?)
        .send()
        .map_err(|error| format!("Failed to contact GitHub Releases: {error}"))?
        .error_for_status()
        .map_err(|error| format!("GitHub Releases returned an error: {error}"))?
        .json()
        .map_err(|error| format!("Failed to read GitHub Releases response: {error}"))?;
    let releases = releases
        .into_iter()
        .map(release_candidate)
        .collect::<Vec<_>>();
    select_update_release(env!("CARGO_PKG_VERSION"), &releases)
}

fn repository_owner_and_name() -> Result<(&'static str, &'static str), String> {
    let repository = env!("CARGO_PKG_REPOSITORY");
    let path = repository
        .strip_prefix("https://github.com/")
        .or_else(|| repository.strip_prefix("http://github.com/"))
        .ok_or_else(|| format!("Unsupported repository URL for updates: {repository}"))?;
    let mut parts = path.split('/');
    let owner = parts
        .next()
        .filter(|part| !part.is_empty())
        .ok_or_else(|| format!("Missing owner in repository URL: {repository}"))?;
    let repo = parts
        .next()
        .filter(|part| !part.is_empty())
        .map(|part| part.trim_end_matches(".git"))
        .ok_or_else(|| format!("Missing repository name in repository URL: {repository}"))?;
    Ok((owner, repo))
}

fn release_candidate(release: GitHubReleaseResponse) -> ReleaseCandidate {
    ReleaseCandidate {
        tag_name: release.tag_name,
        draft: release.draft,
        prerelease: release.prerelease,
        assets: release
            .assets
            .into_iter()
            .map(|asset| ReleaseAsset {
                name: asset.name,
                browser_download_url: asset.browser_download_url,
                size: asset.size,
                sha256_digest: asset.digest,
            })
            .collect(),
    }
}

fn select_update_release(
    current_version: &str,
    releases: &[ReleaseCandidate],
) -> Result<Option<SelectedRelease>, String> {
    logic::select_update_release_by(current_version, releases, platform_asset_matches)
}

fn platform_asset_matches(_release: &ReleaseCandidate, asset: &ReleaseAsset) -> bool {
    #[cfg(target_os = "windows")]
    {
        asset.name.to_ascii_lowercase().ends_with(".msi")
    }

    #[cfg(target_os = "linux")]
    {
        let Some(arch) = linux_release_arch() else {
            return false;
        };
        asset.name == linux_release_asset_name(&_release.tag_name, arch)
    }
}

#[cfg(target_os = "linux")]
fn linux_release_arch() -> Option<&'static str> {
    match std::env::consts::ARCH {
        "x86_64" => Some("x86_64"),
        "aarch64" => Some("aarch64"),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn linux_release_asset_name(tag_name: &str, arch: &str) -> String {
    format!("{}-{tag_name}.{arch}", env!("CARGO_PKG_NAME"))
}
