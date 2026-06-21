mod charts;
mod details;
mod render;
mod rows;

pub(in crate::app) use render::render_budget_page;
pub(in crate::app) use rows::{more_budgets_row, more_categories_row};
