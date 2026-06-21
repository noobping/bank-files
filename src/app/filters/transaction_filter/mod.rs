mod matching;
mod query;

use crate::model::MonthKey;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::app) enum TransactionAmountFilter {
    Income,
    Expense,
    Transfer,
    Refund,
}

#[derive(Debug, Clone, PartialEq)]
pub(in crate::app) enum TransactionFilter {
    All,
    UnconfiguredBudgets,
    OtherCategories,
    CategoryForYear {
        category: String,
        year: i32,
    },
    Scoped {
        budget_code: Option<String>,
        year: Option<i32>,
        month: Option<MonthKey>,
        amount: Option<TransactionAmountFilter>,
    },
}

impl TransactionFilter {
    pub(in crate::app) fn all() -> Self {
        Self::All
    }

    pub(in crate::app) fn year(year: i32) -> Self {
        Self::Scoped {
            budget_code: None,
            year: Some(year),
            month: None,
            amount: None,
        }
    }

    pub(in crate::app) fn month(month: MonthKey) -> Self {
        Self::Scoped {
            budget_code: None,
            year: None,
            month: Some(month),
            amount: None,
        }
    }

    pub(in crate::app) fn income_for_year(year: i32) -> Self {
        Self::Scoped {
            budget_code: None,
            year: Some(year),
            month: None,
            amount: Some(TransactionAmountFilter::Income),
        }
    }

    pub(in crate::app) fn expenses_for_year(year: i32) -> Self {
        Self::Scoped {
            budget_code: None,
            year: Some(year),
            month: None,
            amount: Some(TransactionAmountFilter::Expense),
        }
    }

    pub(in crate::app) fn category_for_year(category: impl Into<String>, year: i32) -> Self {
        Self::CategoryForYear {
            category: category.into(),
            year,
        }
    }

    pub(in crate::app) fn income_for_month(month: MonthKey) -> Self {
        Self::Scoped {
            budget_code: None,
            year: None,
            month: Some(month),
            amount: Some(TransactionAmountFilter::Income),
        }
    }

    pub(in crate::app) fn expenses_for_month(month: MonthKey) -> Self {
        Self::Scoped {
            budget_code: None,
            year: None,
            month: Some(month),
            amount: Some(TransactionAmountFilter::Expense),
        }
    }

    pub(in crate::app) fn budget_for_year(code: impl Into<String>, year: i32) -> Self {
        Self::Scoped {
            budget_code: Some(code.into()),
            year: Some(year),
            month: None,
            amount: None,
        }
    }

    pub(in crate::app) fn budget_for_month(code: impl Into<String>, month: MonthKey) -> Self {
        Self::Scoped {
            budget_code: Some(code.into()),
            year: None,
            month: Some(month),
            amount: None,
        }
    }

    pub(in crate::app) fn with_year(&self, year: i32) -> Option<Self> {
        match self {
            Self::CategoryForYear { category, .. } => Some(Self::category_for_year(category, year)),
            Self::Scoped {
                budget_code,
                month,
                amount,
                ..
            } => Some(Self::Scoped {
                budget_code: budget_code.clone(),
                year: month.is_none().then_some(year),
                month: month.map(|month| MonthKey::new(year, month.month)),
                amount: *amount,
            }),
            Self::All | Self::UnconfiguredBudgets | Self::OtherCategories => None,
        }
    }

    pub(in crate::app) fn is_period_scoped(&self) -> bool {
        matches!(self, Self::CategoryForYear { .. } | Self::Scoped { .. })
    }

    pub(in crate::app) fn period_year(&self) -> Option<i32> {
        match self {
            Self::CategoryForYear { year, .. } => Some(*year),
            Self::Scoped { year, month, .. } => year.or_else(|| month.map(|month| month.year)),
            Self::All | Self::UnconfiguredBudgets | Self::OtherCategories => None,
        }
    }

    pub(in crate::app) fn shows_refunds(&self) -> bool {
        matches!(
            self,
            Self::Scoped {
                amount: Some(TransactionAmountFilter::Refund),
                ..
            }
        )
    }

    pub(in crate::app) fn label(&self) -> &'static str {
        match self {
            Self::All => "All transactions",
            Self::UnconfiguredBudgets => "Unconfigured budgets",
            Self::OtherCategories => "Other categories",
            Self::CategoryForYear { .. } => "Category transactions",
            Self::Scoped { .. } => "Transactions",
        }
    }

    pub(in crate::app) fn description(&self) -> &'static str {
        match self {
            Self::All => "All transactions",
            Self::UnconfiguredBudgets => {
                "Expense transactions with a missing or unknown budget code."
            }
            Self::OtherCategories => "Transactions grouped under OTHER or INC-OTHER.",
            Self::CategoryForYear { .. } => "Transactions for this category and year.",
            Self::Scoped { .. } => "Filtered transactions",
        }
    }
}
