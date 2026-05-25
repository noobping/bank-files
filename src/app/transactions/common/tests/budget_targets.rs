use super::super::*;
use super::{budget, tx};

#[test]
fn simple_mode_budget_move_targets_match_transaction_amount_direction() {
    let transaction = tx(-100, "FOOD", "Groceries");
    let budgets = vec![
        budget("FOOD", BudgetDirection::Expense),
        budget("OTHER", BudgetDirection::Expense),
        budget("INC", BudgetDirection::Income),
        budget("SALARY", BudgetDirection::Income),
        budget("TRANSFER", BudgetDirection::Transfer),
    ];

    let simple_targets = transaction_budget_move_targets(&transaction, &budgets, false)
        .into_iter()
        .map(|target| target.code)
        .collect::<Vec<_>>();
    assert_eq!(simple_targets, vec!["FOOD", "OTHER"]);

    let advanced_targets = transaction_budget_move_targets(&transaction, &budgets, true)
        .into_iter()
        .map(|target| target.code)
        .collect::<Vec<_>>();
    assert_eq!(
        advanced_targets,
        vec!["FOOD", "OTHER", "INC", "SALARY", "TRANSFER"]
    );
}

#[test]
fn simple_mode_income_move_targets_include_inc_even_when_current_code_is_expense() {
    let transaction = tx(100, "OTHER", "Other");
    let budgets = vec![
        budget("OTHER", BudgetDirection::Expense),
        budget("INC", BudgetDirection::Income),
        budget("INC-OTHER", BudgetDirection::Income),
    ];

    let simple_targets = transaction_budget_move_targets(&transaction, &budgets, false)
        .into_iter()
        .map(|target| target.code)
        .collect::<Vec<_>>();
    assert_eq!(simple_targets, vec!["INC", "INC-OTHER"]);
    assert!(transaction_budget_move_available(
        &transaction,
        &budgets,
        false
    ));
}

#[test]
fn budget_move_is_hidden_when_there_is_only_one_target() {
    let transfer = tx(-100, "TRANSFER", "Transfers");
    let budgets = vec![budget("TRANSFER", BudgetDirection::Transfer)];

    assert!(!transaction_budget_move_available(
        &transfer, &budgets, false
    ));
    assert!(
        !visible_transaction_detail_actions(true, false, false, false, false)
            .contains(&TransactionDetailAction::MoveBudgetCode)
    );
}

#[test]
fn budget_move_is_visible_when_an_alternative_target_exists() {
    let transaction = tx(-100, "FOOD", "Groceries");
    let budgets = vec![
        budget("FOOD", BudgetDirection::Expense),
        budget("OTHER", BudgetDirection::Expense),
    ];

    assert!(transaction_budget_move_available(
        &transaction,
        &budgets,
        false
    ));
    assert!(
        visible_transaction_detail_actions(false, true, true, true, false)
            .contains(&TransactionDetailAction::MoveBudgetCode)
    );
}

#[test]
fn simple_mode_blocks_targets_that_do_not_match_transaction_amount_direction() {
    let expense_transaction = tx(-100, "FOOD", "Groceries");
    let income_transaction = tx(100, "OTHER", "Other");
    let budgets = vec![budget("FOOD", BudgetDirection::Expense)];
    let income_target = TransactionBudgetTarget {
        code: "SALARY".to_string(),
        category: "Salary".to_string(),
        description: "Monthly pay".to_string(),
        direction: BudgetDirection::Income,
    };
    let expense_target = TransactionBudgetTarget {
        code: "FOOD".to_string(),
        category: "Groceries".to_string(),
        description: "Food and household shopping".to_string(),
        direction: BudgetDirection::Expense,
    };

    assert!(!transaction_budget_target_allowed(
        &expense_transaction,
        &budgets,
        &income_target,
        false,
    ));
    assert!(transaction_budget_target_allowed(
        &expense_transaction,
        &budgets,
        &income_target,
        true,
    ));
    assert!(transaction_budget_target_allowed(
        &income_transaction,
        &budgets,
        &income_target,
        false,
    ));
    assert!(!transaction_budget_target_allowed(
        &income_transaction,
        &budgets,
        &expense_target,
        false,
    ));
}

