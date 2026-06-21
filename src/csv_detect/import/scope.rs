use super::*;
use chrono::{Datelike, NaiveDate};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum DateSortOrder {
    Ascending,
    Descending,
    Unsorted,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ScopeBounds {
    All,
    Empty,
    Window { start: NaiveDate, end: NaiveDate },
}

impl ScopeBounds {
    pub(super) fn contains(self, date: NaiveDate) -> bool {
        match self {
            Self::All => true,
            Self::Empty => false,
            Self::Window { start, end } => date >= start && date < end,
        }
    }
}

pub(super) fn default_month_from_available_months(months: &[MonthKey]) -> Option<MonthKey> {
    let latest = months.last().copied()?;
    let today = chrono::Local::now().date_naive();
    let current = MonthKey::new(today.year(), today.month());
    if months.contains(&current) {
        Some(current.previous())
    } else {
        Some(latest)
    }
}

pub(super) fn scope_bounds(scope: TransactionLoadScope) -> ScopeBounds {
    match scope {
        TransactionLoadScope::Unloaded => ScopeBounds::Empty,
        TransactionLoadScope::All => ScopeBounds::All,
        TransactionLoadScope::Year(Some(year)) => year_bounds(year, false),
        TransactionLoadScope::YearWithPrevious(Some(year)) => year_bounds(year, true),
        TransactionLoadScope::Month(Some(month)) => month_bounds(month, false),
        TransactionLoadScope::MonthWithPrevious(Some(month)) => month_bounds(month, true),
        TransactionLoadScope::Year(None)
        | TransactionLoadScope::YearWithPrevious(None)
        | TransactionLoadScope::Month(None)
        | TransactionLoadScope::MonthWithPrevious(None) => ScopeBounds::Empty,
    }
}

fn year_bounds(year: i32, include_previous: bool) -> ScopeBounds {
    let start_year = if include_previous { year - 1 } else { year };
    let Some(start) = NaiveDate::from_ymd_opt(start_year, 1, 1) else {
        return ScopeBounds::Empty;
    };
    let Some(end) = NaiveDate::from_ymd_opt(year + 1, 1, 1) else {
        return ScopeBounds::Empty;
    };
    ScopeBounds::Window { start, end }
}

fn month_bounds(month: MonthKey, include_previous: bool) -> ScopeBounds {
    let start_month = if include_previous {
        month.previous()
    } else {
        month
    };
    let end_month = month.next();
    let Some(start) = NaiveDate::from_ymd_opt(start_month.year, start_month.month, 1) else {
        return ScopeBounds::Empty;
    };
    let Some(end) = NaiveDate::from_ymd_opt(end_month.year, end_month.month, 1) else {
        return ScopeBounds::Empty;
    };
    ScopeBounds::Window { start, end }
}

pub(super) fn should_skip_row(
    date: Option<NaiveDate>,
    bounds: ScopeBounds,
    sort_order: DateSortOrder,
) -> bool {
    let ScopeBounds::Window { start, end } = bounds else {
        return matches!(bounds, ScopeBounds::Empty);
    };
    let Some(date) = date else {
        return false;
    };
    match sort_order {
        DateSortOrder::Ascending => date < start,
        DateSortOrder::Descending => date >= end,
        DateSortOrder::Unsorted | DateSortOrder::Unknown => false,
    }
}

pub(super) fn should_stop_before_row(
    date: Option<NaiveDate>,
    bounds: ScopeBounds,
    sort_order: DateSortOrder,
) -> bool {
    let ScopeBounds::Window { start, end } = bounds else {
        return false;
    };
    let Some(date) = date else {
        return false;
    };
    match sort_order {
        DateSortOrder::Ascending => date >= end,
        DateSortOrder::Descending => date < start,
        DateSortOrder::Unsorted | DateSortOrder::Unknown => false,
    }
}
