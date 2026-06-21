use super::*;

pub(in crate::updater::standalone) const GITHUB_API_ACCEPT: &str = "application/vnd.github+json";
pub(in crate::updater::standalone) const GITHUB_API_VERSION: &str = "2022-11-28";
pub(in crate::updater::standalone) const GITHUB_RELEASES_PER_PAGE: usize = 100;
pub(in crate::updater::standalone) const WORKER_POLL_INTERVAL_MS: u64 = 80;
#[cfg(target_os = "linux")]
pub(in crate::updater::standalone) const AUTO_INSTALL_ARG: &str = "--auto-install";
#[cfg(target_os = "linux")]
pub(in crate::updater::standalone) const UPDATE_EXECUTABLE_MODE: u32 = 0o700;
#[cfg(target_os = "linux")]
pub(in crate::updater::standalone) const UPDATE_STAGING_DIR_MODE: u32 = 0o700;
#[cfg(target_os = "linux")]
pub(in crate::updater::standalone) const UPDATE_DIR_ATTEMPTS: usize = 32;

pub(in crate::updater::standalone) static AUTO_CHECK_STARTED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::updater::standalone) enum CheckMode {
    Automatic,
    Manual,
}

pub(in crate::updater::standalone) enum DownloadMessage {
    Progress { downloaded: u64, total: u64 },
    Finished(Result<DownloadOutcome, String>),
}

pub(in crate::updater::standalone) enum DownloadOutcome {
    Completed(DownloadedUpdate),
    Canceled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::updater::standalone) struct DownloadedUpdate {
    pub(in crate::updater::standalone) path: PathBuf,
}

#[derive(Deserialize)]
pub(in crate::updater::standalone) struct GitHubReleaseResponse {
    pub(in crate::updater::standalone) tag_name: String,
    pub(in crate::updater::standalone) draft: bool,
    pub(in crate::updater::standalone) prerelease: bool,
    pub(in crate::updater::standalone) assets: Vec<GitHubAssetResponse>,
}

#[derive(Deserialize)]
pub(in crate::updater::standalone) struct GitHubAssetResponse {
    pub(in crate::updater::standalone) name: String,
    pub(in crate::updater::standalone) browser_download_url: String,
    pub(in crate::updater::standalone) size: u64,
    pub(in crate::updater::standalone) digest: Option<String>,
}

pub fn register_app_actions(app: &Application) {
    if !supports_update_checks() {
        return;
    }

    let action = SimpleAction::new("check-for-updates", None);
    let app_for_action = app.clone();
    action.connect_activate(move |_, _| {
        let parent = active_window(&app_for_action);
        start_update_check(&app_for_action, parent, CheckMode::Manual);
    });
    app.add_action(&action);
}

pub fn supports_update_checks() -> bool {
    platform_update_checks_enabled()
}

pub fn after_window_presented(app: &Application, window: &ApplicationWindow) {
    if !supports_update_checks() {
        return;
    }
    if AUTO_CHECK_STARTED.swap(true, Ordering::Relaxed) {
        return;
    }

    start_update_check(app, Some(window.clone()), CheckMode::Automatic);
}

pub fn shutdown(_app: &Application) {}

pub fn handle_special_command(args: &[OsString]) -> Option<i32> {
    #[cfg(target_os = "linux")]
    {
        let cleanup_dir = auto_install_cleanup_dir(args)?;
        Some(run_auto_install(&cleanup_dir))
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = args;
        None
    }
}
