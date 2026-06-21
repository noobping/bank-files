use super::*;
use crate::model::{
    budget_special_kind_for_config, canonical_special_budget_code, BudgetDirection,
    BudgetIncomeBasis, BudgetSpecialKind,
};

pub(in crate::data) fn parse_editable_budgets(contents: &str) -> Result<Vec<EditableBudget>> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(contents.as_bytes());
    let headers = rdr
        .headers()
        .context("budgetcodes.csv has no header")?
        .iter()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let mut budgets = Vec::new();

    for row in rdr.records() {
        let row = row?;
        let code = budget_code_for_config(&csv_cell(&headers, &row, "code"));
        let special = budget_special_kind_for_config(&budget_special_cell(&headers, &row), &code);
        if code.trim().is_empty() {
            continue;
        }
        let category = non_empty(csv_cell(&headers, &row, "category"), "Uncategorized");
        let direction = budget_direction_for_config(
            &csv_cell(&headers, &row, "direction"),
            &code,
            &category,
            special,
        );
        let income_basis =
            budget_income_basis_for_config(&csv_cell(&headers, &row, "income_basis"), special);
        budgets.push(EditableBudget {
            code,
            parent_code: csv_cell(&headers, &row, "parent_code"),
            special: special.as_config().to_string(),
            category,
            monthly_budget: csv_cell(&headers, &row, "monthly_budget"),
            yearly_budget: csv_cell(&headers, &row, "yearly_budget"),
            direction: direction.as_str().to_string(),
            income_basis: income_basis.as_str().to_string(),
            notes: csv_cell(&headers, &row, "notes"),
        });
    }

    Ok(budgets)
}

fn budget_special_cell(headers: &[String], row: &csv::StringRecord) -> String {
    let special = csv_cell(headers, row, "special");
    if special.trim().is_empty() {
        csv_cell(headers, row, "kind")
    } else {
        special
    }
}

fn budget_code_for_config(code: &str) -> String {
    canonical_special_budget_code(code)
        .unwrap_or_else(|| code.trim())
        .to_string()
}

fn budget_income_basis_for_config(input: &str, special: BudgetSpecialKind) -> BudgetIncomeBasis {
    if !matches!(special, BudgetSpecialKind::None) {
        BudgetIncomeBasis::RealIncome
    } else {
        BudgetIncomeBasis::parse(input)
    }
}

fn budget_direction_for_config(
    input: &str,
    code: &str,
    category: &str,
    special: BudgetSpecialKind,
) -> BudgetDirection {
    match special {
        BudgetSpecialKind::PlannedIncome | BudgetSpecialKind::Refunded => BudgetDirection::Income,
        BudgetSpecialKind::Transfer => BudgetDirection::Transfer,
        BudgetSpecialKind::Refunding => BudgetDirection::Expense,
        BudgetSpecialKind::None => BudgetDirection::parse(input, code, category),
    }
}

pub(in crate::data) fn serialize_editable_budgets(budgets: &[EditableBudget]) -> Result<String> {
    let mut wtr = csv::WriterBuilder::new().from_writer(Vec::new());
    wtr.write_record([
        "code",
        "parent_code",
        "special",
        "category",
        "monthly_budget",
        "yearly_budget",
        "direction",
        "income_basis",
        "notes",
    ])?;

    for budget in budgets
        .iter()
        .filter(|budget| !budget.code.trim().is_empty())
    {
        let special = budget_special_kind_for_config(&budget.special, &budget.code);
        let direction =
            budget_direction_for_config(&budget.direction, &budget.code, &budget.category, special);
        let income_basis = budget_income_basis_for_config(&budget.income_basis, special);
        wtr.write_record([
            budget_code_for_config(&budget.code),
            budget.parent_code.trim().to_string(),
            special.as_config().to_string(),
            budget.category.trim().to_string(),
            budget.monthly_budget.trim().to_string(),
            budget.yearly_budget.trim().to_string(),
            direction.as_str().to_string(),
            income_basis.as_str().to_string(),
            budget.notes.trim().to_string(),
        ])?;
    }

    writer_to_string(wtr)
}
