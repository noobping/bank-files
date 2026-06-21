use adw::glib::{self, Variant};

use std::ffi::OsString;
use std::process::Command;

pub(super) fn handle_launch_search(parameters: &Variant) -> Result<Option<Variant>, glib::Error> {
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

pub(super) fn join_search_terms(terms: &[String]) -> String {
    terms
        .iter()
        .map(String::as_str)
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn launch_search_query(query: &str) -> Result<(), String> {
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
