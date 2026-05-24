use super::forecast::prediction_scope_allows_forecast;
use super::*;

fn app_data_with_years(years: &[i32]) -> AppData {
    AppData {
        available_years: years.to_vec(),
        ..AppData::default()
    }
}

#[test]
fn prediction_scope_blocks_selected_year_when_later_year_exists() {
    let data = app_data_with_years(&[2024, 2025]);

    assert!(!prediction_scope_allows_forecast(&data, Some(2024), 2025));
}

#[test]
fn prediction_scope_blocks_when_future_year_is_loaded() {
    let data = app_data_with_years(&[2026, 2027]);

    assert!(!prediction_scope_allows_forecast(&data, Some(2027), 2026));
}

#[test]
fn prediction_scope_allows_latest_loaded_year_without_next_year() {
    let data = app_data_with_years(&[2024, 2025]);

    assert!(prediction_scope_allows_forecast(&data, Some(2025), 2026));
}
