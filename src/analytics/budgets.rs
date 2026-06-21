use super::*;

pub fn budget_usage(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    month: MonthKey,
) -> Vec<BudgetUsage> {
    let real_month_income = totals_for(
        financial_transactions(transactions, budgets).filter(|tx| tx.month_key() == month),
    )
    .income;
    let planned_month_income = planned_month_income_total(budgets, real_month_income);
    let real_year_income = totals_for(
        financial_transactions(transactions, budgets).filter(|tx| tx.year() == month.year),
    )
    .income;
    let planned_year_income = planned_year_income_total(budgets, real_year_income);
    let mut actual_by_code: HashMap<String, Decimal> = HashMap::new();
    let mut earlier_actual_by_code: HashMap<String, Decimal> = HashMap::new();
    let mut category_by_code: HashMap<String, String> = HashMap::new();
    for tx in transactions
        .iter()
        .filter(|tx| tx.year() == month.year && tx.amount < Decimal::ZERO)
        .filter(|tx| !transaction_is_budget_neutral(tx, budgets))
    {
        if tx.budget_code.trim().is_empty() {
            continue;
        }
        let tx_month = tx.month_key();
        if tx_month == month {
            *actual_by_code.entry(tx.budget_code.clone()).or_default() += -tx.amount;
        } else if tx_month < month {
            *earlier_actual_by_code
                .entry(tx.budget_code.clone())
                .or_default() += -tx.amount;
        } else {
            continue;
        }
        category_by_code
            .entry(tx.budget_code.clone())
            .or_insert_with(|| tx.category.clone());
    }

    let mut rows = Vec::new();
    let mut configured_codes = HashMap::new();
    let context = MonthlyBudgetRollupContext {
        budgets,
        actual_by_code: &actual_by_code,
        earlier_actual_by_code: &earlier_actual_by_code,
        real_month_income,
        planned_month_income,
        real_year_income,
        planned_year_income,
    };
    for budget in budgets.iter().filter(|budget| {
        budget.direction.is_expense()
            && !budget.special.is_refund()
            && !is_refund_budget_code(&budget.code)
    }) {
        configured_codes.insert(budget.code.clone(), ());
        let (planned, actual, budget_basis) = monthly_budget_rollup_usage(budget, &context);
        rows.push(BudgetUsage {
            code: budget.code.clone(),
            category: budget.category.clone(),
            budget: planned,
            actual,
            remaining: planned - actual,
            budget_basis,
            notes: budget.notes.clone(),
        });
    }

    for (code, actual) in actual_by_code {
        if configured_codes.contains_key(&code) || crate::model::is_reserved_budget_code(&code) {
            continue;
        }
        rows.push(BudgetUsage {
            category: category_by_code
                .get(&code)
                .cloned()
                .unwrap_or_else(|| code.clone()),
            code,
            budget: Decimal::ZERO,
            actual,
            remaining: -actual,
            budget_basis: "unconfigured budget".to_string(),
            notes: "No monthly budget configured".to_string(),
        });
    }

    rows.sort_by(|a, b| a.code.cmp(&b.code));
    rows
}

struct MonthlyBudgetRollupContext<'a> {
    budgets: &'a [BudgetCode],
    actual_by_code: &'a HashMap<String, Decimal>,
    earlier_actual_by_code: &'a HashMap<String, Decimal>,
    real_month_income: Decimal,
    planned_month_income: Decimal,
    real_year_income: Decimal,
    planned_year_income: Decimal,
}

fn monthly_budget_rollup_usage(
    budget: &BudgetCode,
    context: &MonthlyBudgetRollupContext<'_>,
) -> (Decimal, Decimal, String) {
    let rollup_codes = budget_rollup_codes(context.budgets, &budget.code);
    let actual = sum_for_codes(context.actual_by_code, &rollup_codes);
    let planned = rollup_codes
        .iter()
        .filter_map(|code| budget_by_code(context.budgets, code))
        .map(|configured| {
            let earlier_actual = context
                .earlier_actual_by_code
                .get(&configured.code)
                .cloned()
                .unwrap_or(Decimal::ZERO);
            monthly_budget_usage_amount(
                configured,
                context.real_month_income,
                context.planned_month_income,
                context.real_year_income,
                context.planned_year_income,
                earlier_actual,
            )
            .0
        })
        .sum();
    let budget_basis = if budget_has_children(context.budgets, &budget.code) {
        "including sub budgets".to_string()
    } else {
        monthly_budget_usage_amount(
            budget,
            context.real_month_income,
            context.planned_month_income,
            context.real_year_income,
            context.planned_year_income,
            context
                .earlier_actual_by_code
                .get(&budget.code)
                .cloned()
                .unwrap_or(Decimal::ZERO),
        )
        .1
    };

    (planned, actual, budget_basis)
}

fn sum_for_codes(values: &HashMap<String, Decimal>, codes: &[String]) -> Decimal {
    codes
        .iter()
        .map(|code| values.get(code).cloned().unwrap_or(Decimal::ZERO))
        .sum()
}

fn budget_by_code<'a>(budgets: &'a [BudgetCode], code: &str) -> Option<&'a BudgetCode> {
    budgets
        .iter()
        .find(|budget| budget.code.trim().eq_ignore_ascii_case(code.trim()))
}

fn monthly_budget_usage_amount(
    budget: &BudgetCode,
    real_month_income: Decimal,
    planned_month_income: Decimal,
    real_year_income: Decimal,
    planned_year_income: Decimal,
    earlier_actual: Decimal,
) -> (Decimal, String) {
    if budget.monthly_budget.is_some() {
        return (
            budget.monthly_amount_with_basis(real_month_income, planned_month_income),
            budget.monthly_budget_description(),
        );
    }

    if budget.yearly_budget.is_some() {
        return (
            (budget.annual_amount_with_basis(real_year_income, planned_year_income)
                - earlier_actual)
                .max(Decimal::ZERO),
            "remaining yearly budget".to_string(),
        );
    }

    (Decimal::ZERO, "no budget".to_string())
}

pub(super) fn totals_for<'a>(transactions: impl Iterator<Item = &'a Transaction>) -> Totals {
    let mut totals = Totals::default();
    for tx in transactions {
        totals.add(tx);
    }
    totals
}
