use crate::i18n;
use crate::model::{
    FieldMap, ImportOutcome, ImportReport, MonthKey, Transaction, TransactionLoadScope,
};
use crate::util::{normalize_key, parse_date, parse_decimal, AppDirs};
use anyhow::{Context, Result};
use csv::{ByteRecord, ReaderBuilder, Trim};
use rust_decimal::Decimal;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod aliases;
mod fields;
mod import;
mod keys;
mod records;
#[cfg(test)]
mod tests;

pub use aliases::FieldAliases;
pub use import::{import_files, import_inbox};

use aliases::CASH_FLOW_DIRECTION_ALIASES;
use fields::guess_field_map;
use keys::{make_loose_key, make_strict_key};
use records::parse_record;
