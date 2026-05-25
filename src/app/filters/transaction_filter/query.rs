use super::super::super::tr;
use super::{TransactionAmountFilter, TransactionFilter};
use crate::model::MonthKey;

impl TransactionFilter {
    pub(in crate::app) fn query(&self) -> String {
        match self {
            Self::All => String::new(),
            Self::UnconfiguredBudgets | Self::OtherCategories => tr(self.label()),
            Self::CategoryForYear { category, year } => {
                format!("category:{} year:{year}", encode_filter_value(category))
            }
            Self::Scoped {
                budget_code,
                year,
                month,
                amount,
            } => {
                let mut tokens = Vec::new();
                if let Some(code) = budget_code {
                    tokens.push(format!("budget:{code}"));
                }
                if let Some(month) = month {
                    tokens.push(format!("month:{month}"));
                } else if let Some(year) = year {
                    tokens.push(format!("year:{year}"));
                }
                if let Some(amount) = amount {
                    tokens.push(match amount {
                        TransactionAmountFilter::Income => "amount:income".to_string(),
                        TransactionAmountFilter::Expense => "amount:expense".to_string(),
                        TransactionAmountFilter::Transfer => "amount:transfer".to_string(),
                    });
                }
                tokens.join(" ")
            }
        }
    }

    pub(in crate::app) fn from_query(text: &str) -> Option<Self> {
        let normalized = text.trim().to_lowercase();
        for filter in [Self::UnconfiguredBudgets, Self::OtherCategories] {
            if normalized == filter.label().to_lowercase()
                || normalized == tr(filter.label()).to_lowercase()
            {
                return Some(filter);
            }
        }

        let mut budget_code = None;
        let mut year = None;
        let mut month = None;
        let mut amount = None;
        let mut category = None;
        let mut recognized = false;

        for token in text.split_whitespace() {
            let Some((key, value)) = token.split_once(':') else {
                continue;
            };
            let value = value.trim();
            match key.trim().to_lowercase().as_str() {
                "budget" | "code" if !value.is_empty() => {
                    budget_code = Some(value.to_string());
                    recognized = true;
                }
                "year" => {
                    if let Ok(parsed) = value.parse::<i32>() {
                        year = Some(parsed);
                        recognized = true;
                    }
                }
                "month" => {
                    if let Some(parsed) = parse_month_filter(value) {
                        month = Some(parsed);
                        recognized = true;
                    }
                }
                "category" | "cat" if !value.is_empty() => {
                    category = Some(decode_filter_value(value));
                }
                "amount" | "type" => match value.to_lowercase().as_str() {
                    "income" | "positive" | "in" => {
                        amount = Some(TransactionAmountFilter::Income);
                        recognized = true;
                    }
                    "expense" | "expenses" | "cost" | "costs" | "negative" | "out" => {
                        amount = Some(TransactionAmountFilter::Expense);
                        recognized = true;
                    }
                    "transfer" | "transfers" => {
                        amount = Some(TransactionAmountFilter::Transfer);
                        recognized = true;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        if let (Some(category), Some(year)) = (category, year) {
            return Some(Self::CategoryForYear { category, year });
        }

        recognized.then_some(Self::Scoped {
            budget_code,
            year,
            month,
            amount,
        })
    }
}

fn encode_filter_value(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::new();
    for byte in value.trim().bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push('%');
                encoded.push(HEX[(byte >> 4) as usize] as char);
                encoded.push(HEX[(byte & 0x0f) as usize] as char);
            }
        }
    }
    encoded
}

fn decode_filter_value(value: &str) -> String {
    let mut decoded = Vec::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                if let (Some(high), Some(low)) =
                    (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
                {
                    decoded.push((high << 4) | low);
                    index += 3;
                } else {
                    decoded.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded).unwrap_or_else(|_| value.replace('+', " "))
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn parse_month_filter(value: &str) -> Option<MonthKey> {
    let (year, month) = value.split_once('-')?;
    Some(MonthKey::new(year.parse().ok()?, month.parse().ok()?))
}
