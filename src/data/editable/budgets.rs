use super::*;
use crate::model::{BudgetDirection, BudgetIncomeBasis};

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
        let code = csv_cell(&headers, &row, "code");
        if code.trim().is_empty() {
            continue;
        }
        let category = non_empty(csv_cell(&headers, &row, "category"), "Uncategorized");
        let direction =
            BudgetDirection::parse(&csv_cell(&headers, &row, "direction"), &code, &category);
        budgets.push(EditableBudget {
            code,
            category,
            monthly_budget: csv_cell(&headers, &row, "monthly_budget"),
            yearly_budget: csv_cell(&headers, &row, "yearly_budget"),
            direction: direction.as_str().to_string(),
            income_basis: BudgetIncomeBasis::parse(&csv_cell(&headers, &row, "income_basis"))
                .as_str()
                .to_string(),
            notes: csv_cell(&headers, &row, "notes"),
        });
    }

    Ok(budgets)
}

pub(in crate::data) fn serialize_editable_budgets(budgets: &[EditableBudget]) -> Result<String> {
    let mut wtr = csv::WriterBuilder::new().from_writer(Vec::new());
    wtr.write_record([
        "code",
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
        let direction = BudgetDirection::parse(&budget.direction, &budget.code, &budget.category);
        let income_basis = BudgetIncomeBasis::parse(&budget.income_basis);
        wtr.write_record([
            budget.code.trim().to_string(),
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
