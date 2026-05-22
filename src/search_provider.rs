use crate::app::transaction_search_text;
use crate::app_info::{APP_ID, SEARCH_PROVIDER_BUS_NAME, SEARCH_PROVIDER_OBJECT_PATH};
use crate::model::{AppData, DedupeMode, Transaction, TransactionLoadScope};
use crate::util::signed_money;

use adw::gio::{self, BusNameOwnerFlags, BusType, DBusConnection, DBusInterfaceInfo, DBusNodeInfo};
use adw::glib::variant::ToVariant;
use adw::glib::{self, MainLoop, Variant};
use sha2::{Digest, Sha256};

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
    let service_for_bus = service.clone();
    let loop_for_failure = main_loop.clone();
    let owner_id = gio::bus_own_name(
        BusType::Session,
        SEARCH_PROVIDER_BUS_NAME,
        BusNameOwnerFlags::NONE,
        move |connection, _name| {
            if let Err(err) = service_for_bus.register(&connection) {
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
}

impl SearchProviderService {
    fn new(interface_info: DBusInterfaceInfo) -> Self {
        Self { interface_info }
    }

    fn register(&self, connection: &DBusConnection) -> Result<(), glib::Error> {
        let _registration_id = connection
            .register_object(SEARCH_PROVIDER_OBJECT_PATH, &self.interface_info)
            .method_call(
                |_connection,
                 _sender,
                 _object_path,
                 _interface_name,
                 method_name,
                 parameters,
                 invocation| match method_name {
                    "GetInitialResultSet" => {
                        invocation.return_result(handle_get_initial_result_set(&parameters));
                    }
                    "GetSubsearchResultSet" => {
                        invocation.return_result(handle_get_subsearch_result_set(&parameters));
                    }
                    "GetResultMetas" => {
                        invocation.return_result(handle_get_result_metas(&parameters));
                    }
                    "ActivateResult" => {
                        invocation.return_result(handle_activate_result(&parameters));
                    }
                    "LaunchSearch" => {
                        invocation.return_result(handle_launch_search(&parameters));
                    }
                    _ => {
                        eprintln!("Unknown search provider method: {method_name}.");
                        invocation.return_result(Ok(None));
                    }
                },
            )
            .build()?;

        Ok(())
    }
}

fn handle_get_initial_result_set(parameters: &Variant) -> Result<Option<Variant>, glib::Error> {
    let Some((terms,)) = parameters.get::<(Vec<String>,)>() else {
        eprintln!("Search provider GetInitialResultSet received invalid parameters.");
        return Ok(Some((Vec::<String>::new(),).to_variant()));
    };

    Ok(Some((search_result_ids(&terms),).to_variant()))
}

fn handle_get_subsearch_result_set(parameters: &Variant) -> Result<Option<Variant>, glib::Error> {
    let Some((_previous_results, terms)) = parameters.get::<(Vec<String>, Vec<String>)>() else {
        eprintln!("Search provider GetSubsearchResultSet received invalid parameters.");
        return Ok(Some((Vec::<String>::new(),).to_variant()));
    };

    Ok(Some((search_result_ids(&terms),).to_variant()))
}

fn handle_get_result_metas(parameters: &Variant) -> Result<Option<Variant>, glib::Error> {
    let Some((identifiers,)) = parameters.get::<(Vec<String>,)>() else {
        eprintln!("Search provider GetResultMetas received invalid parameters.");
        return Ok(Some((Vec::<HashMap<String, Variant>>::new(),).to_variant()));
    };

    let data = load_search_data();
    let metas = identifiers
        .into_iter()
        .map(|identifier| {
            meta_for_identifier(&identifier, &data).unwrap_or_else(|| fallback_meta(&identifier))
        })
        .collect::<Vec<_>>();
    Ok(Some((metas,).to_variant()))
}

fn handle_activate_result(parameters: &Variant) -> Result<Option<Variant>, glib::Error> {
    let Some((identifier, terms, _timestamp)) = parameters.get::<(String, Vec<String>, u32)>()
    else {
        eprintln!("Search provider ActivateResult received invalid parameters.");
        return Ok(None);
    };

    let data = load_search_data();
    let query = transaction_for_identifier(&identifier, &data)
        .map(transaction_activation_query)
        .unwrap_or_else(|| join_search_terms(&terms));
    if let Err(err) = launch_search_query(&query) {
        eprintln!("Failed to launch transaction search result: {err}");
    }
    Ok(None)
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
    )
    .map(|(data, _capabilities)| data)
    .unwrap_or_else(|err| {
        eprintln!("Failed to load transactions for GNOME search: {err}");
        AppData::default()
    })
}

fn meta_for_identifier(identifier: &str, data: &AppData) -> Option<HashMap<String, Variant>> {
    let tx = transaction_for_identifier(identifier, data)?;
    let mut meta = HashMap::new();
    meta.insert("id".to_string(), identifier.to_variant());
    meta.insert("name".to_string(), transaction_title(tx).to_variant());
    meta.insert(
        "description".to_string(),
        transaction_description(tx).to_variant(),
    );
    meta.insert("gicon".to_string(), APP_ID.to_variant());
    Some(meta)
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

fn transaction_for_identifier<'a>(identifier: &str, data: &'a AppData) -> Option<&'a Transaction> {
    if !result_id_is_valid(identifier) {
        return None;
    }

    data.transactions
        .iter()
        .find(|tx| encode_result_id(tx) == identifier)
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

fn search_provider_transactions<'a>(
    data: &'a AppData,
    terms: &[String],
    limit: usize,
) -> Vec<&'a Transaction> {
    let terms = normalized_search_terms(terms);
    if terms.is_empty() {
        return Vec::new();
    }

    data.transactions
        .iter()
        .filter(|tx| transaction_matches_terms(tx, &terms))
        .take(limit)
        .collect()
}

fn normalized_search_terms(terms: &[String]) -> Vec<String> {
    terms
        .iter()
        .map(|term| term.trim().to_lowercase())
        .filter(|term| !term.is_empty())
        .collect()
}

fn transaction_matches_terms(tx: &Transaction, terms: &[String]) -> bool {
    let haystack = transaction_search_text(tx).to_lowercase();
    terms.iter().all(|term| haystack.contains(term))
}

fn search_result_ids(terms: &[String]) -> Vec<String> {
    let data = load_search_data();
    search_provider_transactions(&data, terms, SEARCH_PROVIDER_RESULT_LIMIT)
        .into_iter()
        .map(encode_result_id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        encode_result_id, join_search_terms, normalized_search_terms, result_id_is_valid,
        search_provider_transactions, transaction_matches_terms,
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

        assert!(transaction_matches_terms(
            &tx,
            &["rent".to_string(), "housing".to_string()]
        ));
        assert!(!transaction_matches_terms(&tx, &["salary".to_string()]));
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

        let matches = search_provider_transactions(&data, &["coffee".to_string()], 1);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].description, "Coffee");
    }

    #[test]
    fn normalized_search_terms_drop_empty_values() {
        assert_eq!(
            normalized_search_terms(&[" Rent ".to_string(), "".to_string(), "MAY".to_string()]),
            vec!["rent".to_string(), "may".to_string()]
        );
    }
}
