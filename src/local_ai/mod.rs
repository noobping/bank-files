use crate::data::{
    self, EditableAlias, EditableBudget, EditableRule, GeneratedConfiguration,
    GeneratedConfigurationSummary, IgnoredTransactionPattern,
};
use crate::model::{AppData, Transaction};
#[cfg(feature = "embedded-ai-model")]
use crate::util::app_cache_dir;
use crate::util::normalize_key;
use anyhow::{Context, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
#[cfg(feature = "embedded-ai-model")]
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

const MODEL_ROOT: &str = "models/ai";
#[cfg(target_os = "linux")]
const INSTALLED_MODEL_ROOT: &str = "bank-files/models/ai";
const MANIFEST_FILE: &str = "model.json";
const MODEL_FILES: [&str; 5] = [
    "classifier.onnx",
    "generator.safetensors",
    "tokenizer.json",
    "model.json",
    "LICENSE",
];
const LOCAL_AI_NOTE: &str = "Generated with local AI smart insights.";

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LocalAiAvailability {
    Disabled,
    Available { source: LocalAiModelSource },
    MissingModel(String),
    RuntimeError(String),
}

impl LocalAiAvailability {
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available { .. })
    }

    pub fn status_message(&self) -> Option<String> {
        match self {
            Self::Disabled | Self::Available { .. } => None,
            Self::MissingModel(reason) => Some(format!(
                "Local AI unavailable: {reason}. Using deterministic Smart Insights."
            )),
            Self::RuntimeError(reason) => Some(format!(
                "Local AI failed: {reason}. Using deterministic Smart Insights."
            )),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LocalAiModelSource {
    Sidecar(PathBuf),
    #[cfg(target_os = "linux")]
    Installed(PathBuf),
    #[cfg(feature = "embedded-ai-model")]
    Extracted(PathBuf),
}

impl LocalAiModelSource {
    fn path(&self) -> &Path {
        match self {
            Self::Sidecar(path) => path,
            #[cfg(target_os = "linux")]
            Self::Installed(path) => path,
            #[cfg(feature = "embedded-ai-model")]
            Self::Extracted(path) => path,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LocalAiError {
    message: String,
}

impl fmt::Display for LocalAiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for LocalAiError {}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalAiInput {
    pub locale: String,
    pub records: Vec<LocalAiRecord>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalAiRecord {
    pub label: String,
    pub descriptions: Vec<String>,
    pub tags: Vec<String>,
    pub direction: String,
    pub existing_category: String,
    pub existing_budget_code: String,
    pub count: usize,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalAiDraft {
    #[serde(default)]
    pub budgets: Vec<EditableBudget>,
    #[serde(default)]
    pub rules: Vec<EditableRule>,
    #[serde(default)]
    pub aliases: Vec<EditableAlias>,
    #[serde(default)]
    pub ignored_patterns: Vec<IgnoredTransactionPattern>,
    #[serde(default)]
    pub pattern_hints: Vec<LocalAiPatternHint>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalAiPatternHint {
    pub label: String,
    pub category: String,
    pub budget_code: String,
    pub reason: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LocalAiConfigurationOutcome {
    pub configuration: GeneratedConfiguration,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct LocalAiPatternHintOutcome {
    pub hints: Vec<LocalAiPatternHint>,
    pub status: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LocalAiProvider {
    availability: LocalAiAvailability,
}

impl LocalAiProvider {
    pub fn new(smart_insights_enabled: bool) -> Self {
        Self {
            availability: local_ai_availability(smart_insights_enabled),
        }
    }

    pub fn availability(&self) -> &LocalAiAvailability {
        &self.availability
    }

    pub fn generate_configuration_draft(
        &self,
        input: &LocalAiInput,
    ) -> std::result::Result<Option<LocalAiDraft>, LocalAiError> {
        if !self.availability.is_available() {
            return Ok(None);
        }
        runtime_generate_configuration_draft(input)
    }

    pub fn pattern_hints(
        &self,
        input: &LocalAiInput,
    ) -> std::result::Result<Vec<LocalAiPatternHint>, LocalAiError> {
        if !self.availability.is_available() {
            return Ok(Vec::new());
        }
        runtime_pattern_hints(input)
    }
}

pub fn enhance_generated_configuration(
    data: &AppData,
    deterministic: GeneratedConfiguration,
    smart_insights_enabled: bool,
) -> LocalAiConfigurationOutcome {
    if !cfg!(feature = "local-ai") {
        return LocalAiConfigurationOutcome {
            configuration: deterministic,
            status: None,
        };
    }

    let provider = LocalAiProvider::new(smart_insights_enabled);
    let Some(status) = provider.availability().status_message() else {
        let input = LocalAiInput::from_app_data(data, system_locale());
        return match provider.generate_configuration_draft(&input) {
            Ok(Some(draft)) => match generated_configuration_from_draft(draft, &deterministic) {
                Ok(Some(configuration)) => LocalAiConfigurationOutcome {
                    configuration,
                    status: Some("Local AI generated a validated smart configuration draft.".to_string()),
                },
                Ok(None) => LocalAiConfigurationOutcome {
                    configuration: deterministic,
                    status: Some(
                        "Local AI returned no usable configuration. Using deterministic Smart Insights."
                            .to_string(),
                    ),
                },
                Err(error) => LocalAiConfigurationOutcome {
                    configuration: deterministic,
                    status: Some(format!(
                        "Local AI draft was rejected: {error}. Using deterministic Smart Insights."
                    )),
                },
            },
            Ok(None) => LocalAiConfigurationOutcome {
                configuration: deterministic,
                status: None,
            },
            Err(error) => LocalAiConfigurationOutcome {
                configuration: deterministic,
                status: Some(format!(
                    "Local AI failed: {error}. Using deterministic Smart Insights."
                )),
            },
        };
    };

    LocalAiConfigurationOutcome {
        configuration: deterministic,
        status: Some(status),
    }
}

pub fn transaction_pattern_hints(
    data: &AppData,
    smart_insights_enabled: bool,
) -> LocalAiPatternHintOutcome {
    if !cfg!(feature = "local-ai") || !smart_insights_enabled {
        return LocalAiPatternHintOutcome::default();
    }

    let provider = LocalAiProvider::new(smart_insights_enabled);
    if provider.availability().status_message().is_some() {
        return LocalAiPatternHintOutcome::default();
    }

    let input = LocalAiInput::from_app_data(data, system_locale());
    match provider.pattern_hints(&input) {
        Ok(hints) => LocalAiPatternHintOutcome {
            hints,
            status: None,
        },
        Err(error) => LocalAiPatternHintOutcome {
            hints: Vec::new(),
            status: Some(format!(
                "Local AI pattern hints failed: {error}. Using deterministic pattern detection."
            )),
        },
    }
}

pub fn pattern_hint_for_label<'a>(
    hints: &'a [LocalAiPatternHint],
    label: &str,
) -> Option<&'a LocalAiPatternHint> {
    let key = normalize_key(label);
    hints
        .iter()
        .find(|hint| normalize_key(&hint.label) == key && !hint.category.trim().is_empty())
}

impl LocalAiInput {
    pub fn from_app_data(data: &AppData, locale: impl Into<String>) -> Self {
        let mut groups = BTreeMap::<(String, String), LocalAiRecordBuilder>::new();
        for transaction in &data.transactions {
            let Some(label) = transaction_label(transaction) else {
                continue;
            };
            let direction = transaction_direction(transaction.amount).to_string();
            groups
                .entry((normalize_key(&label), direction.clone()))
                .or_insert_with(|| LocalAiRecordBuilder::new(label, direction))
                .push(transaction);
        }

        let records = groups
            .into_values()
            .map(LocalAiRecordBuilder::finish)
            .collect::<Vec<_>>();

        Self {
            locale: locale.into(),
            records,
        }
    }
}

#[derive(Debug, Clone)]
struct LocalAiRecordBuilder {
    label: String,
    descriptions: BTreeSet<String>,
    tags: BTreeSet<String>,
    direction: String,
    existing_category: String,
    existing_budget_code: String,
    count: usize,
}

impl LocalAiRecordBuilder {
    fn new(label: String, direction: String) -> Self {
        Self {
            label,
            descriptions: BTreeSet::new(),
            tags: BTreeSet::new(),
            direction,
            existing_category: String::new(),
            existing_budget_code: String::new(),
            count: 0,
        }
    }

    fn push(&mut self, transaction: &Transaction) {
        self.count += 1;
        push_sanitized(&mut self.descriptions, &transaction.description, 4);
        for tag in transaction.tags.split([',', ';', '|']) {
            push_sanitized(&mut self.tags, tag, 6);
        }
        if self.existing_category.is_empty() && !transaction.category.trim().is_empty() {
            self.existing_category = transaction.category.trim().to_string();
        }
        if self.existing_budget_code.is_empty() && !transaction.budget_code.trim().is_empty() {
            self.existing_budget_code = transaction.budget_code.trim().to_string();
        }
    }

    fn finish(self) -> LocalAiRecord {
        LocalAiRecord {
            label: self.label,
            descriptions: self.descriptions.into_iter().collect(),
            tags: self.tags.into_iter().collect(),
            direction: self.direction,
            existing_category: self.existing_category,
            existing_budget_code: self.existing_budget_code,
            count: self.count,
        }
    }
}

fn push_sanitized(target: &mut BTreeSet<String>, input: &str, limit: usize) {
    if target.len() >= limit {
        return;
    }
    let value = sanitize_text(input);
    if !value.is_empty() {
        target.insert(value);
    }
}

fn sanitize_text(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|token| !sensitive_token(token))
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(120)
        .collect::<String>()
        .trim()
        .to_string()
}

fn sensitive_token(token: &str) -> bool {
    let digits = token.chars().filter(|ch| ch.is_ascii_digit()).count();
    digits >= 6 || token.contains('@') || token.starts_with("IBAN") || token.starts_with("iban")
}

fn transaction_label(transaction: &Transaction) -> Option<String> {
    [
        transaction.counterparty.trim(),
        transaction.description.trim(),
        transaction.tags.trim(),
    ]
    .into_iter()
    .find(|value| !value.is_empty())
    .map(sanitize_text)
    .filter(|value| !value.is_empty())
}

fn transaction_direction(amount: Decimal) -> &'static str {
    if amount > Decimal::ZERO {
        "income"
    } else if amount < Decimal::ZERO {
        "expense"
    } else {
        "any"
    }
}

fn generated_configuration_from_draft(
    draft: LocalAiDraft,
    fallback: &GeneratedConfiguration,
) -> Result<Option<GeneratedConfiguration>> {
    if draft.budgets.is_empty()
        && draft.rules.is_empty()
        && draft.aliases.is_empty()
        && draft.ignored_patterns.is_empty()
    {
        return Ok(None);
    }

    let summary = GeneratedConfigurationSummary {
        budgets: draft.budgets.len(),
        rules: draft.rules.len(),
        field_mappings: draft.aliases.len(),
        ignored_patterns: draft.ignored_patterns.len(),
        complete_years: fallback.summary.complete_years,
        budget_months: fallback.summary.budget_months,
    };
    let mut configuration = GeneratedConfiguration {
        rules: draft.rules,
        budgets: draft.budgets,
        aliases: draft.aliases,
        ignored_patterns: draft.ignored_patterns,
        summary,
    };
    normalize_generated_notes(&mut configuration);
    data::validate_generated_configuration(&configuration)?;
    Ok(Some(configuration))
}

fn normalize_generated_notes(configuration: &mut GeneratedConfiguration) {
    for budget in &mut configuration.budgets {
        budget.notes = local_ai_note_with_detail(&budget.notes);
    }
    for rule in &mut configuration.rules {
        rule.notes = local_ai_note_with_detail(&rule.notes);
    }
}

fn local_ai_note_with_detail(notes: &str) -> String {
    let notes = notes.trim();
    if notes.is_empty() || notes == LOCAL_AI_NOTE {
        LOCAL_AI_NOTE.to_string()
    } else {
        format!("{LOCAL_AI_NOTE} {notes}")
    }
}

fn system_locale() -> String {
    std::env::var("LANG")
        .ok()
        .and_then(|locale| locale.split('.').next().map(str::to_string))
        .filter(|locale| !locale.trim().is_empty())
        .unwrap_or_else(|| "en".to_string())
}

fn local_ai_availability(smart_insights_enabled: bool) -> LocalAiAvailability {
    if !smart_insights_enabled || !cfg!(feature = "local-ai") {
        return LocalAiAvailability::Disabled;
    }

    let source_availability = local_ai_model_sources_availability(local_ai_model_sources());
    match source_availability {
        Some(LocalAiAvailability::Available { source }) => {
            return LocalAiAvailability::Available { source }
        }
        Some(LocalAiAvailability::RuntimeError(error)) => {
            return LocalAiAvailability::RuntimeError(error)
        }
        _ => {}
    }

    #[cfg(feature = "embedded-ai-model")]
    {
        embedded_model_availability()
    }

    #[cfg(not(feature = "embedded-ai-model"))]
    {
        let missing_model_reason = match source_availability {
            Some(LocalAiAvailability::MissingModel(reason)) => Some(reason),
            _ => None,
        };
        LocalAiAvailability::MissingModel(missing_model_reason.unwrap_or_else(|| {
            format!(
                "model files were not found next to the executable or in the installed data directories ({})",
                MODEL_ROOT
            )
        }))
    }
}

fn local_ai_model_sources_availability(
    sources: Vec<LocalAiModelSource>,
) -> Option<LocalAiAvailability> {
    let mut missing_model_reason = None;
    for source in sources {
        match model_dir_availability(source) {
            Some(LocalAiAvailability::Available { source }) => {
                return Some(LocalAiAvailability::Available { source })
            }
            Some(LocalAiAvailability::RuntimeError(error)) => {
                return Some(LocalAiAvailability::RuntimeError(error))
            }
            Some(LocalAiAvailability::MissingModel(reason)) => {
                missing_model_reason.get_or_insert(reason);
            }
            _ => {}
        }
    }
    missing_model_reason.map(LocalAiAvailability::MissingModel)
}

fn local_ai_model_sources() -> Vec<LocalAiModelSource> {
    let mut sources = Vec::new();
    if let Some(dir) = sidecar_model_dir() {
        sources.push(LocalAiModelSource::Sidecar(dir));
    }
    #[cfg(target_os = "linux")]
    sources.extend(
        installed_model_dirs()
            .into_iter()
            .map(LocalAiModelSource::Installed),
    );
    sources
}

fn sidecar_model_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join(MODEL_ROOT)))
}

#[cfg(target_os = "linux")]
fn installed_model_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(datadir) = option_env!("BANK_FILES_DATADIR") {
        push_installed_model_dir(&mut dirs, Path::new(datadir));
    }

    if let Some(data_dirs) = std::env::var_os("XDG_DATA_DIRS") {
        for data_dir in std::env::split_paths(&data_dirs) {
            push_installed_model_dir(&mut dirs, &data_dir);
        }
    } else {
        push_installed_model_dir(&mut dirs, Path::new("/usr/local/share"));
        push_installed_model_dir(&mut dirs, Path::new("/usr/share"));
    }

    dirs
}

#[cfg(target_os = "linux")]
fn push_installed_model_dir(dirs: &mut Vec<PathBuf>, data_dir: &Path) {
    let dir = installed_model_dir_from_data_dir(data_dir);
    if !dirs.iter().any(|existing| existing == &dir) {
        dirs.push(dir);
    }
}

#[cfg(target_os = "linux")]
fn installed_model_dir_from_data_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(INSTALLED_MODEL_ROOT)
}

fn model_dir_availability(source: LocalAiModelSource) -> Option<LocalAiAvailability> {
    let dir = source.path();
    if !MODEL_FILES.iter().all(|file| dir.join(file).is_file()) {
        return None;
    }

    match model_manifest_is_placeholder(dir) {
        Ok(true) => Some(LocalAiAvailability::MissingModel(format!(
            "{} contains placeholder model assets",
            dir.display()
        ))),
        Ok(false) => Some(LocalAiAvailability::Available { source }),
        Err(error) => Some(LocalAiAvailability::RuntimeError(format!("{error:#}"))),
    }
}

fn model_manifest_is_placeholder(dir: &Path) -> Result<bool> {
    let manifest = fs::read_to_string(dir.join(MANIFEST_FILE))
        .with_context(|| format!("Could not read {}", dir.join(MANIFEST_FILE).display()))?;
    let manifest: AiModelManifest = serde_json::from_str(&manifest)
        .with_context(|| format!("Could not parse {}", dir.join(MANIFEST_FILE).display()))?;
    Ok(manifest.placeholder)
}

#[derive(Debug, Deserialize)]
struct AiModelManifest {
    #[serde(default)]
    placeholder: bool,
}

#[cfg(feature = "embedded-ai-model")]
fn embedded_model_availability() -> LocalAiAvailability {
    match embedded_manifest_is_placeholder() {
        Ok(true) => {
            LocalAiAvailability::MissingModel("embedded model assets are placeholders".to_string())
        }
        Ok(false) => match extract_embedded_assets() {
            Ok(path) => LocalAiAvailability::Available {
                source: LocalAiModelSource::Extracted(path),
            },
            Err(error) => LocalAiAvailability::RuntimeError(format!("{error:#}")),
        },
        Err(error) => LocalAiAvailability::RuntimeError(format!("{error:#}")),
    }
}

#[cfg(feature = "embedded-ai-model")]
fn embedded_manifest_is_placeholder() -> Result<bool> {
    let manifest = std::str::from_utf8(embedded_asset_bytes(MANIFEST_FILE)?)
        .context("Embedded local AI manifest is not UTF-8")?;
    let manifest: AiModelManifest =
        serde_json::from_str(manifest).context("Could not parse embedded local AI manifest")?;
    Ok(manifest.placeholder)
}

#[cfg(feature = "embedded-ai-model")]
fn extract_embedded_assets() -> Result<PathBuf> {
    let hash = embedded_assets_hash();
    let mut candidates = Vec::new();
    if let Ok(cache) = app_cache_dir() {
        candidates.push(cache.join("local-ai").join(&hash));
    }
    candidates.push(std::env::temp_dir().join("bank-files-local-ai").join(&hash));

    let mut last_error = None;
    for dir in candidates {
        match write_embedded_assets_to(&dir) {
            Ok(()) => return Ok(dir),
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("No local AI extraction path available")))
}

#[cfg(feature = "embedded-ai-model")]
fn write_embedded_assets_to(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir)
        .with_context(|| format!("Could not create local AI model folder {}", dir.display()))?;
    for asset in embedded_assets() {
        fs::write(dir.join(asset.name), asset.bytes)
            .with_context(|| format!("Could not write local AI model asset {}", asset.name))?;
    }
    Ok(())
}

#[cfg(feature = "embedded-ai-model")]
fn embedded_assets_hash() -> String {
    let mut hasher = Sha256::new();
    for asset in embedded_assets() {
        hasher.update(asset.name.as_bytes());
        hasher.update(asset.bytes);
    }
    hex::encode(hasher.finalize())
}

#[cfg(feature = "embedded-ai-model")]
fn embedded_asset_bytes(name: &str) -> Result<&'static [u8]> {
    embedded_assets()
        .iter()
        .find(|asset| asset.name == name)
        .map(|asset| asset.bytes)
        .with_context(|| format!("Embedded local AI asset missing: {name}"))
}

