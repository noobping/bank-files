use super::index::{fallback_meta, SearchProviderIndex};
use super::launch::{handle_launch_search, join_search_terms, launch_search_query};
use super::SEARCH_PROVIDER_RESULT_LIMIT;
use crate::app_info::SEARCH_PROVIDER_OBJECT_PATH;

use adw::gio::{DBusConnection, DBusInterfaceInfo};
use adw::glib::variant::ToVariant;
use adw::glib::{self, Variant};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub(super) struct SearchProviderService {
    interface_info: DBusInterfaceInfo,
    index: RefCell<Option<SearchProviderIndex>>,
}

impl SearchProviderService {
    pub(super) fn new(interface_info: DBusInterfaceInfo) -> Self {
        Self {
            interface_info,
            index: RefCell::new(None),
        }
    }

    pub(super) fn register(self: Rc<Self>, connection: &DBusConnection) -> Result<(), glib::Error> {
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
