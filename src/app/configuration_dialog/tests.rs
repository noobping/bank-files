use super::snapshot::configuration_snapshot_rows;
use super::*;

#[test]
fn configuration_snapshot_lists_configuration_actions() {
    let rows = configuration_snapshot_rows();

    assert_eq!(rows.len(), 4);
    assert!(rows
        .iter()
        .any(|row| row[1] == tr("Back Up Current Configuration")));
    assert!(rows
        .iter()
        .any(|row| row[1] == tr("Restore Configuration Backup")));
    assert!(rows
        .iter()
        .any(|row| row[1] == tr("Use Default Configuration")));
    assert!(rows
        .iter()
        .any(|row| row[1] == tr("Use Empty Configuration")));
}
