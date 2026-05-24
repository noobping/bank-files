use super::*;

pub(in crate::app) fn tr(message: &str) -> String {
    gettext(message)
}

pub(in crate::app) fn trf(message: &str, replacements: &[(&str, String)]) -> String {
    crate::i18n::format(message, replacements)
}
