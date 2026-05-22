pub const APP_ID: &str = "io.github.noobping.BankFiles";
pub const APP_NAME: &str = "Bank Files";
#[cfg(target_os = "linux")]
pub const SEARCH_PROVIDER_BUS_NAME: &str = "io.github.noobping.BankFiles.SearchProvider";
#[cfg(target_os = "linux")]
pub const SEARCH_PROVIDER_OBJECT_PATH: &str = "/io/github/noobping/BankFiles/SearchProvider";
pub const RESOURCE_ID: &str = "/io/github/noobping/BankFiles";

pub fn display_name() -> String {
    crate::i18n::gettext(APP_NAME)
}

pub fn summary() -> String {
    crate::i18n::gettext(env!("CARGO_PKG_DESCRIPTION"))
}
