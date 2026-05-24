use super::groups::{
    automatic_configuration_generation_enabled, automatic_configuration_generation_visible,
};
use super::snapshot::configuration_snapshot_rows;
use super::*;

#[test]
fn automatic_configuration_generation_hides_in_simple_mode() {
    assert!(!automatic_configuration_generation_visible(false));
}

#[test]
fn automatic_configuration_generation_shows_disabled_in_advanced_mode_when_feature_exists() {
    assert_eq!(
        automatic_configuration_generation_visible(true),
        cfg!(feature = "smart-insights")
    );
    assert!(!automatic_configuration_generation_enabled(true, false));
    assert!(!automatic_configuration_generation_enabled(false, true));
}

#[test]
fn configuration_snapshot_follows_generation_visibility() {
    let simple_rows = configuration_snapshot_rows(false, false);
    let advanced_rows = configuration_snapshot_rows(true, false);
    let expected_advanced_rows = if cfg!(feature = "smart-insights") {
        5
    } else {
        4
    };

    assert_eq!(simple_rows.len(), 4);
    assert_eq!(advanced_rows.len(), expected_advanced_rows);
    assert!(!simple_rows.iter().any(|row| row[0] == tr("Experimental")));
    assert_eq!(
        advanced_rows.iter().any(|row| row[0] == tr("Experimental")),
        cfg!(feature = "smart-insights")
    );
}
