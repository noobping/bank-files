use super::*;
use std::path::{Path, PathBuf};

pub(super) fn should_copy_to_app_storage(ui: &UiHandles) -> bool {
    !ui.remember_mode.get().opens_live_files() && ui.storage_capabilities.borrow().data_writable
}

pub(super) fn live_sources_from_paths(files: Vec<PathBuf>) -> (Vec<TransactionSource>, usize) {
    let mut skipped = 0;
    let mut sources = Vec::new();
    for file in files {
        if file.is_file() && path_is_csv(&file) {
            sources.push(TransactionSource::live_file(file));
        } else {
            skipped += 1;
        }
    }
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    sources.dedup_by(|left, right| left.path == right.path);
    (sources, skipped)
}

pub(super) fn local_paths_from_uris(uris: &[String]) -> (Vec<PathBuf>, usize) {
    let mut unresolved = 0;
    let mut paths = Vec::new();
    for uri in uris {
        let file = gtk::gio::File::for_uri(uri);
        if let Some(path) = file.path() {
            paths.push(path);
        } else {
            unresolved += 1;
        }
    }
    (paths, unresolved)
}

fn path_is_csv(path: &Path) -> bool {
    path.extension()
        .map(|extension| extension.eq_ignore_ascii_case("csv"))
        .unwrap_or(false)
}

pub(super) fn live_source_set(
    data: &AppData,
    remember_mode: RememberMode,
    mut new_sources: Vec<TransactionSource>,
) -> Vec<TransactionSource> {
    if remember_mode.opens_live_files() {
        return new_sources;
    }

    let mut sources = data.transaction_sources.clone();
    sources.append(&mut new_sources);
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    sources.dedup_by(|left, right| left.path == right.path && left.kind == right.kind);
    sources
}

pub(in crate::app) fn current_sources_for_reload(
    data: &AppData,
    remember_mode: RememberMode,
) -> Vec<TransactionSource> {
    if remember_mode.opens_live_files()
        || data
            .transaction_sources
            .iter()
            .any(TransactionSource::is_live)
    {
        data.transaction_sources.clone()
    } else {
        Vec::new()
    }
}
