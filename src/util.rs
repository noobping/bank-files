use anyhow::{Context, Result};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use unicode_normalization::UnicodeNormalization;

const APP_DIR_NAME: &str = "bank-files";

#[derive(Debug, Clone)]
pub struct AppDirs {
    pub config: PathBuf,
    pub data: PathBuf,
    pub inbox: PathBuf,
}

pub fn app_dirs() -> Result<AppDirs> {
    if let Ok(path) = std::env::var("BANK_FILES_HOME") {
        let root = PathBuf::from(path);
        return Ok(AppDirs {
            config: root.join("config"),
            data: root.join("data"),
            inbox: root.join("data"),
        });
    }

    let config = dirs_next::config_dir()
        .context("Could not determine the default configuration folder")?
        .join(APP_DIR_NAME);
    let data = dirs_next::data_dir()
        .context("Could not determine the default data folder")?
        .join(APP_DIR_NAME);

    Ok(AppDirs {
        inbox: data.clone(),
        config,
        data,
    })
}

pub fn ensure_layout(dirs: &AppDirs) -> Result<()> {
    fs::create_dir_all(&dirs.config)?;
    fs::create_dir_all(&dirs.data)?;
    fs::create_dir_all(&dirs.inbox)?;
    Ok(())
}

pub fn normalize_key(input: &str) -> String {
    input
        .nfd()
        .filter(|c| !matches!(c, '\u{0300}'..='\u{036f}'))
        .flat_map(|c| c.to_lowercase())
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn parse_date(input: &str) -> Option<NaiveDate> {
    let s = input.trim().trim_matches('"').trim();
    if s.is_empty() {
        return None;
    }

    let formats = [
        "%Y-%m-%d", "%d-%m-%Y", "%d/%m/%Y", "%Y/%m/%d", "%d.%m.%Y", "%d-%m-%y", "%d/%m/%y",
        "%Y%m%d",
    ];

    for fmt in formats {
        if let Ok(date) = NaiveDate::parse_from_str(s, fmt) {
            return Some(date);
        }
    }

    // Excel/LibreOffice serial date. 25569 = 1970-01-01.
    if let Ok(days) = s.parse::<i64>() {
        if (25_000..80_000).contains(&days) {
            return NaiveDate::from_ymd_opt(1970, 1, 1)?
                .checked_add_signed(chrono::Duration::days(days - 25_569));
        }
    }

    None
}

pub fn parse_decimal(input: &str) -> Option<Decimal> {
    let mut s = input.trim().trim_matches('"').trim().to_string();
    if s.is_empty() || s == "-" {
        return None;
    }

    let mut negative = false;
    if s.starts_with('(') && s.ends_with(')') {
        negative = true;
        s = s.trim_start_matches('(').trim_end_matches(')').to_string();
    }

    let lower = s.to_lowercase();
    if lower.contains(" af") || lower.ends_with("af") || lower.contains("debet") {
        negative = true;
    }

    s = s
        .replace('€', "")
        .replace("eur", "")
        .replace("EUR", "")
        .replace(['\u{00a0}', ' ', '\''], "");

    if s.starts_with('-') || s.ends_with('-') || s.contains("-") {
        negative = true;
    }

    let filtered: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == ',' || *c == '.')
        .collect();

    if filtered.is_empty() {
        return None;
    }

    let comma_pos = filtered.rfind(',');
    let dot_pos = filtered.rfind('.');
    let decimal_sep = match (comma_pos, dot_pos) {
        (Some(c), Some(d)) => Some(if c > d { ',' } else { '.' }),
        (Some(_), None) => infer_single_separator(&filtered, ','),
        (None, Some(_)) => infer_single_separator(&filtered, '.'),
        (None, None) => None,
    };

    let mut normalized = String::new();
    for c in filtered.chars() {
        if c.is_ascii_digit() {
            normalized.push(c);
        } else if Some(c) == decimal_sep {
            normalized.push('.');
        }
    }

    if normalized.matches('.').count() > 1 {
        let last = normalized.rfind('.')?;
        normalized = normalized
            .chars()
            .enumerate()
            .filter_map(|(i, c)| if c == '.' && i != last { None } else { Some(c) })
            .collect();
    }

    if normalized.is_empty() {
        return None;
    }

    let mut value = Decimal::from_str(&normalized).ok()?;
    if negative {
        value = -value;
    }
    Some(value)
}

fn infer_single_separator(input: &str, sep: char) -> Option<char> {
    let count = input.matches(sep).count();
    if count == 0 {
        return None;
    }
    if count > 1 {
        // 1.234.567 or 1,234,567 usually means thousands only.
        return None;
    }
    let idx = input.rfind(sep)?;
    let digits_after = input[idx + sep.len_utf8()..]
        .chars()
        .filter(|c| c.is_ascii_digit())
        .count();
    if digits_after == 2 || digits_after == 1 {
        Some(sep)
    } else {
        None
    }
}

pub fn money(value: Decimal) -> String {
    format!("€ {}", value.round_dp(2))
}

pub fn signed_money(value: Decimal) -> String {
    if value >= Decimal::ZERO {
        format!("+{}", money(value))
    } else {
        format!("-{}", money(-value))
    }
}

pub fn pad_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let cols = headers.len();
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate().take(cols) {
            widths[i] = widths[i].max(cell.chars().count());
        }
    }

    let mut out = String::new();
    write_row(
        &mut out,
        headers.iter().map(|s| s.to_string()).collect(),
        &widths,
    );
    out.push_str(
        &widths
            .iter()
            .map(|w| "-".repeat(*w))
            .collect::<Vec<_>>()
            .join("  "),
    );
    out.push('\n');
    for row in rows {
        write_row(&mut out, row.clone(), &widths);
    }
    out
}

fn write_row(out: &mut String, cells: Vec<String>, widths: &[usize]) {
    for (i, width) in widths.iter().enumerate() {
        let cell = cells.get(i).cloned().unwrap_or_default();
        out.push_str(&format!("{:<width$}", cell, width = width));
        if i + 1 < widths.len() {
            out.push_str("  ");
        }
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parses_european_amounts() {
        assert_eq!(
            parse_decimal("€ 1.234,56").unwrap(),
            Decimal::from_str("1234.56").unwrap()
        );
        assert_eq!(
            parse_decimal("-1.234,56").unwrap(),
            Decimal::from_str("-1234.56").unwrap()
        );
        assert_eq!(
            parse_decimal("1,234.56").unwrap(),
            Decimal::from_str("1234.56").unwrap()
        );
    }

    #[test]
    fn parses_dates() {
        assert_eq!(
            parse_date("2024-03-01").unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()
        );
        assert_eq!(
            parse_date("01-03-2024").unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()
        );
    }
}
