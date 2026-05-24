use super::logic::{self, ReleaseAsset, ReleaseCandidate, SelectedRelease};
use crate::app_info::APP_NAME;
use crate::i18n::gettext;
#[cfg(target_os = "linux")]
use crate::setup;
use crate::ui;
use adw::gio::SimpleAction;
use adw::glib;
use adw::gtk::{Align, Box as GtkBox, Label, Orientation, ProgressBar};
use adw::prelude::*;
use adw::{AlertDialog, Application, ApplicationWindow, Dialog};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::fs::{self, File};
#[cfg(target_os = "linux")]
use std::io;
use std::io::{Read, Write};
#[cfg(target_os = "linux")]
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};
#[cfg(target_os = "linux")]
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;

mod check;
mod download;
mod platform;
mod release;
#[cfg(target_os = "linux")]
mod staging;
mod types;
mod validation;

pub use types::{
    after_window_presented, handle_special_command, register_app_actions, shutdown,
    supports_update_checks,
};

use check::start_update_check;
use download::{
    build_download_dialog, confirm_install, download_release, present_alert, show_alert,
};
use platform::{active_window, platform_update_checks_enabled};
#[cfg(target_os = "linux")]
use platform::{auto_install_cleanup_dir, run_auto_install};
use release::fetch_update_release;
#[cfg(target_os = "linux")]
use staging::create_update_dir;
use types::{
    CheckMode, DownloadMessage, DownloadedUpdate, GitHubReleaseResponse, GITHUB_API_ACCEPT,
    GITHUB_API_VERSION, GITHUB_RELEASES_PER_PAGE, WORKER_POLL_INTERVAL_MS,
};
#[cfg(target_os = "linux")]
use types::{
    AUTO_INSTALL_ARG, UPDATE_DIR_ATTEMPTS, UPDATE_EXECUTABLE_MODE, UPDATE_STAGING_DIR_MODE,
};
use validation::{
    cached_download_matches_release, github_headers, http_client, human_size,
    validate_downloaded_update,
};
