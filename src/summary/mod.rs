use crate::analytics;
use crate::i18n::gettext;
use crate::model::{AppData, FieldMap, MonthKey, Transaction};
use crate::util::{money, pad_table, signed_money};
use std::collections::HashMap;

mod categories;
mod diagnostics;
mod overview;
mod tables;

pub use categories::{render_categories, render_categories_for_month, render_transactions};
pub use diagnostics::render_debug;
pub use overview::render_overview;

use overview::Totals;
use tables::{budget_rows, category_rows, truncate};

fn month_label(month: MonthKey) -> String {
    format!("{} {}", month_name(month.month), month.year)
}

fn month_name(month: u32) -> String {
    let name = match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => return month.to_string(),
    };
    gettext(name)
}
