#[cfg(target_os = "linux")]
use super::availability::installed_model_dir_from_data_dir;
use super::availability::{
    local_ai_model_sources_availability, model_dir_availability, MANIFEST_FILE, MODEL_FILES,
    MODEL_ROOT,
};
use super::configuration::{generated_configuration_from_draft, LOCAL_AI_NOTE};
use super::*;
use crate::data::GeneratedConfigurationSummary;
use crate::model::Transaction;
use chrono::NaiveDate;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn smart_insights_off_disables_local_ai() {
    let provider = LocalAiProvider::new(false);
    assert_eq!(provider.availability(), &LocalAiAvailability::Disabled);
}

#[test]
fn input_sanitization_excludes_sensitive_fields() {
    let mut data = AppData::default();
    data.transactions.push(transaction(
        "2025-01-01",
        "-12.34",
        "Card 123456789 groceries jane@example.test",
        "Market 123456789",
        "food;weekly",
        "NL00BANK0123456789",
        "secret-row-id",
    ));

    let input = LocalAiInput::from_app_data(&data, "nl_NL");
    let json = serde_json::to_string(&input).expect("input should serialize");
    assert!(json.contains("Market"));
    assert!(json.contains("groceries"));
    assert!(!json.contains("123456789"));
    assert!(!json.contains("jane@example.test"));
    assert!(!json.contains("NL00BANK"));
    assert!(!json.contains("secret-row-id"));
    assert!(!json.contains("2025-01-01"));
    assert!(!json.contains("12.34"));
}

#[test]
fn valid_ai_draft_becomes_generated_configuration() {
    let fallback = GeneratedConfiguration {
        rules: Vec::new(),
        budgets: Vec::new(),
        aliases: Vec::new(),
        ignored_patterns: Vec::new(),
        summary: GeneratedConfigurationSummary {
            complete_years: 1,
            budget_months: 12,
            ..GeneratedConfigurationSummary::default()
        },
    };
    let draft = serde_json::from_str::<LocalAiDraft>(
        r#"{
            "budgets": [{
                "code": "FOOD",
                "category": "Food",
                "monthly_budget": "100",
                "yearly_budget": "",
                "direction": "expense",
                "income_basis": "real",
                "notes": ""
            }],
            "rules": [{
                "priority": 120,
                "active": true,
                "field": "counterparty",
                "search": "Market",
                "is_regex": false,
                "category": "Food",
                "budget_code": "FOOD",
                "direction": "expense",
                "amount_min": "",
                "amount_max": "",
                "notes": ""
            }],
            "aliases": [],
            "ignored_patterns": []
        }"#,
    )
    .expect("draft should parse");

    let generated = generated_configuration_from_draft(draft, &fallback)
        .expect("draft should validate")
        .expect("draft should produce config");
    assert_eq!(generated.summary.budgets, 1);
    assert_eq!(generated.summary.rules, 1);
    assert_eq!(generated.summary.complete_years, 1);
    assert_eq!(generated.budgets[0].notes, LOCAL_AI_NOTE);
}

#[test]
fn invalid_ai_regex_is_rejected() {
    let fallback = GeneratedConfiguration {
        rules: Vec::new(),
        budgets: vec![EditableBudget {
            code: "FOOD".to_string(),
            category: "Food".to_string(),
            monthly_budget: "100".to_string(),
            yearly_budget: String::new(),
            direction: "expense".to_string(),
            income_basis: "real".to_string(),
            notes: String::new(),
        }],
        aliases: Vec::new(),
        ignored_patterns: Vec::new(),
        summary: GeneratedConfigurationSummary::default(),
    };
    let draft = LocalAiDraft {
        budgets: fallback.budgets.clone(),
        rules: vec![EditableRule {
            priority: 120,
            active: true,
            field: "counterparty".to_string(),
            search: "(".to_string(),
            is_regex: true,
            category: "Food".to_string(),
            budget_code: "FOOD".to_string(),
            direction: "expense".to_string(),
            amount_min: String::new(),
            amount_max: String::new(),
            notes: String::new(),
        }],
        aliases: Vec::new(),
        ignored_patterns: Vec::new(),
        pattern_hints: Vec::new(),
    };

    assert!(generated_configuration_from_draft(draft, &fallback).is_err());
}

#[test]
fn placeholder_model_dir_is_not_available() {
    let root = unique_test_dir("placeholder-model");
    let dir = root.join(MODEL_ROOT);
    write_test_model_dir(&dir, true);

    let availability = model_dir_availability(LocalAiModelSource::Sidecar(dir))
        .expect("complete model dir should produce an availability value");
    assert!(matches!(availability, LocalAiAvailability::MissingModel(_)));
}

#[test]
fn source_scan_reports_placeholder_assets_before_generic_missing_message() {
    let root = unique_test_dir("placeholder-source-scan");
    let dir = root.join(MODEL_ROOT);
    write_test_model_dir(&dir, true);

    let availability = local_ai_model_sources_availability(vec![LocalAiModelSource::Sidecar(dir)]);

    assert!(matches!(
        availability,
        Some(LocalAiAvailability::MissingModel(reason))
            if reason.contains("placeholder model assets")
    ));
}

#[cfg(target_os = "linux")]
#[test]
fn installed_model_dir_uses_linux_data_layout() {
    assert_eq!(
        installed_model_dir_from_data_dir(std::path::Path::new("/usr/share")),
        std::path::PathBuf::from("/usr/share/bank-files/models/ai")
    );
}

#[test]
fn non_placeholder_model_dir_is_available() {
    let root = unique_test_dir("real-model");
    let dir = root.join(MODEL_ROOT);
    write_test_model_dir(&dir, false);

    let availability = model_dir_availability(LocalAiModelSource::Sidecar(dir.clone()))
        .expect("complete model dir should produce an availability value");
    assert_eq!(
        availability,
        LocalAiAvailability::Available {
            source: LocalAiModelSource::Sidecar(dir)
        }
    );
}

#[test]
fn pattern_hint_matching_uses_normalized_labels() {
    let hints = vec![LocalAiPatternHint {
        label: "Café Market".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        reason: "local".to_string(),
    }];
    assert!(pattern_hint_for_label(&hints, "Cafe Market").is_some());
}

fn write_test_model_dir(dir: &Path, placeholder: bool) {
    fs::create_dir_all(dir).expect("model dir should be created");
    for file in MODEL_FILES {
        let contents = if file == MANIFEST_FILE {
            format!("{{\"placeholder\":{placeholder}}}")
        } else {
            "x".to_string()
        };
        fs::write(dir.join(file), contents).expect("model file should be written");
    }
}

fn transaction(
    date: &str,
    amount: &str,
    description: &str,
    counterparty: &str,
    tags: &str,
    account: &str,
    transaction_id: &str,
) -> Transaction {
    Transaction {
        date: NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("date should parse"),
        amount: amount.parse().expect("amount should parse"),
        description: description.to_string(),
        counterparty: counterparty.to_string(),
        tags: tags.to_string(),
        account: account.to_string(),
        transaction_id: transaction_id.to_string(),
        currency: "EUR".to_string(),
        source_file: "test.csv".to_string(),
        source_row: 1,
        category: String::new(),
        budget_code: String::new(),
        notes: String::new(),
        strict_key: String::new(),
        loose_key: String::new(),
    }
}

fn unique_test_dir(name: &str) -> PathBuf {
    let unique = format!(
        "{}-{}-{}",
        name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should be available")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    fs::create_dir_all(&dir).expect("test dir should be created");
    dir
}
