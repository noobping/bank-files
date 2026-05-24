use crate::app::transaction_search_text;
use crate::app_info::{APP_ID, SEARCH_PROVIDER_BUS_NAME, SEARCH_PROVIDER_OBJECT_PATH};
use crate::model::{AppData, DedupeMode, Transaction, TransactionLoadScope};
use crate::util::signed_money;

#[cfg(feature = "smart-insights")]
use adw::gio::prelude::SettingsExt;
use adw::gio::{self, BusNameOwnerFlags, BusType, DBusConnection, DBusInterfaceInfo, DBusNodeInfo};
use adw::glib::variant::ToVariant;
use adw::glib::{self, MainLoop, Variant};
use sha2::{Digest, Sha256};

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsString;
use std::process::Command;
use std::rc::Rc;

const SEARCH_PROVIDER_INTERFACE: &str = "org.gnome.Shell.SearchProvider2";
const SEARCH_PROVIDER_RESULT_LIMIT: usize = 24;
const RESULT_ID_SEPARATOR: char = '\u{1f}';
const SEARCH_PROVIDER_XML: &str = r#"
<node>
  <interface name="org.gnome.Shell.SearchProvider2">
    <method name="GetInitialResultSet">
      <arg type="as" name="terms" direction="in" />
      <arg type="as" name="results" direction="out" />
    </method>
    <method name="GetSubsearchResultSet">
      <arg type="as" name="previous_results" direction="in" />
      <arg type="as" name="terms" direction="in" />
      <arg type="as" name="results" direction="out" />
    </method>
    <method name="GetResultMetas">
      <arg type="as" name="identifiers" direction="in" />
      <arg type="aa{sv}" name="metas" direction="out" />
    </method>
    <method name="ActivateResult">
      <arg type="s" name="identifier" direction="in" />
      <arg type="as" name="terms" direction="in" />
      <arg type="u" name="timestamp" direction="in" />
    </method>
    <method name="LaunchSearch">
      <arg type="as" name="terms" direction="in" />
      <arg type="u" name="timestamp" direction="in" />
    </method>
  </interface>
</node>
"#;

pub(crate) fn is_search_provider_command(args: &[OsString]) -> bool {
    args.get(1).is_some_and(|arg| arg == "--search-provider")
}

pub(crate) fn run() -> i32 {
    let node_info = match DBusNodeInfo::for_xml(SEARCH_PROVIDER_XML) {
        Ok(node_info) => node_info,
        Err(err) => {
            eprintln!("Failed to parse search provider D-Bus XML: {err}");
            return 1;
        }
    };
    let Some(interface_info) = node_info.lookup_interface(SEARCH_PROVIDER_INTERFACE) else {
        eprintln!("Search provider interface metadata is missing.");
        return 1;
    };

    let main_loop = MainLoop::new(None, false);
    let service = Rc::new(SearchProviderService::new(interface_info));
    let service_for_bus = Rc::clone(&service);
    let loop_for_failure = main_loop.clone();
    let owner_id = gio::bus_own_name(
        BusType::Session,
        SEARCH_PROVIDER_BUS_NAME,
        BusNameOwnerFlags::NONE,
        move |connection, _name| {
            if let Err(err) = Rc::clone(&service_for_bus).register(&connection) {
                eprintln!("Failed to export search provider object: {err}");
                loop_for_failure.quit();
            }
        },
        |_connection, _name| {},
        {
            let main_loop = main_loop.clone();
            move |_connection, _name| {
                main_loop.quit();
            }
        },
    );

    main_loop.run();
    gio::bus_unown_name(owner_id);
    0
}

struct SearchProviderService {
    interface_info: DBusInterfaceInfo,
    index: RefCell<Option<SearchProviderIndex>>,
}

impl SearchProviderService {
    fn new(interface_info: DBusInterfaceInfo) -> Self {
        Self {
            interface_info,
            index: RefCell::new(None),
        }
    }

    fn register(self: Rc<Self>, connection: &DBusConnection) -> Result<(), glib::Error> {
        let service = Rc::clone(&self);
        let _registration_id = connection
            .register_object(SEARCH_PROVIDER_OBJECT_PATH, &self.interface_info)
            .method_call(
                move |_connection,
                      _sender,
                      _object_path,
                      _interface_name,
                      method_name,
                      parameters,
                      invocation| {
                    invocation.return_result(service.handle_method_call(method_name, &parameters));
                },
            )
            .build()?;

        Ok(())
    }

