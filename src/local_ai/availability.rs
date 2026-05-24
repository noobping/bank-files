use super::{LocalAiAvailability, LocalAiModelSource};
#[cfg(feature = "embedded-ai-model")]
use crate::util::app_cache_dir;
use anyhow::{Context, Result};
use serde::Deserialize;
#[cfg(feature = "embedded-ai-model")]
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) const MODEL_ROOT: &str = "models/ai";
#[cfg(target_os = "linux")]
const INSTALLED_MODEL_ROOT: &str = "bank-files/models/ai";
pub(super) const MANIFEST_FILE: &str = "model.json";
pub(super) const MODEL_FILES: [&str; 5] = [
    "classifier.onnx",
    "generator.safetensors",
    "tokenizer.json",
    "model.json",
    "LICENSE",
];

pub(super) fn local_ai_availability(smart_insights_enabled: bool) -> LocalAiAvailability {
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

pub(super) fn local_ai_model_sources_availability(
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
pub(super) fn installed_model_dir_from_data_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(INSTALLED_MODEL_ROOT)
}

pub(super) fn model_dir_availability(source: LocalAiModelSource) -> Option<LocalAiAvailability> {
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
