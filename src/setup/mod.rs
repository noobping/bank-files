use crate::app_info::{
    APP_ID, APP_NAME, RESOURCE_ID, SEARCH_PROVIDER_BUS_NAME, SEARCH_PROVIDER_OBJECT_PATH,
};
use std::fs;
use std::io::{Error, ErrorKind};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::{
    process,
    time::{SystemTime, UNIX_EPOCH},
};

mod local;
mod paths;
#[cfg(test)]
mod tests;

pub use local::{
    can_install_locally, install_locally, is_current_executable_installed_locally,
    is_installed_locally, local_menu_action_label, uninstall_locally,
};

use paths::{
    can_install_into, installed_local_binary_path, same_file_path, write_desktop_file, write_icon,
    write_search_provider_file, write_search_provider_service_file,
};
#[cfg(test)]
use paths::{install_target_dir_is_eligible, is_writable};