    fn handle_method_call(
        &self,
        method_name: &str,
        parameters: &Variant,
    ) -> Result<Option<Variant>, glib::Error> {
        match method_name {
            "GetInitialResultSet" => self.handle_get_initial_result_set(parameters),
            "GetSubsearchResultSet" => self.handle_get_subsearch_result_set(parameters),
            "GetResultMetas" => self.handle_get_result_metas(parameters),
            "ActivateResult" => self.handle_activate_result(parameters),
            "LaunchSearch" => handle_launch_search(parameters),
            _ => {
                eprintln!("Unknown search provider method: {method_name}.");
                Ok(None)
            }
        }
    }

    fn handle_get_initial_result_set(
        &self,
        parameters: &Variant,
    ) -> Result<Option<Variant>, glib::Error> {
        let Some((terms,)) = parameters.get::<(Vec<String>,)>() else {
            eprintln!("Search provider GetInitialResultSet received invalid parameters.");
            return Ok(Some((Vec::<String>::new(),).to_variant()));
        };

        Ok(Some((self.search_result_ids(&terms),).to_variant()))
    }

    fn handle_get_subsearch_result_set(
        &self,
        parameters: &Variant,
    ) -> Result<Option<Variant>, glib::Error> {
        let Some((_previous_results, terms)) = parameters.get::<(Vec<String>, Vec<String>)>()
        else {
            eprintln!("Search provider GetSubsearchResultSet received invalid parameters.");
            return Ok(Some((Vec::<String>::new(),).to_variant()));
        };

        Ok(Some((self.search_result_ids(&terms),).to_variant()))
    }

    fn handle_get_result_metas(
        &self,
        parameters: &Variant,
    ) -> Result<Option<Variant>, glib::Error> {
        let Some((identifiers,)) = parameters.get::<(Vec<String>,)>() else {
            eprintln!("Search provider GetResultMetas received invalid parameters.");
            return Ok(Some((Vec::<HashMap<String, Variant>>::new(),).to_variant()));
        };

        let metas = self.with_index(|index| {
            identifiers
                .into_iter()
                .map(|identifier| {
                    index
                        .meta_for_identifier(&identifier)
                        .unwrap_or_else(|| fallback_meta(&identifier))
                })
                .collect::<Vec<_>>()
        });
        Ok(Some((metas,).to_variant()))
    }

    fn handle_activate_result(&self, parameters: &Variant) -> Result<Option<Variant>, glib::Error> {
        let Some((identifier, terms, _timestamp)) = parameters.get::<(String, Vec<String>, u32)>()
        else {
            eprintln!("Search provider ActivateResult received invalid parameters.");
            return Ok(None);
        };

        let query = self
            .with_index(|index| index.activation_query_for_identifier(&identifier))
            .unwrap_or_else(|| join_search_terms(&terms));
        if let Err(err) = launch_search_query(&query) {
            eprintln!("Failed to launch transaction search result: {err}");
        }
        Ok(None)
    }

    fn search_result_ids(&self, terms: &[String]) -> Vec<String> {
        self.with_index(|index| index.search_result_ids(terms, SEARCH_PROVIDER_RESULT_LIMIT))
    }

    fn with_index<T>(&self, f: impl FnOnce(&SearchProviderIndex) -> T) -> T {
        if self.index.borrow().is_none() {
            *self.index.borrow_mut() = Some(SearchProviderIndex::load());
        }
        let index = self.index.borrow();
        if let Some(index) = index.as_ref() {
            return f(index);
        }

        let fallback = SearchProviderIndex::default();
        f(&fallback)
    }
}

fn handle_launch_search(parameters: &Variant) -> Result<Option<Variant>, glib::Error> {
    let Some((terms, _timestamp)) = parameters.get::<(Vec<String>, u32)>() else {
        eprintln!("Search provider LaunchSearch received invalid parameters.");
        return Ok(None);
    };

    let query = join_search_terms(&terms);
    if let Err(err) = launch_search_query(&query) {
        eprintln!("Failed to launch transaction search: {err}");
    }
    Ok(None)
}

fn load_search_data() -> AppData {
    crate::data::load_app_data_read_only_aware(
        DedupeMode::Enabled,
        false,
        TransactionLoadScope::All,
        search_smart_insights_enabled(),
    )
    .map(|(data, _capabilities)| data)
    .unwrap_or_else(|err| {
        eprintln!("Failed to load transactions for GNOME search: {err}");
        AppData::default()
    })
}

#[cfg(feature = "smart-insights")]
fn search_smart_insights_enabled() -> bool {
    gio::SettingsSchemaSource::default()
        .and_then(|source| source.lookup(APP_ID, true))
        .map(|_| gio::Settings::new(APP_ID).boolean("show-predictions"))
        .unwrap_or(false)
}

