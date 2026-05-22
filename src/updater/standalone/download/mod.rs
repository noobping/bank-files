use super::*;

mod core;
mod dialogs;

pub(in crate::updater::standalone) use core::download_release;
pub(in crate::updater::standalone) use dialogs::{
    build_download_dialog, confirm_install, present_alert, show_alert,
};
