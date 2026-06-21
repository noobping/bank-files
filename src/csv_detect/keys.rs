use super::*;

pub(super) fn make_strict_key(
    date: chrono::NaiveDate,
    amount: Decimal,
    desc: &str,
    counterparty: &str,
    account: &str,
    txid: &str,
) -> String {
    if !txid.trim().is_empty() {
        hash_key(&[
            "id",
            txid,
            account,
            &date.to_string(),
            &amount.round_dp(2).to_string(),
        ])
    } else {
        hash_key(&[
            "strict",
            account,
            &date.to_string(),
            &amount.round_dp(2).to_string(),
            &normalize_key(desc),
            &normalize_key(counterparty),
        ])
    }
}

pub(super) fn make_loose_key(
    date: chrono::NaiveDate,
    amount: Decimal,
    desc: &str,
    counterparty: &str,
) -> String {
    hash_key(&[
        "loose",
        &date.to_string(),
        &amount.round_dp(2).to_string(),
        &normalize_key(desc),
        &normalize_key(counterparty),
    ])
}

fn hash_key(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0]);
    }
    let digest = hasher.finalize();
    hex::encode(&digest[..12])
}
