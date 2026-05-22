use super::*;

pub(in crate::data) fn parse_editable_aliases(contents: &str) -> Result<Vec<EditableAlias>> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(contents.as_bytes());
    let headers = rdr
        .headers()
        .context("field_aliases.csv has no header")?
        .iter()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let mut aliases = Vec::new();

    for row in rdr.records() {
        let row = row?;
        let canonical = csv_cell(&headers, &row, "canonical");
        let alias = csv_cell(&headers, &row, "alias");
        if canonical.trim().is_empty() || alias.trim().is_empty() {
            continue;
        }
        aliases.push(EditableAlias { canonical, alias });
    }

    Ok(aliases)
}

pub(in crate::data) fn serialize_editable_aliases(aliases: &[EditableAlias]) -> Result<String> {
    let mut wtr = csv::WriterBuilder::new().from_writer(Vec::new());
    wtr.write_record(["canonical", "alias"])?;

    for alias in aliases
        .iter()
        .filter(|alias| !alias.canonical.trim().is_empty() && !alias.alias.trim().is_empty())
    {
        wtr.write_record([
            alias.canonical.trim().to_string(),
            alias.alias.trim().to_string(),
        ])?;
    }

    writer_to_string(wtr)
}
