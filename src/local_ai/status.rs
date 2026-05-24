use super::{local_ai_availability, LocalAiAvailability, LocalAiModelSource};

pub fn local_ai_status_message(smart_insights_enabled: bool) -> String {
    match local_ai_availability(smart_insights_enabled) {
        LocalAiAvailability::Disabled => {
            "Local AI disabled: Smart Insights is off or local AI is not compiled in.".to_string()
        }
        LocalAiAvailability::Available { source } => {
            format!("Local AI available: using {}.", model_source(&source))
        }
        LocalAiAvailability::MissingModel(reason) => {
            format!("Local AI unavailable: {reason}. Deterministic Smart Insights are active.")
        }
        LocalAiAvailability::RuntimeError(reason) => {
            format!("Local AI failed: {reason}. Deterministic Smart Insights are active.")
        }
    }
}

fn model_source(source: &LocalAiModelSource) -> String {
    match source {
        LocalAiModelSource::Sidecar(path) => {
            format!("sidecar model bundle at {}", path.display())
        }
        #[cfg(target_os = "linux")]
        LocalAiModelSource::Installed(path) => {
            format!("installed model bundle at {}", path.display())
        }
        #[cfg(feature = "embedded-ai-model")]
        LocalAiModelSource::Extracted(path) => {
            format!("embedded model bundle extracted to {}", path.display())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn disabled_status_explains_local_ai_is_off() {
        assert!(local_ai_status_message(false).contains("Local AI disabled"));
    }

    #[test]
    fn sidecar_source_status_includes_path() {
        let source = LocalAiModelSource::Sidecar(PathBuf::from("/tmp/bank-files-ai"));

        assert_eq!(
            model_source(&source),
            "sidecar model bundle at /tmp/bank-files-ai"
        );
    }
}
