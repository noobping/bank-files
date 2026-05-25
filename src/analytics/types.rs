use crate::model::{MonthKey, Transaction};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Totals {
    pub income: Decimal,
    pub expenses: Decimal,
    pub balance: Decimal,
    pub count: usize,
}

impl Totals {
    pub fn add(&mut self, tx: &Transaction) {
        self.count += 1;
        self.balance += tx.amount;
        if tx.amount >= Decimal::ZERO {
            self.income += tx.amount;
        } else {
            self.expenses += -tx.amount;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonthSummary {
    pub month: MonthKey,
    pub totals: Totals,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategorySummary {
    pub category: String,
    pub budget_code: String,
    pub totals: Totals,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetUsage {
    pub code: String,
    pub category: String,
    pub budget: Decimal,
    pub actual: Decimal,
    pub remaining: Decimal,
    pub budget_basis: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnnualBudgetUsage {
    pub code: String,
    pub category: String,
    pub budget: Decimal,
    pub actual: Decimal,
    pub previous_actual: Option<Decimal>,
    pub remaining: Decimal,
    pub budget_basis: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YearCategoryComparison {
    pub category: String,
    pub budget_code: String,
    pub current: Totals,
    pub previous: Option<Totals>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YearComparison {
    pub year: i32,
    pub previous_year: i32,
    pub current: Totals,
    pub previous: Totals,
    pub income_delta: Decimal,
    pub expense_delta: Decimal,
    pub balance_delta: Decimal,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CashFlowSegmentKind {
    PlannedIncome,
    ActualIncome,
    PlannedExpense,
    ActualExpense,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CashFlowSegment {
    pub label: String,
    pub budget_code: String,
    pub amount: Decimal,
    pub kind: CashFlowSegmentKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CashFlowPeriod {
    pub label: String,
    pub totals: Totals,
    pub planned_income: Vec<CashFlowSegment>,
    pub actual_income: Vec<CashFlowSegment>,
    pub planned_expenses: Vec<CashFlowSegment>,
    pub actual_expenses: Vec<CashFlowSegment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CashFlowBreakdown {
    pub current: CashFlowPeriod,
    pub previous: Option<CashFlowPeriod>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Dashboard {
    pub latest_month: Option<MonthKey>,
    pub latest_totals: Totals,
    pub all_totals: Totals,
    pub monthly: Vec<MonthSummary>,
    pub top_categories: Vec<CategorySummary>,
    pub budgets: Vec<BudgetUsage>,
}

pub const DASHBOARD_MONTH_LIMIT: usize = 24;