#[cfg(not(feature = "smart-insights"))]
fn search_smart_insights_enabled() -> bool {
    false
}

fn fallback_meta(identifier: &str) -> HashMap<String, Variant> {
    let mut meta = HashMap::new();
    meta.insert("id".to_string(), identifier.to_variant());
    meta.insert("name".to_string(), identifier.to_variant());
    meta.insert("gicon".to_string(), APP_ID.to_variant());
    meta
}

fn transaction_title(tx: &Transaction) -> String {
    first_present([
        tx.counterparty.as_str(),
        tx.description.as_str(),
        tx.category.as_str(),
        tx.budget_code.as_str(),
    ])
    .unwrap_or("Transaction")
    .to_string()
}

fn transaction_description(tx: &Transaction) -> String {
    [
        tx.date.to_string(),
        signed_money(tx.amount),
        tx.category.trim().to_string(),
        tx.description.trim().to_string(),
    ]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" · ")
}

fn first_present<'a>(values: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    values
        .into_iter()
        .map(str::trim)
        .find(|value| !value.is_empty())
}

fn join_search_terms(terms: &[String]) -> String {
    terms
        .iter()
        .map(String::as_str)
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn launch_search_query(query: &str) -> Result<(), String> {
    if query.is_empty() {
        launch_app(&[])
    } else {
        launch_app(
            [
                OsString::from("--transaction-search"),
                OsString::from(query),
            ]
            .as_slice(),
        )
    }
}

fn launch_app(args: &[OsString]) -> Result<(), String> {
    let executable = std::env::current_exe()
        .map_err(|err| format!("Failed to resolve current executable path: {err}"))?;
    Command::new(executable)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|err| format!("Failed to spawn app: {err}"))
}

