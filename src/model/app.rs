use super::{BudgetCode, DedupeMode, ImportReport, MonthKey, Transaction};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum RememberMode {
    Forget,
    DataOnly,
    #[default]
    DataAndAnalytics,
}

impl RememberMode {
    pub const SETTINGS_VALUES: [Self; 3] = [Self::Forget, Self::DataOnly, Self::DataAndAnalytics];

    pub fn from_settings(value: &str) -> Self {
        match value {
            "forget" => Self::Forget,
            "data-and-analytics" => Self::DataAndAnalytics,
            "data-only" => Self::DataOnly,
            _ => Self::default(),
        }
    }

    pub fn as_settings(self) -> &'static str {
        match self {
            Self::Forget => "forget",
            Self::DataOnly => "data-only",
            Self::DataAndAnalytics => "data-and-analytics",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Forget => "Forget",
            Self::DataOnly => "Data only",
            Self::DataAndAnalytics => "Data and analytics",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Forget => "Open CSV files live for this session only.",
            Self::DataOnly => "Remember copied CSV files and configuration.",
            Self::DataAndAnalytics => "Remember copied CSV files and reusable analysis data.",
        }
    }

    pub fn opens_live_files(self) -> bool {
        matches!(self, Self::Forget)
    }

    pub fn uses_analytics_cache(self) -> bool {
        matches!(self, Self::DataAndAnalytics)
    }

    pub fn retains_less_than(self, other: Self) -> bool {
        self.retention_level() < other.retention_level()
    }

    fn retention_level(self) -> u8 {
        match self {
            Self::Forget => 0,
            Self::DataOnly => 1,
            Self::DataAndAnalytics => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionSourceKind {
    InboxFile,
    LiveFile,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionSource {
    pub kind: TransactionSourceKind,
    pub path: PathBuf,
}

impl TransactionSource {
    pub fn inbox_file(path: PathBuf) -> Self {
        Self {
            kind: TransactionSourceKind::InboxFile,
            path,
        }
    }

    pub fn live_file(path: PathBuf) -> Self {
        Self {
            kind: TransactionSourceKind::LiveFile,
            path,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_live(&self) -> bool {
        matches!(self.kind, TransactionSourceKind::LiveFile)
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum DataCacheStatus {
    #[default]
    Disabled,
    Hit,
    Updated,
    Skipped,
    Failed(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    pub remember_mode: RememberMode,
    pub transaction_sources: Vec<TransactionSource>,
    pub cache_status: DataCacheStatus,
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
    fn remember_mode_retention_levels_are_ordered() {
        assert!(RememberMode::Forget.retains_less_than(RememberMode::DataOnly));
        assert!(RememberMode::DataOnly.retains_less_than(RememberMode::DataAndAnalytics));
        assert!(RememberMode::Forget.retains_less_than(RememberMode::DataAndAnalytics));
        assert!(!RememberMode::DataAndAnalytics.retains_less_than(RememberMode::DataOnly));
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
