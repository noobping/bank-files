use super::*;
use chrono::Datelike;

pub fn dashboard(data: &AppData) -> Dashboard {
    let monthly =
        monthly_totals_without_transfers(&data.transactions, &data.budgets, DASHBOARD_MONTH_LIMIT);
    let latest_month = default_reporting_month(&data.transactions, &data.budgets);
    let latest_totals = latest_month
        .map(|month| totals_for_month(&data.transactions, &data.budgets, month))
        .unwrap_or_default();

    Dashboard {
        latest_month,
        latest_totals,
        all_totals: totals_for(financial_transactions(&data.transactions, &data.budgets)),
        monthly,
        top_categories: latest_month
            .map(|month| category_totals_for_month(&data.transactions, &data.budgets, month, 8))
            .unwrap_or_default(),
        budgets: latest_month
            .map(|month| budget_usage(&data.transactions, &data.budgets, month))
            .unwrap_or_default(),
    }
}

pub fn totals_for_month(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    month: MonthKey,
) -> Totals {
    totals_for(financial_transactions(transactions, budgets).filter(|tx| tx.month_key() == month))
}

pub fn totals_for_year(transactions: &[Transaction], budgets: &[BudgetCode], year: i32) -> Totals {
    totals_for(financial_transactions(transactions, budgets).filter(|tx| tx.year() == year))
}

pub fn monthly_totals_without_transfers(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    limit: usize,
) -> Vec<MonthSummary> {
    monthly_totals_from(financial_transactions(transactions, budgets), limit)
}

pub fn default_reporting_month(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
) -> Option<MonthKey> {
    let months = monthly_totals_without_transfers(transactions, budgets, usize::MAX);
    let latest = months.last().map(|summary| summary.month)?;
    let today = chrono::Local::now().date_naive();
    let current = MonthKey::new(today.year(), today.month());

    if months.iter().any(|summary| summary.month == current) {
        Some(current.previous())
    } else {
        Some(latest)
    }
}

fn monthly_totals_from<'a>(
    transactions: impl Iterator<Item = &'a Transaction>,
    limit: usize,
) -> Vec<MonthSummary> {
    let mut by_month: BTreeMap<MonthKey, Totals> = BTreeMap::new();
    for tx in transactions {
        by_month.entry(tx.month_key()).or_default().add(tx);
    }

    let mut months = by_month
        .into_iter()
        .map(|(month, totals)| MonthSummary { month, totals })
        .collect::<Vec<_>>();
    if months.len() > limit {
        months = months.split_off(months.len() - limit);
    }
    months
}

pub fn year_comparison(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    year: i32,
) -> Option<YearComparison> {
    let previous_year = year - 1;
    let current = totals_for_year(transactions, budgets, year);
    if current.count == 0 {
        return None;
    }
    let previous = totals_for_year(transactions, budgets, previous_year);

    Some(YearComparison {
        year,
        previous_year,
        income_delta: current.income - previous.income,
        expense_delta: current.expenses - previous.expenses,
        balance_delta: current.balance - previous.balance,
        current,
        previous,
    })
}

pub fn survival_forecast(data: &AppData) -> Option<SurvivalForecast> {
    if data.transactions.is_empty() {
        return None;
    }

    let anchor_month = forecast_anchor_month(&data.transactions)?;
    let next_month = anchor_month.next();
    let recent = monthly_totals_without_transfers(&data.transactions, &data.budgets, 6);
    let average_income = average_recent_amount(&recent, anchor_month, |totals| totals.income);
    let average_expenses = average_recent_amount(&recent, anchor_month, |totals| totals.expenses);
    let expected_income = expected_monthly_income(&data.budgets, average_income);
    let planned_expenses =
        planned_monthly_expenses(&data.budgets, expected_income, average_expenses);

    let current_actual = month_totals(data, anchor_month);
    let next_actual = month_totals(data, next_month);
    let current_month = forecast_month(current_actual, expected_income, planned_expenses);
    let next_month_period = forecast_month(next_actual, expected_income, planned_expenses);
    let rest_of_year = forecast_rest_of_year(
        data,
        anchor_month,
        &current_month,
        expected_income,
        planned_expenses,
    );

    Some(SurvivalForecast {
        anchor_month,
        next_month,
        current_month,
        next_month_period,
        rest_of_year,
    })
}

fn forecast_anchor_month(transactions: &[Transaction]) -> Option<MonthKey> {
    let today = chrono::Local::now().date_naive();
    let current = MonthKey::new(today.year(), today.month());
    let latest = transactions.iter().map(Transaction::month_key).max()?;
    if transactions.iter().any(|tx| tx.month_key() == current) || latest > current {
        Some(current)
    } else {
        Some(latest)
    }
}