#[test]
fn budget_move_dialog_title_uses_match_value_only() {
    assert_eq!(
        transaction_budget_move_dialog_title("FNV", "Move Category"),
        "FNV"
    );
    assert_eq!(
        transaction_budget_move_dialog_title("", "Move Category"),
        tr("Move Category")
    );
}

#[test]
fn budget_move_list_height_tracks_window_height() {
    assert_eq!(transaction_budget_move_list_max_height_for_window(0), 620);
    assert_eq!(transaction_budget_move_list_max_height_for_window(320), 224);
    assert_eq!(transaction_budget_move_list_max_height_for_window(800), 704);
}

#[test]
fn advanced_budget_move_form_save_tracks_changed_values() {
    let initial = EditableRule {
        priority: 140,
        active: true,
        field: "counterparty".to_string(),
        search: "Store".to_string(),
        is_regex: false,
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        direction: "expense".to_string(),
        amount_min: String::new(),
        amount_max: String::new(),
        notes: String::new(),
    };

    assert!(!transaction_budget_move_form_values_changed(
        &initial,
        "Groceries",
        "FOOD",
        "expense",
    ));
    assert!(transaction_budget_move_form_values_changed(
        &initial,
        "Fixed costs",
        "FOOD",
        "expense",
    ));
    assert!(transaction_budget_move_form_values_changed(
        &initial,
        "Groceries",
        "UTIL",
        "expense",
    ));
    assert!(transaction_budget_move_form_values_changed(
        &initial,
        "Groceries",
        "FOOD",
        "income",
    ));
}

#[test]
fn budget_move_list_save_requires_changed_target() {
    let transaction = tx(-100, "FOOD", "Groceries");
    let current = TransactionBudgetTarget {
        code: "FOOD".to_string(),
        category: "Groceries".to_string(),
        description: "Food and household shopping".to_string(),
        direction: BudgetDirection::Expense,
    };
    let other = TransactionBudgetTarget {
        code: "OTHER".to_string(),
        category: "Other".to_string(),
        description: "Other expenses".to_string(),
        direction: BudgetDirection::Expense,
    };

    assert!(!transaction_budget_target_is_changed(
        &transaction,
        &current,
        false
    ));
    assert!(transaction_budget_target_is_changed(
        &transaction,
        &other,
        false
    ));
}

#[test]
fn simple_budget_move_current_target_matches_visible_category() {
    let transaction = tx(-100, "HIDDEN", "Fixed costs");
    let same_category = TransactionBudgetTarget {
        code: "UTIL".to_string(),
        category: "Fixed costs".to_string(),
        description: "Monthly bills".to_string(),
        direction: BudgetDirection::Expense,
    };

    assert!(transaction_budget_target_is_current(
        &transaction,
        &same_category,
        false
    ));
    assert!(!transaction_budget_target_is_current(
        &transaction,
        &same_category,
        true
    ));
}

#[test]
fn budget_move_list_subtitle_uses_direction_and_description() {
    let target = TransactionBudgetTarget {
        code: "SHOP".to_string(),
        category: "Groceries".to_string(),
        description: "Food and household shopping".to_string(),
        direction: BudgetDirection::Expense,
    };

    assert_eq!(
        transaction_budget_target_subtitle(&target, false),
        format!("{} · Food and household shopping", tr("Expenses"))
    );
    assert_eq!(
        transaction_budget_target_subtitle(&target, true),
        format!("SHOP · {} · Food and household shopping", tr("Expenses"))
    );
}

#[test]
fn budget_move_list_search_includes_budget_code_only_in_advanced_mode() {
    let target = TransactionBudgetTarget {
        code: "SHOP".to_string(),
        category: "Groceries".to_string(),
        description: "Food and groceries".to_string(),
        direction: BudgetDirection::Expense,
    };

    let simple_keywords = transaction_budget_target_search_keywords(
        &target,
        false,
        "Matching by Counterparty: Store",
    )
    .join(" ")
    .to_lowercase();
    let advanced_keywords =
        transaction_budget_target_search_keywords(&target, true, "Matching by Counterparty: Store")
            .join(" ")
            .to_lowercase();

    assert!(simple_keywords.contains("grocer"));
    assert!(!simple_keywords.contains("shop"));
    assert!(advanced_keywords.contains("shop"));
}

#[test]
fn budget_move_more_options_are_advanced_only() {
    assert!(!transaction_budget_more_options_visible(false));
    assert!(transaction_budget_more_options_visible(true));
}
