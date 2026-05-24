mod dialog;
mod loading;
mod render;
mod sections;
mod setup;
mod sizing;

pub(in crate::app) use dialog::show_management_dialog;

const MANAGEMENT_DIALOG_PARENT_INSET: i32 = 48;
const MANAGEMENT_DIALOG_MIN_WIDTH: i32 = 320;
const MANAGEMENT_DIALOG_MIN_HEIGHT: i32 = 360;
const MANAGEMENT_DIALOG_FALLBACK_WIDTH: i32 = 1040;
const MANAGEMENT_DIALOG_FALLBACK_HEIGHT: i32 = 760;
const MANAGEMENT_FORM_RENDER_BATCH_SIZE: usize = 18;
