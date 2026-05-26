use once_cell::sync::Lazy;
use std::collections::HashMap;

static NL_TRANSLATIONS: Lazy<HashMap<String, String>> =
    Lazy::new(|| parse_po(include_str!("../po/nl.po")));
static DE_TRANSLATIONS: Lazy<HashMap<String, String>> =
    Lazy::new(|| parse_po(include_str!("../po/de.po")));
static ACTIVE_LANGUAGE: Lazy<Language> = Lazy::new(detect_language);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    English,
    Dutch,
    German,
}

pub fn init() {
    Lazy::force(&NL_TRANSLATIONS);
    Lazy::force(&DE_TRANSLATIONS);
    configure_process_locale(active_language());
}

pub fn active_language() -> Language {
    *ACTIVE_LANGUAGE
}

pub fn gettext(message: &str) -> String {
    if message.is_empty() {
        return String::new();
    }

    let translated = match active_language() {
        Language::English => None,
        Language::Dutch => NL_TRANSLATIONS.get(message),
        Language::German => DE_TRANSLATIONS.get(message),
    };

    if let Some(translated) = translated {
        return translated.clone();
    }

    message.to_string()
}

pub fn format(message: &str, replacements: &[(&str, String)]) -> String {
    let mut translated = gettext(message);
    for (key, value) in replacements {
        translated = translated.replace(&format!("{{{key}}}"), value);
    }
    translated
}

fn detect_language() -> Language {
    detect_environment_language()
        .or_else(detect_platform_language)
        .unwrap_or(Language::English)
}

fn detect_environment_language() -> Option<Language> {
    for name in ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"] {
        let Ok(value) = std::env::var(name) else {
            continue;
        };

        for locale in value.split(':') {
            if let Some(language) = language_from_locale(locale) {
                return Some(language);
            }
        }
    }

    None
}

fn language_from_locale(locale: &str) -> Option<Language> {
    let normalized = locale.trim().to_ascii_lowercase();
    if normalized.starts_with("nl") {
        return Some(Language::Dutch);
    }
    if normalized.starts_with("de") {
        return Some(Language::German);
    }
    if normalized.starts_with("en") || normalized == "c" || normalized == "posix" {
        return Some(Language::English);
    }

    None
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    #[link_name = "GetUserDefaultUILanguage"]
    fn get_user_default_ui_language() -> u16;
}

#[cfg(target_os = "windows")]
fn detect_platform_language() -> Option<Language> {
    language_from_windows_lang_id(unsafe { get_user_default_ui_language() })
}

#[cfg(not(target_os = "windows"))]
fn detect_platform_language() -> Option<Language> {
    None
}

#[cfg(any(target_os = "windows", test))]
fn language_from_windows_lang_id(lang_id: u16) -> Option<Language> {
    match lang_id & 0x03ff {
        0x07 => Some(Language::German),
        0x09 => Some(Language::English),
        0x13 => Some(Language::Dutch),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn configure_process_locale(language: Language) {
    let (language_code, locale) = match language {
        Language::English => ("en", "en_US.UTF-8"),
        Language::Dutch => ("nl", "nl_NL.UTF-8"),
        Language::German => ("de", "de_DE.UTF-8"),
    };

    set_env_if_missing("LANGUAGE", language_code);
    set_env_if_missing("LC_MESSAGES", locale);
    set_env_if_missing("LANG", locale);
}

#[cfg(not(target_os = "windows"))]
fn configure_process_locale(_language: Language) {}

#[cfg(target_os = "windows")]
fn set_env_if_missing(name: &str, value: &str) {
    if std::env::var_os(name).is_none() {
        std::env::set_var(name, value);
    }
}

#[derive(Clone, Copy)]
enum ActiveField {
    Id,
    Str,
}

fn parse_po(source: &str) -> HashMap<String, String> {
    let mut messages = HashMap::new();
    let mut msgid = String::new();
    let mut msgstr = String::new();
    let mut active = None;

    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            insert_message(&mut messages, &mut msgid, &mut msgstr);
            active = None;
            continue;
        }

        if line.starts_with('#') {
            continue;
        }

        if let Some(value) = line.strip_prefix("msgid ") {
            insert_message(&mut messages, &mut msgid, &mut msgstr);
            msgid = parse_po_string(value);
            msgstr.clear();
            active = Some(ActiveField::Id);
            continue;
        }

        if let Some(value) = line.strip_prefix("msgstr ") {
            msgstr = parse_po_string(value);
            active = Some(ActiveField::Str);
            continue;
        }

        if line.starts_with('"') {
            let value = parse_po_string(line);
            match active {
                Some(ActiveField::Id) => msgid.push_str(&value),
                Some(ActiveField::Str) => msgstr.push_str(&value),
                None => {}
            }
        }
    }

    insert_message(&mut messages, &mut msgid, &mut msgstr);
    messages
}

fn insert_message(messages: &mut HashMap<String, String>, msgid: &mut String, msgstr: &mut String) {
    if !msgid.is_empty() && !msgstr.is_empty() {
        messages.insert(std::mem::take(msgid), std::mem::take(msgstr));
    } else {
        msgid.clear();
        msgstr.clear();
    }
}

fn parse_po_string(value: &str) -> String {
    let value = value.trim();
    let value = value.strip_prefix('"').unwrap_or(value);
    let value = value.strip_suffix('"').unwrap_or(value);

    let mut output = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            output.push(ch);
            continue;
        }

        match chars.next() {
            Some('n') => output.push('\n'),
            Some('r') => output.push('\r'),
            Some('t') => output.push('\t'),
            Some('"') => output.push('"'),
            Some('\\') => output.push('\\'),
            Some(other) => output.push(other),
            None => output.push('\\'),
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_prefixes_detect_supported_languages() {
        assert_eq!(language_from_locale("nl_NL.UTF-8"), Some(Language::Dutch));
        assert_eq!(language_from_locale("de_DE.UTF-8"), Some(Language::German));
        assert_eq!(language_from_locale("en_US.UTF-8"), Some(Language::English));
        assert_eq!(language_from_locale("C"), Some(Language::English));
        assert_eq!(language_from_locale("fr_FR.UTF-8"), None);
    }

    #[test]
    fn windows_primary_language_id_detects_supported_languages() {
        assert_eq!(language_from_windows_lang_id(0x0413), Some(Language::Dutch));
        assert_eq!(
            language_from_windows_lang_id(0x0407),
            Some(Language::German)
        );
        assert_eq!(
            language_from_windows_lang_id(0x0409),
            Some(Language::English)
        );
        assert_eq!(language_from_windows_lang_id(0x040c), None);
    }
}
