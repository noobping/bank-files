use super::BudgetSpecialKind;
use crate::util::{normalize_key, parse_decimal};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BudgetAmount {
    Fixed(Decimal),
    IncomePercent(Decimal),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum BudgetIncomeBasis {
    RealIncome,
    PlannedIncome,
}

impl BudgetAmount {
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();
        if input.is_empty() {
            return Some(Self::default());
        }

        if input.contains('%') {
            let percent_text = input.split('%').next().unwrap_or(input);
            return parse_decimal(percent_text).map(Self::IncomePercent);
        }

        parse_decimal(input).map(Self::Fixed)
    }

    pub fn parse_optional(input: &str) -> Option<Self> {
        let input = input.trim();
        if input.is_empty() {
            None
        } else {
            Self::parse(input)
        }
    }

    pub fn monthly_amount(&self, income: Decimal) -> Decimal {
        self.amount_for_income(income)
    }

    pub fn annual_amount(&self, income: Decimal) -> Decimal {
        match self {
            Self::Fixed(amount) => *amount * Decimal::new(12, 0),
            Self::IncomePercent(_) => self.amount_for_income(income),
        }
    }

    pub fn yearly_amount(&self, income: Decimal) -> Decimal {
        self.amount_for_income(income)
    }

    pub fn description_with_basis(&self, basis: BudgetIncomeBasis) -> String {
        match self {
            Self::Fixed(_) => "fixed budget".to_string(),
            Self::IncomePercent(percent) => format!("{percent}% of {}", basis.description()),
        }
    }

    fn amount_for_income(&self, income: Decimal) -> Decimal {
        match self {
            Self::Fixed(amount) => *amount,
            Self::IncomePercent(percent) => income * *percent / Decimal::new(100, 0),
        }
    }
}

impl Default for BudgetAmount {
    fn default() -> Self {
        Self::Fixed(Decimal::ZERO)
    }
}

impl BudgetIncomeBasis {
    pub fn parse(input: &str) -> Self {
        Self::from_config(input).unwrap_or(Self::RealIncome)
    }

    pub fn from_config(input: &str) -> Option<Self> {
        match normalize_key(input).as_str() {
            "real" | "actual" | "real income" | "actual income" | "werkelijk"
            | "werkelijk inkomen" | "echt" | "ist" | "tatsaechlich" | "tatsachlich"
            | "tatsächlich" => Some(Self::RealIncome),
            "planned"
            | "budget"
            | "budgeted"
            | "planned income"
            | "budgeted income"
            | "gepland"
            | "gepland inkomen"
            | "begroot"
            | "begroot inkomen"
            | "plan"
            | "geplant"
            | "geplantes einkommen" => Some(Self::PlannedIncome),
            _ => None,
        }
    }

