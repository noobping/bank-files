use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DedupeMode {
    #[default]
    Enabled,
    Disabled,
}

impl DedupeMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Enabled => "on",
            Self::Disabled => "off",
        }
    }

    pub fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }

    pub fn from_enabled(enabled: bool) -> Self {
        if enabled {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Enabled => "Exact duplicates are skipped during import.",
            Self::Disabled => "All imported rows remain visible, including possible duplicates.",
        }
    }
}
