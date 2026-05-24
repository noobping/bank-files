use super::super::*;
use super::files::{read_config_file, write_config_file};

pub(super) fn load_ignored_transaction_patterns() -> Result<Vec<IgnoredTransactionPattern>> {
    let (_, contents) = read_config_file("ignored_transaction_patterns.csv")?;
    parse_ignored_transaction_patterns(&contents)
}

pub fn ignored_transaction_pattern_keys() -> Result<HashSet<String>> {
    Ok(load_ignored_transaction_patterns()?
        .into_iter()
        .map(|pattern| pattern.key)
        .collect())
}

pub fn ignore_transaction_pattern(key: &str, label: &str) -> Result<bool> {
    let key = key.trim();
    if key.is_empty() {
        anyhow::bail!("Ignored transaction pattern needs a key");
    }

    let mut patterns = load_ignored_transaction_patterns()?;
    if patterns.iter().any(|pattern| pattern.key == key) {
        return Ok(false);
    }

    patterns.push(IgnoredTransactionPattern {
        key: key.to_string(),
        label: label.trim().to_string(),
    });
    patterns.sort_by(|left, right| left.label.cmp(&right.label).then(left.key.cmp(&right.key)));
    write_ignored_transaction_patterns(&patterns)?;
    Ok(true)
}

pub fn reopen_transaction_pattern(key: &str) -> Result<bool> {
    let mut patterns = load_ignored_transaction_patterns()?;
    let original_len = patterns.len();
    patterns.retain(|pattern| pattern.key != key.trim());
    let changed = patterns.len() != original_len;
    if changed {
        write_ignored_transaction_patterns(&patterns)?;
    }
    Ok(changed)
}

pub(super) fn write_ignored_transaction_patterns(
    patterns: &[IgnoredTransactionPattern],
) -> Result<PathBuf> {
    let contents = serialize_ignored_transaction_patterns(patterns)?;
    write_config_file("ignored_transaction_patterns.csv", &contents)
}

pub(super) fn parse_ignored_transaction_patterns(
    contents: &str,
) -> Result<Vec<IgnoredTransactionPattern>> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(contents.as_bytes());
    let headers = rdr
        .headers()
        .context("ignored_transaction_patterns.csv has no header")?
        .iter()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let mut patterns = Vec::new();
    let mut seen = HashSet::new();

    for row in rdr.records() {
        let row = row?;
        let key = csv_cell(&headers, &row, "key");
        if key.trim().is_empty() || !seen.insert(key.clone()) {
            continue;
        }
        patterns.push(IgnoredTransactionPattern {
            key,
            label: csv_cell(&headers, &row, "label"),
        });
    }

    Ok(patterns)
}

pub(super) fn serialize_ignored_transaction_patterns(
    patterns: &[IgnoredTransactionPattern],
) -> Result<String> {
    let mut wtr = csv::WriterBuilder::new().from_writer(Vec::new());
    wtr.write_record(["key", "label"])?;

    for pattern in patterns
        .iter()
        .filter(|pattern| !pattern.key.trim().is_empty())
    {
        wtr.write_record([pattern.key.trim(), pattern.label.trim()])?;
    }

    writer_to_string(wtr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignored_transaction_patterns_roundtrip_and_dedupe_keys() {
        let contents = "key,label\nrefund:coffee,Coffee refund\nrefund:coffee,Duplicate\ntransfer:rent,Rent transfer\n";

        let patterns = parse_ignored_transaction_patterns(contents).unwrap();

        assert_eq!(patterns.len(), 2);
        assert_eq!(patterns[0].key, "refund:coffee");
        assert_eq!(patterns[0].label, "Coffee refund");
        assert_eq!(patterns[1].key, "transfer:rent");

        let serialized = serialize_ignored_transaction_patterns(&patterns).unwrap();
        assert!(serialized.contains("refund:coffee"));
        assert!(serialized.contains("Coffee refund"));
        assert!(!serialized.contains("Duplicate"));
    }
}
