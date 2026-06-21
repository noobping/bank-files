use super::*;

pub(super) fn guess_field_map(
    headers: &[String],
    sample: &[Vec<String>],
    aliases: &FieldAliases,
) -> FieldMap {
    let mut map = FieldMap {
        date: best_header(headers, aliases, "date"),
        amount: best_header(headers, aliases, "amount"),
        debit: best_header(headers, aliases, "debit"),
        credit: best_header(headers, aliases, "credit"),
        description: best_header(headers, aliases, "description"),
        counterparty: best_header(headers, aliases, "counterparty"),
        tags: best_header(headers, aliases, "tags"),
        account: best_header(headers, aliases, "account"),
        transaction_id: best_header(headers, aliases, "transaction_id"),
        currency: best_header(headers, aliases, "currency"),
        direction: best_header(headers, aliases, "direction"),
    };

    if map.date.is_none() {
        map.date = infer_date_column(headers, sample);
    }

    if map.amount.is_none() && map.debit.is_none() && map.credit.is_none() {
        map.amount = infer_amount_column(headers, sample);
    }

    if map.description.is_none() {
        map.description = headers
            .iter()
            .find(|h| normalize_key(h).contains("omsch") || normalize_key(h).contains("desc"))
            .cloned();
    }

    map
}

fn best_header(headers: &[String], aliases: &FieldAliases, canonical: &str) -> Option<String> {
    let mut best: Option<(i32, String)> = None;
    for header in headers {
        let hn = normalize_key(header);
        let mut score = 0;
        for alias in aliases.get(canonical) {
            if hn == *alias {
                score = score.max(100);
            } else if hn.contains(alias) || alias.contains(&hn) {
                score = score.max(80);
            } else {
                let tokens = alias.split_whitespace().collect::<Vec<_>>();
                if !tokens.is_empty() && tokens.iter().all(|t| hn.contains(t)) {
                    score = score.max(65);
                }
            }
        }
        if score > 0 && best.as_ref().map(|b| score > b.0).unwrap_or(true) {
            best = Some((score, header.clone()));
        }
    }
    best.map(|(_, h)| h)
}

fn infer_date_column(headers: &[String], sample: &[Vec<String>]) -> Option<String> {
    let mut scores = vec![0usize; headers.len()];
    for record in sample.iter().take(50) {
        for (i, cell) in record.iter().enumerate() {
            if parse_date(cell).is_some() {
                scores[i] += 1;
            }
        }
    }
    scores
        .iter()
        .enumerate()
        .max_by_key(|(_, score)| *score)
        .and_then(|(i, score)| {
            if *score >= 3 {
                headers.get(i).cloned()
            } else {
                None
            }
        })
}

fn infer_amount_column(headers: &[String], sample: &[Vec<String>]) -> Option<String> {
    let mut scores = vec![0usize; headers.len()];
    for record in sample.iter().take(50) {
        for (i, cell) in record.iter().enumerate() {
            if parse_decimal(cell).is_some() && parse_date(cell).is_none() {
                scores[i] += 1;
            }
        }
    }
    scores
        .iter()
        .enumerate()
        .max_by_key(|(_, score)| *score)
        .and_then(|(i, score)| {
            if *score >= 3 {
                headers.get(i).cloned()
            } else {
                None
            }
        })
}
