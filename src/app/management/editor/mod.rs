use super::*;

mod actions;
mod layout;

use actions::{connect_management_dialog_actions, ManagementDialogActions};

pub(in crate::app) use layout::show_management_dialog;
