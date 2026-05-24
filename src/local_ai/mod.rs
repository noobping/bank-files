mod availability;
#[cfg(feature = "local-ai")]
mod categories;
mod configuration;
mod input;
mod runtime;
#[cfg(feature = "smart-insights")]
mod status;

use availability::local_ai_availability;
use configuration::generated_configuration_from_draft;
use runtime::{runtime_generate_configuration_draft, runtime_pattern_hints};
#[cfg(feature = "smart-insights")]
pub use status::local_ai_status_message;

use crate::data::{
    EditableAlias, EditableBudget, EditableRule, GeneratedConfiguration, IgnoredTransactionPattern,
};
use crate::model::AppData;
use crate::util::normalize_key;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LocalAiAvailability {
    Disabled,
    Available { source: LocalAiModelSource },
    MissingModel(String),
    RuntimeError(String),
}

impl LocalAiAvailability {
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
        let LocalAiAvailability::Available { source } = &self.availability else {
            return Ok(None);
        };
        runtime_generate_configuration_draft(input, Some(source))
    }

    pub fn pattern_hints(
        &self,
        input: &LocalAiInput,
    ) -> std::result::Result<Vec<LocalAiPatternHint>, LocalAiError> {
        let LocalAiAvailability::Available { source } = &self.availability else {
            return Ok(Vec::new());
        };
        runtime_pattern_hints(input, Some(source))
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

fn system_locale() -> String {
    std::env::var("LANG")
        .ok()
        .and_then(|locale| locale.split('.').next().map(str::to_string))
        .filter(|locale| !locale.trim().is_empty())
        .unwrap_or_else(|| "en".to_string())
}

#[cfg(test)]
mod tests;