    pub fn is_valid_config(input: &str) -> bool {
        input.trim().is_empty() || Self::from_config(input).is_some()
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::RealIncome => "real",
            Self::PlannedIncome => "planned",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::RealIncome => "real income",
            Self::PlannedIncome => "planned income",
        }
    }

    fn income(self, real_income: Decimal, planned_income: Decimal) -> Decimal {
        match self {
            Self::RealIncome => real_income,
            Self::PlannedIncome => planned_income,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum BudgetDirection {
    Expense,
    Income,
    Transfer,
}

impl BudgetDirection {
    pub fn parse(input: &str, code: &str, category: &str) -> Self {
        Self::from_config(input).unwrap_or_else(|| Self::infer(code, category))
    }

    pub fn from_config(input: &str) -> Option<Self> {
        match normalize_key(input).as_str() {
            "expense" | "expenses" | "uitgave" | "uitgaven" | "kosten" | "ausgabe" | "ausgaben"
            | "out" | "af" => Some(Self::Expense),
            "income" | "incomes" | "inkomen" | "inkomsten" | "opbrengst" | "opbrengsten"
            | "einkommen" | "einnahme" | "einnahmen" | "in" | "bij" => Some(Self::Income),
            "transfer"
            | "transfers"
            | "overboeking"
            | "overboekingen"
            | "interne overboeking"
            | "ueberweisung"
            | "überweisung"
            | "ueberweisungen"
            | "überweisungen"
            | "umbuchung"
            | "umbuchungen" => Some(Self::Transfer),
            _ => None,
        }
    }

    pub fn is_valid_config(input: &str) -> bool {
        input.trim().is_empty() || Self::from_config(input).is_some()
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Expense => "expense",
            Self::Income => "income",
            Self::Transfer => "transfer",
        }
    }

    pub fn is_expense(self) -> bool {
        matches!(self, Self::Expense)
    }

    pub fn is_transfer(self) -> bool {
        matches!(self, Self::Transfer)
    }

    fn infer(code: &str, category: &str) -> Self {
        let code = normalize_key(code);
        let category = normalize_key(category);
        if code.starts_with("transfer")
            || code == "trans"
            || category.contains("transfer")
            || category.contains("overboeking")
            || category.contains("overboekingen")
            || category.contains("ueberweisung")
            || category.contains("uberweisung")
            || category.contains("umbuchung")
        {
            Self::Transfer
        } else if code.starts_with("inc")
            || category.contains("income")
            || category.contains("inkomen")
            || category.contains("inkomsten")
            || category.contains("einkommen")
            || category.contains("einnahme")
            || category.contains("einnahmen")
        {
            Self::Income
        } else {
            Self::Expense
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetCode {
    pub code: String,
    pub special: BudgetSpecialKind,
    pub category: String,
    pub monthly_budget: Option<BudgetAmount>,
    pub yearly_budget: Option<BudgetAmount>,
    pub direction: BudgetDirection,
    pub income_basis: BudgetIncomeBasis,
    pub notes: String,
}

impl BudgetCode {
    pub fn monthly_amount_with_basis(
        &self,
        real_month_income: Decimal,
        planned_month_income: Decimal,
    ) -> Decimal {
        if let Some(monthly_budget) = &self.monthly_budget {
            let income = self
                .income_basis
                .income(real_month_income, planned_month_income);
            return monthly_budget.monthly_amount(income);
        }

        self.yearly_budget
            .as_ref()
            .map(|yearly_budget| {
                let income = self.income_basis.income(
                    real_month_income * Decimal::new(12, 0),
                    planned_month_income * Decimal::new(12, 0),
                );
                yearly_budget.yearly_amount(income) / Decimal::new(12, 0)
            })
            .unwrap_or(Decimal::ZERO)
    }

    pub fn annual_amount_with_basis(
        &self,
        real_year_income: Decimal,
        planned_year_income: Decimal,
    ) -> Decimal {
        if let Some(yearly_budget) = &self.yearly_budget {
            let income = self
                .income_basis
                .income(real_year_income, planned_year_income);
            return yearly_budget.yearly_amount(income);
        }

        self.monthly_budget
            .as_ref()
            .map(|monthly_budget| {
                let income = self
                    .income_basis
                    .income(real_year_income, planned_year_income);
                monthly_budget.annual_amount(income)
            })
            .unwrap_or(Decimal::ZERO)
    }

    pub fn monthly_budget_description(&self) -> String {
        if let Some(monthly_budget) = &self.monthly_budget {
            return monthly_budget.description_with_basis(self.income_basis);
        }

        self.yearly_budget
            .as_ref()
            .map(|_| "yearly budget / 12".to_string())
            .unwrap_or_else(|| "no budget".to_string())
    }

    pub fn annual_budget_description(&self) -> String {
        if let Some(yearly_budget) = &self.yearly_budget {
            return format!(
                "{} ({})",
                yearly_budget.description_with_basis(self.income_basis),
                "yearly budget"
            );
        }

        self.monthly_budget
            .as_ref()
            .map(|monthly_budget| monthly_budget.description_with_basis(self.income_basis))
            .unwrap_or_else(|| "no budget".to_string())
    }
}
