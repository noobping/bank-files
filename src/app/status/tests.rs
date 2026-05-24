use super::history::{status_log_rows, status_log_text};
use super::history_rows::{status_log_keywords, status_log_matches_keywords};
use super::*;

#[test]
fn status_log_search_matches_timestamp_and_message() {
    let entry = StatusLogEntry {
        timestamp: "12:34:56".to_string(),
        message: "CSV import finished".to_string(),
    };
    let keywords = status_log_keywords(&entry);

    assert!(status_log_matches_keywords(&keywords, "12:34"));
    assert!(status_log_matches_keywords(&keywords, "import"));
    assert!(status_log_matches_keywords(&keywords, ""));
    assert!(!status_log_matches_keywords(&keywords, "backup"));
}

#[test]
fn status_log_search_supports_negative_terms() {
    let debug_entry = StatusLogEntry {
        timestamp: "12:34:56".to_string(),
        message: "[debug] render started".to_string(),
    };
    let normal_entry = StatusLogEntry {
        timestamp: "12:35:00".to_string(),
        message: "CSV import finished".to_string(),
    };
    let debug_keywords = status_log_keywords(&debug_entry);
    let normal_keywords = status_log_keywords(&normal_entry);

    assert!(!status_log_matches_keywords(&debug_keywords, "!debug"));
    assert!(status_log_matches_keywords(&normal_keywords, "!debug"));
    assert!(status_log_matches_keywords(&normal_keywords, "csv !debug"));
    assert!(!status_log_matches_keywords(
        &debug_keywords,
        "render !debug"
    ));
    assert!(status_log_matches_keywords(&debug_keywords, "!"));
}

#[test]
fn status_log_text_includes_all_entries_and_sanitizes_messages() {
    let entries = vec![
        StatusLogEntry {
            timestamp: "12:34:56".to_string(),
            message: "CSV import finished".to_string(),
        },
        StatusLogEntry {
            timestamp: "12:35:00".to_string(),
            message: "Line one\nline two".to_string(),
        },
    ];

    assert_eq!(
        status_log_text(&entries),
        "12:34:56\tCSV import finished\n12:35:00\tLine one line two"
    );
}

#[test]
fn status_log_rows_include_printable_timestamp_and_message() {
    let entries = vec![StatusLogEntry {
        timestamp: "12:35:00".to_string(),
        message: "Line one\nline two".to_string(),
    }];

    assert_eq!(
        status_log_rows(&entries),
        vec![vec![
            "12:35:00".to_string(),
            "Line one line two".to_string(),
        ]]
    );
}

#[test]
fn page_action_snapshot_builds_copy_text_and_csv() {
    let snapshot = PageActionSnapshot::from_rows(
        "sample",
        "Sample Page",
        "Rows visible on the sample page.",
        vec!["Name".to_string(), "Notes".to_string()],
        vec![vec![
            "Groceries".to_string(),
            "Line one
line two"
                .to_string(),
        ]],
    )
    .expect("snapshot should serialize to CSV");

    assert_eq!(
        snapshot.text,
        [
            tr("Sample Page"),
            tr("Rows visible on the sample page."),
            String::new(),
            format!("{}	{}", tr("Name"), tr("Notes")),
            "Groceries	Line one line two".to_string(),
        ]
        .join("\n")
    );
    assert!(snapshot.csv.contains("Name"));
    assert!(snapshot.csv.contains("Groceries"));
}
