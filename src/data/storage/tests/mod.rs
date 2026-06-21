use super::super::ensure_default_files;
use super::*;
use crate::model::{
    AppData, DataCacheStatus, DedupeMode, ImportReport, MonthKey, RememberMode, Transaction,
    TransactionLoadScope, TransactionSource,
};
use crate::util::{ensure_layout, AppDirs};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static CACHE_ENV_LOCK: Mutex<()> = Mutex::new(());

fn make_permissions_writable(permissions: &mut fs::Permissions) {
    #[cfg(unix)]
    permissions.set_mode(permissions.mode() | 0o200);
    #[cfg(not(unix))]
    permissions.set_readonly(false);
}

mod cache;
mod capabilities;
mod load;
mod reload;

fn unique_test_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("test clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("bank-files-{name}-{suffix}"))
}

fn import_report(source: &Path) -> ImportReport {
    ImportReport {
        source: source.to_path_buf(),
        rows_seen: 1,
        rows_imported: 1,
        ..ImportReport::default()
    }
}

fn transaction(source_file: &str, description: &str) -> Transaction {
    Transaction {
        date: NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid test date"),
        amount: Decimal::new(-1000, 2),
        description: description.to_string(),
        counterparty: String::new(),
        tags: String::new(),
        account: String::new(),
        transaction_id: String::new(),
        currency: "EUR".to_string(),
        source_file: source_file.to_string(),
        source_row: 2,
        category: "Uncategorized".to_string(),
        budget_code: String::new(),
        notes: String::new(),
        strict_key: format!("strict-{source_file}-{description}"),
        loose_key: format!("loose-{source_file}-{description}"),
        rule_match: None,
    }
}
