use super::*;

mod mapping;
mod page;
mod widgets;

pub(in crate::app) use mapping::diagnostic_field_items;
pub(in crate::app) use page::render_diagnostics_page;
pub(in crate::app) use widgets::*;
