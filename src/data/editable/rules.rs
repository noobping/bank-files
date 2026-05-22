use super::*;

pub(in crate::data) fn parse_editable_rules(contents: &str) -> Result<Vec<EditableRule>> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(contents.as_bytes());
    let headers = rdr
        .headers()
        .context("rules.csv has no header")?
        .iter()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let mut rules = Vec::new();

    for row in rdr.records() {
        let row = row?;
        let pattern = csv_cell(&headers, &row, "pattern");
        if pattern.trim().is_empty() {
            continue;
        }
        let (search, is_regex) = form_search_from_pattern(&pattern);
        rules.push(EditableRule {
            priority: csv_cell(&headers, &row, "priority").parse().unwrap_or(0),
            active: parse_bool_cell(&csv_cell(&headers, &row, "active")),
            field: non_empty(csv_cell(&headers, &row, "field"), "any"),
            search,
            is_regex,
            category: non_empty(csv_cell(&headers, &row, "category"), "Uncategorized"),
            budget_code: csv_cell(&headers, &row, "budget_code"),
            direction: csv_cell(&headers, &row, "direction"),
            amount_min: csv_cell(&headers, &row, "amount_min"),
            amount_max: csv_cell(&headers, &row, "amount_max"),
            notes: csv_cell(&headers, &row, "notes"),
        });
    }

    Ok(rules)
}

pub(in crate::data) fn serialize_editable_rules(rules: &[EditableRule]) -> Result<String> {
    let mut wtr = csv::WriterBuilder::new().from_writer(Vec::new());
    wtr.write_record([
        "priority",
        "active",
        "field",
        "pattern",
        "category",
        "budget_code",
        "direction",
        "amount_min",
        "amount_max",
        "notes",
    ])?;

    for rule in rules.iter().filter(|rule| !rule.search.trim().is_empty()) {
        wtr.write_record([
            rule.priority.to_string(),
            rule.active.to_string(),
            rule.field.trim().to_string(),
            pattern_from_form(rule),
            rule.category.trim().to_string(),
            rule.budget_code.trim().to_string(),
            rule.direction.trim().to_string(),
            rule.amount_min.trim().to_string(),
            rule.amount_max.trim().to_string(),
            rule.notes.trim().to_string(),
        ])?;
    }

    writer_to_string(wtr)
}