#[cfg(feature = "embedded-ai-model")]
#[derive(Debug, Clone, Copy)]
struct EmbeddedAiAsset {
    name: &'static str,
    bytes: &'static [u8],
}

#[cfg(feature = "embedded-ai-model")]
fn embedded_assets() -> &'static [EmbeddedAiAsset] {
    &[
        EmbeddedAiAsset {
            name: "classifier.onnx",
            bytes: include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/ai/classifier.onnx"
            )),
        },
        EmbeddedAiAsset {
            name: "generator.safetensors",
            bytes: include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/ai/generator.safetensors"
            )),
        },
        EmbeddedAiAsset {
            name: "tokenizer.json",
            bytes: include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/ai/tokenizer.json"
            )),
        },
        EmbeddedAiAsset {
            name: "model.json",
            bytes: include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/ai/model.json")),
        },
        EmbeddedAiAsset {
            name: "LICENSE",
            bytes: include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/ai/LICENSE")),
        },
    ]
}

#[cfg(feature = "local-ai")]
fn runtime_generate_configuration_draft(
    _input: &LocalAiInput,
) -> std::result::Result<Option<LocalAiDraft>, LocalAiError> {
    let _ = std::any::type_name::<candle_core::Device>();
    let _ = std::any::type_name::<tokenizers::Tokenizer>();
    let _ = tract_onnx::onnx();
    Ok(None)
}

