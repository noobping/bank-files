mod data;
mod index;
mod launch;
mod service;

use crate::app_info::SEARCH_PROVIDER_BUS_NAME;

use adw::gio::{self, BusNameOwnerFlags, BusType, DBusNodeInfo};
use adw::glib::MainLoop;

use service::SearchProviderService;
use std::ffi::OsString;
use std::rc::Rc;

const SEARCH_PROVIDER_INTERFACE: &str = "org.gnome.Shell.SearchProvider2";
const SEARCH_PROVIDER_RESULT_LIMIT: usize = 24;
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

#[cfg(test)]
mod tests;
