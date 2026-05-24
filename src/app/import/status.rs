use super::*;

pub(super) fn import_status(result: data::CsvCopyResult) -> String {
    let mut message = match (result.transaction_csvs, result.config_csvs) {
        (transactions, configs) if transactions > 0 && configs > 0 => trf(
            "{transactions} transaction CSV file(s) and {configs} configuration CSV file(s) were opened and applied.",
            &[
                ("transactions", transactions.to_string()),
                ("configs", configs.to_string()),
            ],
        ),
        (transactions, _) if transactions > 0 => trf(
            "{count} transaction CSV file(s) were opened and remembered.",
            &[("count", transactions.to_string())],
        ),
        (_, configs) if configs > 0 => trf(
            "{count} configuration CSV file(s) were opened and applied.",
            &[("count", configs.to_string())],
        ),
        _ => tr("No CSV files were opened."),
    };
    if result.skipped > 0 {
        message.push_str(&trf(
            " {count} file(s) skipped because they were not CSV files.",
            &[("count", result.skipped.to_string())],
        ));
    }
    message
}

pub(super) fn status_with_cache(mut message: String, data: &AppData) -> String {
    match &data.cache_status {
        DataCacheStatus::Disabled | DataCacheStatus::Skipped => {}
        DataCacheStatus::Hit => message.push_str(&tr(" Loaded from the data and analytics cache.")),
        DataCacheStatus::Updated => message.push_str(&tr(" Data and analytics cache updated.")),
        DataCacheStatus::Failed(error) => message.push_str(&trf(
            " Data and analytics cache was skipped: {error}",
            &[("error", error.clone())],
        )),
    }
    message
}
