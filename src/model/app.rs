use super::{BudgetCode, DedupeMode, ImportReport, MonthKey, Transaction};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum ComparisonMode {
    #[default]
    CurrentOnly,
    WithPrevious,
}

impl ComparisonMode {
    pub fn includes_previous(self) -> bool {
        matches!(self, Self::WithPrevious)
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum TransactionLoadScope {
    #[default]
    Unloaded,
    All,
    Year(Option<i32>),
    YearWithPrevious(Option<i32>),
    Month(Option<MonthKey>),
    MonthWithPrevious(Option<MonthKey>),
}

impl TransactionLoadScope {
    pub fn for_year(year: Option<i32>, comparison: ComparisonMode) -> Self {
        if comparison.includes_previous() {
            Self::YearWithPrevious(year)
        } else {
            Self::Year(year)
        }
    }

    pub fn for_month(month: Option<MonthKey>, comparison: ComparisonMode) -> Self {
        if comparison.includes_previous() {
            Self::MonthWithPrevious(month)
        } else {
            Self::Month(month)
        }
    }

    pub fn resolve(self, default_month: Option<MonthKey>) -> Self {
        match self {
            Self::Year(None) => Self::Year(default_month.map(|month| month.year)),
            Self::YearWithPrevious(None) => {
                Self::YearWithPrevious(default_month.map(|month| month.year))
            }
            Self::Month(None) => Self::Month(default_month),
            Self::MonthWithPrevious(None) => Self::MonthWithPrevious(default_month),
            _ => self,
        }
    }

    pub fn satisfies(self, desired: Self) -> bool {
        if self == desired {
            return true;
        }

        match self {
            Self::Unloaded => false,
            Self::All => !matches!(desired, Self::Unloaded),
            Self::Year(Some(year)) => desired_is_inside_years(desired, &[year]),
            Self::YearWithPrevious(Some(year)) => {
                desired_is_inside_years(desired, &[year - 1, year])
            }
            Self::Month(Some(month)) => desired_is_inside_months(desired, &[month]),
            Self::MonthWithPrevious(Some(month)) => {
                desired_is_inside_months(desired, &[month.previous(), month])
            }
            Self::Year(None)
            | Self::YearWithPrevious(None)
            | Self::Month(None)
            | Self::MonthWithPrevious(None) => false,
        }
    }
}

fn desired_is_inside_years(desired: TransactionLoadScope, years: &[i32]) -> bool {
    match desired {
        TransactionLoadScope::Year(Some(year)) => years.contains(&year),
        TransactionLoadScope::YearWithPrevious(Some(year)) => {
            years.contains(&year) && years.contains(&(year - 1))
        }
        TransactionLoadScope::Month(Some(month)) => years.contains(&month.year),
        TransactionLoadScope::MonthWithPrevious(Some(month)) => {
            years.contains(&month.year) && years.contains(&month.previous().year)
        }
        TransactionLoadScope::Unloaded
        | TransactionLoadScope::All
        | TransactionLoadScope::Year(None)
        | TransactionLoadScope::YearWithPrevious(None)
        | TransactionLoadScope::Month(None)
        | TransactionLoadScope::MonthWithPrevious(None) => false,
    }
}

fn desired_is_inside_months(desired: TransactionLoadScope, months: &[MonthKey]) -> bool {
    match desired {
        TransactionLoadScope::Month(Some(month)) => months.contains(&month),
        TransactionLoadScope::MonthWithPrevious(Some(month)) => {
            months.contains(&month) && months.contains(&month.previous())
        }
        TransactionLoadScope::Unloaded
        | TransactionLoadScope::All
        | TransactionLoadScope::Year(_)
        | TransactionLoadScope::YearWithPrevious(_)
        | TransactionLoadScope::Month(None)
        | TransactionLoadScope::MonthWithPrevious(None) => false,
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppData {
    pub transactions: Vec<Transaction>,
    pub reports: Vec<ImportReport>,
    pub warnings: Vec<String>,
    pub duplicate_count: usize,
    pub dedupe_mode: DedupeMode,
    pub budgets: Vec<BudgetCode>,
    pub rules_count: usize,
    pub available_years: Vec<i32>,
    pub available_months: Vec<MonthKey>,
    pub default_month: Option<MonthKey>,
    pub loaded_scope: TransactionLoadScope,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_scope_satisfies_narrower_scopes() {
        assert!(TransactionLoadScope::All.satisfies(TransactionLoadScope::Year(Some(2025))));
        assert!(TransactionLoadScope::All
            .satisfies(TransactionLoadScope::Month(Some(MonthKey::new(2025, 5),))));
        assert!(TransactionLoadScope::All.satisfies(TransactionLoadScope::Year(None)));
        assert!(!TransactionLoadScope::All.satisfies(TransactionLoadScope::Unloaded));
    }

    #[test]
    fn year_scope_satisfies_months_inside_that_year() {
        assert!(TransactionLoadScope::Year(Some(2025))
            .satisfies(TransactionLoadScope::Month(Some(MonthKey::new(2025, 5)))));
        assert!(!TransactionLoadScope::Year(Some(2025))
            .satisfies(TransactionLoadScope::Month(Some(MonthKey::new(2024, 12)))));
        assert!(!TransactionLoadScope::Month(Some(MonthKey::new(2025, 5)))
            .satisfies(TransactionLoadScope::Year(Some(2025))));
    }

    #[test]
    fn previous_period_scopes_cover_current_and_previous_periods() {
        assert!(
            TransactionLoadScope::MonthWithPrevious(Some(MonthKey::new(2025, 5)))
                .satisfies(TransactionLoadScope::Month(Some(MonthKey::new(2025, 4))))
        );
        assert!(TransactionLoadScope::YearWithPrevious(Some(2025))
            .satisfies(TransactionLoadScope::Year(Some(2024))));
        assert!(
            TransactionLoadScope::YearWithPrevious(Some(2025)).satisfies(
                TransactionLoadScope::MonthWithPrevious(Some(MonthKey::new(2025, 1)))
            )
        );
        assert!(
            !TransactionLoadScope::YearWithPrevious(Some(2025)).satisfies(
                TransactionLoadScope::MonthWithPrevious(Some(MonthKey::new(2024, 1)))
            )
        );
    }

    #[test]
    fn unresolved_scopes_only_satisfy_exact_or_all_scopes() {
        assert!(TransactionLoadScope::Year(None).satisfies(TransactionLoadScope::Year(None)));
        assert!(TransactionLoadScope::All.satisfies(TransactionLoadScope::Month(None)));
        assert!(!TransactionLoadScope::Year(Some(2025)).satisfies(TransactionLoadScope::Year(None)));
        assert!(!TransactionLoadScope::Month(Some(MonthKey::new(2025, 5)))
            .satisfies(TransactionLoadScope::Month(None)));
    }
}