fn average_recent_amount(
    months: &[MonthSummary],
    anchor_month: MonthKey,
    amount: impl Fn(&Totals) -> Decimal,
) -> Decimal {
    let mut values = months
        .iter()
        .filter(|month| month.month < anchor_month)
        .rev()
        .take(3)
        .map(|month| amount(&month.totals))
        .filter(|value| *value > Decimal::ZERO)
        .collect::<Vec<_>>();

    if values.is_empty() {
        values = months
            .iter()
            .rev()
            .take(3)
            .map(|month| amount(&month.totals))
            .filter(|value| *value > Decimal::ZERO)
            .collect();
    }

    if values.is_empty() {
        Decimal::ZERO
    } else {
        values.iter().copied().sum::<Decimal>() / Decimal::from(values.len() as u64)
    }
}

fn expected_monthly_income(budgets: &[BudgetCode], historical_income: Decimal) -> Decimal {
    let budgeted_income = planned_month_income_total(budgets, historical_income);

    if budgeted_income > Decimal::ZERO {
        budgeted_income
    } else {
        historical_income
    }
}

fn planned_monthly_expenses(
    budgets: &[BudgetCode],
    expected_income: Decimal,
    historical_expenses: Decimal,
) -> Decimal {
    let planned_income = planned_month_income_total(budgets, expected_income);
    let budgeted_expenses = budgets
        .iter()
        .filter(|budget| budget.direction.is_expense())
        .map(|budget| budget.monthly_amount_with_basis(expected_income, planned_income))
        .sum::<Decimal>();

    if budgeted_expenses > Decimal::ZERO {
        budgeted_expenses
    } else {
        historical_expenses
    }
}

fn month_totals(data: &AppData, month: MonthKey) -> Totals {
    totals_for_month(&data.transactions, &data.budgets, month)
}

fn forecast_month(
    actual: Totals,
    expected_income: Decimal,
    planned_expenses: Decimal,
) -> ForecastPeriod {
    let income = actual.income.max(expected_income);
    let expenses = actual.expenses.max(planned_expenses);
    forecast_period(
        income,
        expenses,
        actual.income,
        expected_income,
        actual.expenses,
        planned_expenses,
    )
}

fn forecast_rest_of_year(
    data: &AppData,
    anchor_month: MonthKey,
    current_month: &ForecastPeriod,
    expected_income: Decimal,
    planned_expenses: Decimal,
) -> ForecastPeriod {
    let before_anchor = totals_for(
        financial_transactions(&data.transactions, &data.budgets)
            .filter(|tx| tx.year() == anchor_month.year && tx.month_key() < anchor_month),
    );
    let future_months = 12_u32.saturating_sub(anchor_month.month);
    let future_months_decimal = Decimal::from(future_months);
    let expected_period_income =
        current_month.expected_income + expected_income * future_months_decimal;
    let projected_income = current_month.income + expected_income * future_months_decimal;
    let projected_expenses_by_month =
        current_month.expenses + planned_expenses * future_months_decimal;

    let projected_year_income = before_anchor.income + projected_income;
    let planned_year_income = planned_year_income_total(&data.budgets, projected_year_income);
    let annual_expenses = data
        .budgets
        .iter()
        .filter(|budget| budget.direction.is_expense())
        .map(|budget| budget.annual_amount_with_basis(projected_year_income, planned_year_income))
        .sum::<Decimal>();
    let remaining_annual_expenses = (annual_expenses - before_anchor.expenses).max(Decimal::ZERO);
    let expenses = if annual_expenses > Decimal::ZERO {
        projected_expenses_by_month.max(remaining_annual_expenses)
    } else {
        projected_expenses_by_month
    };

    forecast_period(
        projected_income,
        expenses,
        current_month.imported_income,
        expected_period_income,
        current_month.imported_expenses,
        expenses,
    )
}

fn forecast_period(
    income: Decimal,
    expenses: Decimal,
    imported_income: Decimal,
    expected_income: Decimal,
    imported_expenses: Decimal,
    planned_expenses: Decimal,
) -> ForecastPeriod {
    let projected_balance = income - expenses;
    ForecastPeriod {
        income,
        expenses,
        projected_balance,
        imported_income,
        expected_income,
        imported_expenses,
        planned_expenses,
        status: forecast_status(projected_balance, income),
    }
}

fn forecast_status(projected_balance: Decimal, income: Decimal) -> ForecastStatus {
    if projected_balance < Decimal::ZERO {
        ForecastStatus::Short
    } else if income > Decimal::ZERO && projected_balance < income / Decimal::new(10, 0) {
        ForecastStatus::Tight
    } else {
        ForecastStatus::Safe
    }
}
