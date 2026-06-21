use crate::util::normalize_key;
use anyhow::{Context, Result};
use csv::Trim;
use std::collections::HashMap;
use std::path::Path;

const DEFAULT_FIELD_ALIASES: &str = include_str!("../../data/defaults/field_aliases.csv");
pub(super) const CASH_FLOW_DIRECTION_ALIASES: &str =
    include_str!("../../data/defaults/cash_flow_direction_aliases.tsv");

#[derive(Debug, Clone)]
pub struct FieldAliases {
    aliases: HashMap<String, Vec<String>>,
}

impl FieldAliases {
    pub fn load(config_dir: &Path) -> Result<Self> {
        let mut aliases = Self::defaults();
        let path = config_dir.join("field_aliases.csv");
        if path.exists() {
            let mut rdr = csv::ReaderBuilder::new()
                .flexible(true)
                .trim(Trim::All)
                .from_path(&path)
                .with_context(|| format!("Could not read field aliases: {}", path.display()))?;
            for row in rdr.records() {
                let row = row?;
                let canonical = row.get(0).unwrap_or_default().trim();
                let alias = row.get(1).unwrap_or_default().trim();
                if !canonical.is_empty() && !alias.is_empty() {
                    aliases.add(canonical, alias);
                }
            }
        }
        Ok(aliases)
    }

    pub(super) fn defaults() -> Self {
        let mut aliases = Self {
            aliases: HashMap::new(),
        };
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(Trim::All)
            .from_reader(DEFAULT_FIELD_ALIASES.as_bytes());
        for row in rdr.records().flatten() {
            let canonical = row.get(0).unwrap_or_default().trim();
            let alias = row.get(1).unwrap_or_default().trim();
            if canonical != "canonical" && !canonical.is_empty() && !alias.is_empty() {
                aliases.add(canonical, alias);
            }
        }
        aliases
    }

    fn add(&mut self, canonical: &str, alias: &str) {
        self.aliases
            .entry(canonical.to_string())
            .or_default()
            .push(normalize_key(alias));
    }

    pub(super) fn get(&self, canonical: &str) -> &[String] {
        self.aliases
            .get(canonical)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}
