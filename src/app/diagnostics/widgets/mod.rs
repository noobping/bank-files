use super::*;

mod fields;
mod file_actions;
mod file_card;
mod helpers;
mod patterns;

pub(in crate::app) use file_card::diagnostic_file_card;
pub(in crate::app) use helpers::{delimiter_label, diagnostic_error_text, empty_page};
pub(in crate::app) use patterns::{transaction_pattern_card, transaction_pattern_matches};

const TRANSACTION_PATTERN_VALUE_PREVIEW_LIMIT: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum DetectedFieldsVisibility {
    FollowShowAll,
    Collapsed,
    Expanded,
}

impl DetectedFieldsVisibility {
    fn reveal_initially(self, show_all: bool) -> bool {
        match self {
            Self::FollowShowAll => show_all,
            Self::Collapsed => false,
            Self::Expanded => true,
        }
    }
}

#[cfg(test)]
mod tests;
