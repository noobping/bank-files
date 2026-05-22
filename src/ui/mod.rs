use crate::analytics::{CashFlowBreakdown, CashFlowSegment, CashFlowSegmentKind, MonthSummary};
use crate::i18n::{format as trf, gettext};
use crate::model::MonthKey;
use adw::glib::prelude::IsA;
use adw::gtk;
use adw::gtk::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

mod basic;
mod cards;
mod chart;
mod forms;
mod progress;
mod style;

pub use basic::*;
pub use cards::*;
pub use chart::*;
pub use forms::*;
pub use progress::*;
pub use style::*;
