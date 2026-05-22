use semver::Version;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
    pub sha256_digest: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseCandidate {
    pub tag_name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedRelease {
    pub tag_name: String,
    pub version: Version,
    pub asset: ReleaseAsset,
}

pub fn parse_release_version(tag_name: &str) -> Option<Version> {
    let trimmed = tag_name.trim();
    let normalized = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);
    Version::parse(normalized).ok()
}

pub fn select_update_release_by<F>(
    current_version: &str,
    releases: &[ReleaseCandidate],
    mut asset_matches: F,
) -> Result<Option<SelectedRelease>, String>
where
    F: FnMut(&ReleaseCandidate, &ReleaseAsset) -> bool,
{
    let current = Version::parse(current_version)
        .map_err(|error| format!("Current version is not valid SemVer: {error}"))?;

    Ok(releases
        .iter()
        .filter(|release| !release.draft && !release.prerelease)
        .filter_map(|release| {
            let version = parse_release_version(&release.tag_name)?;
            if version <= current {
                return None;
            }

            let asset = release
                .assets
                .iter()
                .find(|asset| asset_matches(release, asset))?
                .clone();

            Some(SelectedRelease {
                tag_name: release.tag_name.clone(),
                version,
                asset,
            })
        })
        .max_by(|left, right| left.version.cmp(&right.version)))
}

#[cfg(test)]
mod tests {
    use super::{parse_release_version, select_update_release_by, ReleaseAsset, ReleaseCandidate};

    fn asset(name: &str, size: u64) -> ReleaseAsset {
        ReleaseAsset {
            name: name.to_string(),
            browser_download_url: format!("https://example.com/{name}"),
            size,
            sha256_digest: None,
        }
    }

    fn release(
        tag_name: &str,
        draft: bool,
        prerelease: bool,
        assets: Vec<ReleaseAsset>,
    ) -> ReleaseCandidate {
        ReleaseCandidate {
            tag_name: tag_name.to_string(),
            draft,
            prerelease,
            assets,
        }
    }

    #[test]
    fn parses_semver_tags_with_or_without_v_prefix() {
        assert_eq!(
            parse_release_version("v1.2.3").map(|version| version.to_string()),
            Some("1.2.3".to_string())
        );
        assert_eq!(
            parse_release_version("1.2.3").map(|version| version.to_string()),
            Some("1.2.3".to_string())
        );
        assert_eq!(
            parse_release_version(" V2.0.0 ").map(|version| version.to_string()),
            Some("2.0.0".to_string())
        );
        assert!(parse_release_version("release-1.2.3").is_none());
    }

    #[test]
    fn selects_highest_newer_stable_release_with_matching_asset() {
        let releases = vec![
            release("v1.0.1", false, false, vec![asset("bank-files.zip", 10)]),
            release("v1.1.0", false, true, vec![asset("bank-files.msi", 20)]),
            release("v1.2.0", true, false, vec![asset("bank-files.msi", 30)]),
            release("v1.3.0", false, false, vec![asset("bank-files.msi", 40)]),
            release("v1.4.0", false, false, vec![asset("bank-files.exe", 50)]),
        ];

        let selected =
            select_update_release_by("1.0.0", &releases, |_, asset| asset.name.ends_with(".msi"))
                .expect("valid current version")
                .expect("expected release");
        assert_eq!(selected.version.to_string(), "1.3.0");
        assert_eq!(selected.asset.name, "bank-files.msi");
    }

    #[test]
    fn ignores_equal_and_older_versions() {
        let releases = vec![
            release("v0.9.9", false, false, vec![asset("bank-files.msi", 10)]),
            release("v1.0.0", false, false, vec![asset("bank-files.msi", 20)]),
        ];

        assert!(select_update_release_by("1.0.0", &releases, |_, asset| {
            asset.name.ends_with(".msi")
        })
        .expect("valid current version")
        .is_none());
    }

    #[test]
    fn can_match_linux_arch_assets_against_release_tags() {
        let releases = vec![
            release(
                "v1.1.0",
                false,
                false,
                vec![asset("bank-files-v1.1.0.aarch64", 20)],
            ),
            release(
                "v1.2.0",
                false,
                false,
                vec![
                    asset("bank-files-v1.2.0.aarch64", 30),
                    asset("bank-files-v1.2.0.x86_64", 31),
                ],
            ),
            release(
                "v1.3.0",
                false,
                true,
                vec![asset("bank-files-v1.3.0.x86_64", 40)],
            ),
        ];

        let selected = select_update_release_by("1.0.0", &releases, |release, asset| {
            asset.name == format!("bank-files-{}.x86_64", release.tag_name)
        })
        .expect("valid current version")
        .expect("expected release");
        assert_eq!(selected.version.to_string(), "1.2.0");
        assert_eq!(selected.asset.name, "bank-files-v1.2.0.x86_64");
    }
}