#[cfg(not(feature = "local-ai"))]
fn runtime_generate_configuration_draft(
    _input: &LocalAiInput,
) -> std::result::Result<Option<LocalAiDraft>, LocalAiError> {
    Ok(None)
}

#[cfg(feature = "local-ai")]
fn runtime_pattern_hints(
    _input: &LocalAiInput,
) -> std::result::Result<Vec<LocalAiPatternHint>, LocalAiError> {
    let _ = std::any::type_name::<candle_core::Device>();
    let _ = std::any::type_name::<tokenizers::Tokenizer>();
    let _ = tract_onnx::onnx();
    Ok(Vec::new())
}

#[cfg(not(feature = "local-ai"))]
fn runtime_pattern_hints(
    _input: &LocalAiInput,
) -> std::result::Result<Vec<LocalAiPatternHint>, LocalAiError> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn smart_insights_off_disables_local_ai() {
        let provider = LocalAiProvider::new(false);
        assert_eq!(provider.availability(), &LocalAiAvailability::Disabled);
    }

    #[test]
    fn input_sanitization_excludes_sensitive_fields() {
        let mut data = AppData::default();
        data.transactions.push(transaction(
            "2025-01-01",
            "-12.34",
            "Card 123456789 groceries jane@example.test",
            "Market 123456789",
            "food;weekly",
            "NL00BANK0123456789",
            "secret-row-id",
        ));

        let input = LocalAiInput::from_app_data(&data, "nl_NL");
        let json = serde_json::to_string(&input).expect("input should serialize");
        assert!(json.contains("Market"));
        assert!(json.contains("groceries"));
        assert!(!json.contains("123456789"));
        assert!(!json.contains("jane@example.test"));
        assert!(!json.contains("NL00BANK"));
        assert!(!json.contains("secret-row-id"));
        assert!(!json.contains("2025-01-01"));
        assert!(!json.contains("12.34"));
    }

    #[test]
    fn valid_ai_draft_becomes_generated_configuration() {
        let fallback = GeneratedConfiguration {
            rules: Vec::new(),
            budgets: Vec::new(),
            aliases: Vec::new(),
            ignored_patterns: Vec::new(),
            summary: GeneratedConfigurationSummary {
                complete_years: 1,
                budget_months: 12,
                ..GeneratedConfigurationSummary::default()
            },
        };
        let draft = serde_json::from_str::<LocalAiDraft>(
            r#"{
                "budgets": [{
                    "code": "FOOD",
                    "category": "Food",
                    "monthly_budget": "100",
                    "yearly_budget": "",
                    "direction": "expense",
                    "income_basis": "real",
                    "notes": ""
                }],
                "rules": [{
                    "priority": 120,
                    "active": true,
                    "field": "counterparty",
                    "search": "Market",
                    "is_regex": false,
                    "category": "Food",
                    "budget_code": "FOOD",
                    "direction": "expense",
                    "amount_min": "",
                    "amount_max": "",
                    "notes": ""
                }],
                "aliases": [],
                "ignored_patterns": []
            }"#,
        )
        .expect("draft should parse");

        let generated = generated_configuration_from_draft(draft, &fallback)
            .expect("draft should validate")
            .expect("draft should produce config");
        assert_eq!(generated.summary.budgets, 1);
        assert_eq!(generated.summary.rules, 1);
        assert_eq!(generated.summary.complete_years, 1);
        assert_eq!(generated.budgets[0].notes, LOCAL_AI_NOTE);
    }

    #[test]
    fn invalid_ai_regex_is_rejected() {
        let fallback = GeneratedConfiguration {
            rules: Vec::new(),
            budgets: vec![EditableBudget {
                code: "FOOD".to_string(),
                category: "Food".to_string(),
                monthly_budget: "100".to_string(),
                yearly_budget: String::new(),
                direction: "expense".to_string(),
                income_basis: "real".to_string(),
                notes: String::new(),
            }],
            aliases: Vec::new(),
            ignored_patterns: Vec::new(),
            summary: GeneratedConfigurationSummary::default(),
        };
        let draft = LocalAiDraft {
            budgets: fallback.budgets.clone(),
            rules: vec![EditableRule {
                priority: 120,
                active: true,
                field: "counterparty".to_string(),
                search: "(".to_string(),
                is_regex: true,
                category: "Food".to_string(),
                budget_code: "FOOD".to_string(),
                direction: "expense".to_string(),
                amount_min: String::new(),
                amount_max: String::new(),
                notes: String::new(),
            }],
            aliases: Vec::new(),
            ignored_patterns: Vec::new(),
            pattern_hints: Vec::new(),
        };

        assert!(generated_configuration_from_draft(draft, &fallback).is_err());
    }

    #[test]
    fn placeholder_model_dir_is_not_available() {
        let root = unique_test_dir("placeholder-model");
        let dir = root.join(MODEL_ROOT);
        write_test_model_dir(&dir, true);

        let availability = model_dir_availability(LocalAiModelSource::Sidecar(dir))
            .expect("complete model dir should produce an availability value");
        assert!(matches!(availability, LocalAiAvailability::MissingModel(_)));
    }

    #[test]
    fn source_scan_reports_placeholder_assets_before_generic_missing_message() {
        let root = unique_test_dir("placeholder-source-scan");
        let dir = root.join(MODEL_ROOT);
        write_test_model_dir(&dir, true);

        let availability =
            local_ai_model_sources_availability(vec![LocalAiModelSource::Sidecar(dir)]);

        assert!(matches!(
            availability,
            Some(LocalAiAvailability::MissingModel(reason))
                if reason.contains("placeholder model assets")
        ));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn installed_model_dir_uses_linux_data_layout() {
        assert_eq!(
            installed_model_dir_from_data_dir(std::path::Path::new("/usr/share")),
            std::path::PathBuf::from("/usr/share/bank-files/models/ai")
        );
    }

    #[test]
    fn non_placeholder_model_dir_is_available() {
        let root = unique_test_dir("real-model");
        let dir = root.join(MODEL_ROOT);
        write_test_model_dir(&dir, false);

        let availability = model_dir_availability(LocalAiModelSource::Sidecar(dir.clone()))
            .expect("complete model dir should produce an availability value");
        assert_eq!(
            availability,
            LocalAiAvailability::Available {
                source: LocalAiModelSource::Sidecar(dir)
            }
        );
    }

    #[test]
    fn pattern_hint_matching_uses_normalized_labels() {
        let hints = vec![LocalAiPatternHint {
            label: "Café Market".to_string(),
            category: "Groceries".to_string(),
            budget_code: "FOOD".to_string(),
            reason: "local".to_string(),
        }];
        assert!(pattern_hint_for_label(&hints, "Cafe Market").is_some());
    }

    fn write_test_model_dir(dir: &Path, placeholder: bool) {
        fs::create_dir_all(dir).expect("model dir should be created");
        for file in MODEL_FILES {
            let contents = if file == MANIFEST_FILE {
                format!("{{\"placeholder\":{placeholder}}}")
            } else {
                "x".to_string()
            };
            fs::write(dir.join(file), contents).expect("model file should be written");
        }
    }

    fn transaction(
        date: &str,
        amount: &str,
        description: &str,
        counterparty: &str,
        tags: &str,
        account: &str,
        transaction_id: &str,
    ) -> Transaction {
        Transaction {
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("date should parse"),
            amount: amount.parse().expect("amount should parse"),
            description: description.to_string(),
            counterparty: counterparty.to_string(),
            tags: tags.to_string(),
            account: account.to_string(),
            transaction_id: transaction_id.to_string(),
            currency: "EUR".to_string(),
            source_file: "test.csv".to_string(),
            source_row: 1,
            category: String::new(),
            budget_code: String::new(),
            notes: String::new(),
            strict_key: String::new(),
            loose_key: String::new(),
        }
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        let unique = format!(
            "{}-{}-{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be available")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        fs::create_dir_all(&dir).expect("test dir should be created");
        dir
    }
}