fn encode_result_id(tx: &Transaction) -> String {
    let mut digest = Sha256::new();
    digest.update(tx.source_file.as_bytes());
    digest.update([RESULT_ID_SEPARATOR as u8]);
    digest.update(tx.source_row.to_string().as_bytes());
    digest.update([RESULT_ID_SEPARATOR as u8]);
    digest.update(tx.strict_key.as_bytes());
    digest.update([RESULT_ID_SEPARATOR as u8]);
    digest.update(tx.date.to_string().as_bytes());
    digest.update([RESULT_ID_SEPARATOR as u8]);
    digest.update(tx.amount.to_string().as_bytes());
    digest
        .finalize()
        .into_iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn result_id_is_valid(identifier: &str) -> bool {
    identifier.len() == 64 && identifier.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn transaction_activation_query(tx: &Transaction) -> String {
    first_present([tx.transaction_id.as_str(), tx.strict_key.as_str()])
        .map(str::to_string)
        .unwrap_or_else(|| {
            [
                tx.source_file.trim().to_string(),
                tx.source_row.to_string(),
                tx.date.to_string(),
            ]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
        })
}

#[derive(Default)]
struct SearchProviderIndex {
    entries: Vec<SearchProviderEntry>,
}

impl SearchProviderIndex {
    fn load() -> Self {
        Self::from_data(&load_search_data())
    }

    fn from_data(data: &AppData) -> Self {
        Self {
            entries: data
                .transactions
                .iter()
                .map(SearchProviderEntry::from_transaction)
                .collect(),
        }
    }

    fn search_result_ids(&self, terms: &[String], limit: usize) -> Vec<String> {
        let terms = normalized_search_terms(terms);
        if terms.is_empty() {
            return Vec::new();
        }

        self.entries
            .iter()
            .filter(|entry| entry.matches_terms(&terms))
            .take(limit)
            .map(|entry| entry.identifier.clone())
            .collect()
    }

    fn entry_for_identifier(&self, identifier: &str) -> Option<&SearchProviderEntry> {
        if !result_id_is_valid(identifier) {
            return None;
        }

        self.entries
            .iter()
            .find(|entry| entry.identifier == identifier)
    }

    fn meta_for_identifier(&self, identifier: &str) -> Option<HashMap<String, Variant>> {
        let entry = self.entry_for_identifier(identifier)?;
        let mut meta = HashMap::new();
        meta.insert("id".to_string(), entry.identifier.to_variant());
        meta.insert("name".to_string(), entry.title.to_variant());
        meta.insert("description".to_string(), entry.description.to_variant());
        meta.insert("gicon".to_string(), APP_ID.to_variant());
        Some(meta)
    }

    fn activation_query_for_identifier(&self, identifier: &str) -> Option<String> {
        self.entry_for_identifier(identifier)
            .map(|entry| entry.activation_query.clone())
    }
}

struct SearchProviderEntry {
    identifier: String,
    search_text: String,
    title: String,
    description: String,
    activation_query: String,
}

impl SearchProviderEntry {
    fn from_transaction(tx: &Transaction) -> Self {
        Self {
            identifier: encode_result_id(tx),
            search_text: transaction_search_text(tx).to_lowercase(),
            title: transaction_title(tx),
            description: transaction_description(tx),
            activation_query: transaction_activation_query(tx),
        }
    }

    fn matches_terms(&self, terms: &[String]) -> bool {
        terms.iter().all(|term| self.search_text.contains(term))
    }
}

fn normalized_search_terms(terms: &[String]) -> Vec<String> {
    terms
        .iter()
        .map(|term| term.trim().to_lowercase())
        .filter(|term| !term.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        encode_result_id, join_search_terms, normalized_search_terms, result_id_is_valid,
        SearchProviderEntry, SearchProviderIndex,
    };
    use crate::model::{AppData, Transaction};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn transaction(description: &str, counterparty: &str, amount: &str) -> Transaction {
        Transaction {
            date: chrono::NaiveDate::from_ymd_opt(2026, 5, 21).expect("valid date"),
            amount: Decimal::from_str(amount).expect("valid decimal"),
            description: description.to_string(),
            counterparty: counterparty.to_string(),
            tags: String::new(),
            account: String::new(),
            transaction_id: String::new(),
            currency: "EUR".to_string(),
            source_file: "bank.csv".to_string(),
            source_row: 42,
            category: String::new(),
            budget_code: String::new(),
            notes: String::new(),
            strict_key: format!("{description}:{counterparty}:{amount}"),
            loose_key: String::new(),
        }
    }

    #[test]
    fn result_ids_are_opaque_hashes() {
        let tx = transaction("Groceries", "Shop", "-12.34");
        let identifier = encode_result_id(&tx);

        assert_eq!(identifier.len(), 64);
        assert!(result_id_is_valid(&identifier));
        assert!(!identifier.contains("bank.csv"));
        assert!(!identifier.contains("Groceries"));
    }

    #[test]
    fn invalid_result_ids_are_rejected() {
        assert!(!result_id_is_valid(""));
        assert!(!result_id_is_valid("bank.csv"));
        assert!(!result_id_is_valid("xyz"));
    }

    #[test]
    fn search_terms_join_with_spaces() {
        assert_eq!(
            join_search_terms(&["rent".to_string(), "".to_string(), "may".to_string(),]),
            "rent may".to_string()
        );
    }

    #[test]
    fn search_matches_all_normalized_terms() {
        let tx = transaction("Monthly rent", "Housing Company", "-950");
        let entry = SearchProviderEntry::from_transaction(&tx);

        assert!(entry.matches_terms(&["rent".to_string(), "housing".to_string()]));
        assert!(!entry.matches_terms(&["salary".to_string()]));
    }

    #[test]
    fn search_result_order_follows_transaction_order_and_limit() {
        let data = AppData {
            transactions: vec![
                transaction("Coffee", "Cafe", "-3.50"),
                transaction("Coffee beans", "Roaster", "-12.00"),
            ],
            ..AppData::default()
        };

        let index = SearchProviderIndex::from_data(&data);
        let matches = index.search_result_ids(&["coffee".to_string()], 1);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], encode_result_id(&data.transactions[0]));
    }

    #[test]
    fn index_returns_metas_and_activation_queries_by_identifier() {
        let mut tx = transaction("Coffee", "Cafe", "-3.50");
        tx.transaction_id = "tx-123".to_string();
        let identifier = encode_result_id(&tx);
        let data = AppData {
            transactions: vec![tx],
            ..AppData::default()
        };
        let index = SearchProviderIndex::from_data(&data);
        let meta = index
            .meta_for_identifier(&identifier)
            .expect("known identifier should produce metadata");

        assert_eq!(
            index.activation_query_for_identifier(&identifier),
            Some("tx-123".to_string())
        );
        assert!(meta.contains_key("name"));
        assert_eq!(
            index.activation_query_for_identifier("not-a-result-id"),
            None
        );
    }

    #[test]
    fn normalized_search_terms_drop_empty_values() {
        assert_eq!(
            normalized_search_terms(&[" Rent ".to_string(), "".to_string(), "MAY".to_string()]),
            vec!["rent".to_string(), "may".to_string()]
        );
    }
}
