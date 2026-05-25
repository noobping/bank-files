use super::*;

pub(super) fn parse_record(
    path: &Path,
    headers: &[String],
    record: &[String],
    map: &FieldMap,
    source_row: usize,
) -> Option<Transaction> {
    let date = parse_date(&value(headers, record, &map.date))?;
    let mut amount = parse_amount(headers, record, map)?;

    if let Some(direction_col) = &map.direction {
        let direction = normalize_key(&value(headers, record, &Some(direction_col.clone())));
        let direction_flips_amount = (amount > Decimal::ZERO && is_out_direction(&direction))
            || (amount < Decimal::ZERO && is_in_direction(&direction));
        if direction_flips_amount {
            amount = -amount;
        }
    }

    let description = non_empty_or(
        value(headers, record, &map.description),
        fallback_description(headers, record, map),
    );
    let counterparty = value(headers, record, &map.counterparty);
    let tags = value(headers, record, &map.tags);
    let account = value(headers, record, &map.account);
    let transaction_id = value(headers, record, &map.transaction_id);
    let currency = non_empty_or(value(headers, record, &map.currency), "EUR".to_string());
    let source_file = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());

    let strict_key = make_strict_key(
        date,
        amount,
        &description,
        &counterparty,
        &account,
        &transaction_id,
    );
    let loose_key = make_loose_key(date, amount, &description, &counterparty);

    Some(Transaction {
        date,
        amount,
        description,
        counterparty,
        tags,
        account,
        transaction_id,
        currency,
        source_file,
        source_row,
        category: "Uncategorized".to_string(),
        budget_code: String::new(),
        notes: String::new(),
        strict_key,
        loose_key,
        rule_match: None,
    })
}

fn parse_amount(headers: &[String], record: &[String], map: &FieldMap) -> Option<Decimal> {
    if let Some(amount_col) = &map.amount {
        if let Some(value) = parse_decimal(&value(headers, record, &Some(amount_col.clone()))) {
            return Some(value);
        }
    }

    let debit = map
        .debit
        .as_ref()
        .and_then(|col| parse_decimal(&value(headers, record, &Some(col.clone()))))
        .unwrap_or(Decimal::ZERO)
        .abs();
    let credit = map
        .credit
        .as_ref()
        .and_then(|col| parse_decimal(&value(headers, record, &Some(col.clone()))))
        .unwrap_or(Decimal::ZERO)
        .abs();

    if credit != Decimal::ZERO || debit != Decimal::ZERO {
        Some(credit - debit)
    } else {
        None
    }
}

fn value(headers: &[String], record: &[String], col: &Option<String>) -> String {
    let Some(col) = col else {
        return String::new();
    };
    let Some(index) = headers.iter().position(|h| h == col) else {
        return String::new();
    };
    record
        .get(index)
        .map(String::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn fallback_description(headers: &[String], record: &[String], map: &FieldMap) -> String {
    let mut parts = Vec::new();
    for (i, cell) in record.iter().enumerate() {
        let header = headers.get(i).cloned().unwrap_or_default();
        let skip = [
            map.date.as_ref(),
            map.amount.as_ref(),
            map.debit.as_ref(),
            map.credit.as_ref(),
            map.account.as_ref(),
            map.tags.as_ref(),
            map.currency.as_ref(),
        ]
        .iter()
        .any(|c| c.map(|v| v == &header).unwrap_or(false));
        if !skip && !cell.trim().is_empty() {
            parts.push(cell.trim().to_string());
        }
    }
    parts.join(" | ")
}

fn non_empty_or(value: String, fallback: String) -> String {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}

fn is_out_direction(direction: &str) -> bool {
    cash_flow_direction_matches(direction, "out")
}

fn is_in_direction(direction: &str) -> bool {
    cash_flow_direction_matches(direction, "in")
}

fn cash_flow_direction_matches(direction: &str, canonical: &str) -> bool {
    CASH_FLOW_DIRECTION_ALIASES
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut cols = line.splitn(3, '\t');
            Some((
                cols.next()?.trim(),
                cols.next()?.trim(),
                cols.next()?.trim(),
            ))
        })
        .find(|(name, _, _)| *name == canonical)
        .map(|(_, exact, contains)| {
            exact
                .split('|')
                .any(|alias| direction == normalize_key(alias))
                || contains
                    .split('|')
                    .any(|alias| direction.contains(&normalize_key(alias)))
        })
        .unwrap_or(false)
}
