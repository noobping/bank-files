use super::*;

pub(in crate::app) fn transaction_title(tx: &Transaction) -> String {
    if tx.counterparty.trim().is_empty() {
        tx.description.clone()
    } else {
        tx.counterparty.clone()
    }
}

pub(in crate::app) fn transaction_subtitle(tx: &Transaction) -> String {
    if tx.tags.trim().is_empty() {
        format!(
            "{} · {} · {} · {}",
            tx.date, tx.category, tx.budget_code, tx.description
        )
    } else {
        format!(
            "{} · {} · {} · {} · {}",
            tx.date, tx.category, tx.budget_code, tx.tags, tx.description
        )
    }
}

pub(crate) fn transaction_search_text(tx: &Transaction) -> String {
    format!(
        "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
        tx.date,
        signed_money(tx.amount),
        tx.amount,
        tx.description,
        tx.counterparty,
        tx.tags,
        tx.account,
        tx.transaction_id,
        tx.currency,
        tx.source_file,
        tx.source_row,
        tx.category,
        tx.budget_code,
        tx.notes,
        tx.strict_key,
        tx.loose_key,
    )
}

pub(in crate::app) fn transaction_matches(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
    filter: Option<&SearchFilter>,
) -> bool {
    filter
        .map(|filter| filter.matches_transaction(tx, budgets))
        .unwrap_or(true)
}

pub(in crate::app) fn filtered_transactions<'a>(
    transactions: &'a [Transaction],
    budgets: &[crate::model::BudgetCode],
    filter: Option<&SearchFilter>,
) -> Vec<&'a Transaction> {
    transactions
        .iter()
        .filter(|tx| transaction_matches(tx, budgets, filter))
        .collect()
}

pub(in crate::app) fn transaction_list(
    transactions: &[&Transaction],
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> gtk::ListBox {
    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    list.set_selection_mode(gtk::SelectionMode::None);
    for tx in transactions {
        let title = markup_escape(&truncate(&transaction_title(tx), 80));
        let subtitle = markup_escape(&truncate(&transaction_subtitle(tx), 140));
        let row = adw::ActionRow::builder()
            .title(title)
            .subtitle(subtitle)
            .build();
        row.set_tooltip_text(Some(&tr("Click to show transaction details")));

        let direction_icon = gtk::Image::from_icon_name(if tx.amount >= Decimal::ZERO {
            "go-up-symbolic"
        } else {
            "go-down-symbolic"
        });
        direction_icon.add_css_class(if tx.amount >= Decimal::ZERO {
            "success"
        } else {
            "error"
        });
        row.add_prefix(&direction_icon);

        let amount = gtk::Label::new(Some(&signed_money(tx.amount)));
        amount.add_css_class(if tx.amount >= Decimal::ZERO {
            "success"
        } else {
            "error"
        });
        amount.set_xalign(1.0);
        row.add_suffix(&amount);

        let expand_icon = gtk::Image::from_icon_name("pan-down-symbolic");
        expand_icon.add_css_class("dim-label");
        row.add_suffix(&expand_icon);

        let revealer = gtk::Revealer::builder()
            .transition_type(gtk::RevealerTransitionType::SlideDown)
            .build();

        let click = gtk::GestureClick::new();
        click.set_button(0);
        let tx_for_details = (**tx).clone();
        let state_for_details = Rc::clone(state);
        let ui_for_details = Rc::clone(ui_handles);
        let revealer_for_click = revealer.clone();
        let expand_icon_for_click = expand_icon.clone();
        let details_built = std::rc::Rc::new(std::cell::Cell::new(false));
        let details_built_for_click = std::rc::Rc::clone(&details_built);
        click.connect_released(move |_, _, _, _| {
            let reveal = !revealer_for_click.reveals_child();
            if reveal && !details_built_for_click.get() {
                let details =
                    transaction_details_table(&tx_for_details, &state_for_details, &ui_for_details);
                revealer_for_click.set_child(Some(&details));
                details_built_for_click.set(true);
            }
            revealer_for_click.set_reveal_child(reveal);
            expand_icon_for_click.set_icon_name(Some(if reveal {
                "pan-up-symbolic"
            } else {
                "pan-down-symbolic"
            }));
        });
        row.add_controller(click);

        let item = gtk::Box::new(gtk::Orientation::Vertical, 0);
        item.append(&row);
        item.append(&revealer);
        list.append(&item);
    }
    list
}

fn transaction_details_table(
    tx: &Transaction,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> gtk::Box {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 10);
    content.set_margin_top(0);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let details = gtk::Box::new(gtk::Orientation::Vertical, 8);
    details.set_hexpand(true);

    let mut rows = vec![
        ("Date", tx.date.to_string()),
        ("Amount", signed_money(tx.amount)),
        ("Counterparty", tx.counterparty.clone()),
        ("Description", tx.description.clone()),
        ("Tags", tx.tags.clone()),
        ("Category", tx.category.clone()),
    ];
    if ui_handles.advanced_features.get() {
        rows.push(("Budget code", tx.budget_code.clone()));
    }
    rows.extend([
        ("Account", tx.account.clone()),
        ("Transaction ID", tx.transaction_id.clone()),
        ("Currency", tx.currency.clone()),
        ("Source file", tx.source_file.clone()),
        ("Notes", tx.notes.clone()),
    ]);

    for (label, value) in rows {
        details.append(&transaction_detail_row(label, &value));
    }

    content.append(&details);
    content.append(&transaction_detail_actions(tx, state, ui_handles));
    content
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum TransactionDetailAction {
    CreateRule,
    EditBudgetCode,
    MoveBudgetCode,
    DuplicateAsFake,
    MarkTransfer,
    Similar,
    FindPattern,
}

fn visible_transaction_detail_actions(
    advanced_features: bool,
    smart_patterns_enabled: bool,
    markable_as_transfer: bool,
    budget_move_available: bool,
) -> Vec<TransactionDetailAction> {
    [
        TransactionDetailAction::CreateRule,
        TransactionDetailAction::EditBudgetCode,
        TransactionDetailAction::MoveBudgetCode,
        TransactionDetailAction::DuplicateAsFake,
        TransactionDetailAction::MarkTransfer,
        TransactionDetailAction::Similar,
        TransactionDetailAction::FindPattern,
    ]
    .into_iter()
    .filter(|action| {
        transaction_detail_action_visible(
            *action,
            advanced_features,
            smart_patterns_enabled,
            markable_as_transfer,
            budget_move_available,
        )
    })
    .collect()
}

fn transaction_detail_action_visible(
    action: TransactionDetailAction,
    advanced_features: bool,
    smart_patterns_enabled: bool,
    markable_as_transfer: bool,
    budget_move_available: bool,
) -> bool {
    let visible = match action {
        TransactionDetailAction::CreateRule | TransactionDetailAction::EditBudgetCode => {
            advanced_features
        }
        TransactionDetailAction::MoveBudgetCode => budget_move_available,
        TransactionDetailAction::DuplicateAsFake
        | TransactionDetailAction::MarkTransfer
        | TransactionDetailAction::Similar => true,
        TransactionDetailAction::FindPattern => smart_patterns_enabled,
    };
    visible && (action != TransactionDetailAction::MarkTransfer || markable_as_transfer)
}

fn transaction_detail_actions(
    tx: &Transaction,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> gtk::Box {
    let actions = gtk::Box::new(gtk::Orientation::Vertical, 6);
    actions.set_hexpand(true);

    let safe_actions = ui::linked_button_group();
    safe_actions.set_halign(gtk::Align::Start);
    let search_actions = ui::linked_button_group();
    search_actions.set_halign(gtk::Align::Start);
    let advanced_actions = ui::linked_button_group();
    advanced_actions.set_halign(gtk::Align::Start);

    let advanced_features = ui_handles.advanced_features.get();
    let smart_patterns_enabled = smart_pattern_detection_enabled(ui_handles.show_predictions.get());
    let (markable_as_transfer, budget_move_available) = {
        let data = state.borrow();
        (
            transaction_is_markable_as_transfer(tx, &data.budgets),
            transaction_budget_move_available(tx, &data.budgets, advanced_features),
        )
    };
    let visible_actions = visible_transaction_detail_actions(
        advanced_features,
        smart_patterns_enabled,
        markable_as_transfer,
        budget_move_available,
    );

    if visible_actions.contains(&TransactionDetailAction::MoveBudgetCode) {
        let tx_for_change = tx.clone();
        let state_for_change = Rc::clone(state);
        let ui_for_change = Rc::clone(ui_handles);
        let (move_label, move_tooltip) = if advanced_features {
            (
                "Move Budget Code",
                "Move this transaction to another budget code",
            )
        } else {
            ("Move Category", "Move this transaction to another category")
        };
        let move_button =
            ui::primary_text_icon_button("send-to-symbolic", move_label, move_tooltip);
        register_config_widget(ui_handles, &move_button);
        move_button.connect_clicked(move |_| {
            show_transaction_budget_code_dialog(&tx_for_change, &state_for_change, &ui_for_change);
        });
        safe_actions.append(&move_button);
    }

    if visible_actions.contains(&TransactionDetailAction::MarkTransfer) {
        let tx_for_transfer = tx.clone();
        let ui_for_transfer = Rc::clone(ui_handles);
        let button = ui::plain_text_icon_button(
            "send-to-symbolic",
            "Mark transfer",
            "Create a transfer rule from this transaction",
        );
        register_config_widget(ui_handles, &button);
        button.connect_clicked(move |_| {
            apply_transaction_direction_rule(&tx_for_transfer, "transfer", &ui_for_transfer);
        });
        safe_actions.append(&button);
    }

    let tx_for_fake = tx.clone();
    let state_for_fake = Rc::clone(state);
    let ui_for_fake = Rc::clone(ui_handles);
    let button = ui::plain_text_icon_button(
        "document-new-symbolic",
        "Duplicate as Fake",
        "Duplicate this transaction as a runtime fake transaction",
    );
    button.connect_clicked(move |_| {
        duplicate_transaction_as_fake(&state_for_fake, &ui_for_fake, &tx_for_fake);
    });
    safe_actions.append(&button);

    let tx_for_similar = tx.clone();
    let state_for_similar = Rc::clone(state);
    let ui_for_similar = Rc::clone(ui_handles);
    let button =
        ui::plain_text_icon_button("edit-find-symbolic", "Similar", "Show similar transactions");
    button.connect_clicked(move |_| {
        show_transactions_text_search(
            &state_for_similar,
            &ui_for_similar,
            &similar_transaction_query(&tx_for_similar),
            "Showing similar transactions.",
        );
    });
    search_actions.append(&button);

    if visible_actions.contains(&TransactionDetailAction::FindPattern) {
        let tx_for_pattern = tx.clone();
        let state_for_pattern = Rc::clone(state);
        let ui_for_pattern = Rc::clone(ui_handles);
        let button = ui::plain_text_icon_button(
            "view-refresh-symbolic",
            "Find pattern",
            "Search Diagnostics for related transaction patterns",
        );
        button.connect_clicked(move |_| {
            show_diagnostics_text_search(
                &state_for_pattern,
                &ui_for_pattern,
                &similar_transaction_query(&tx_for_pattern),
            );
        });
        search_actions.append(&button);
    }

    if visible_actions.contains(&TransactionDetailAction::CreateRule) {
        let tx_for_rule = tx.clone();
        let state_for_rule = Rc::clone(state);
        let ui_for_rule = Rc::clone(ui_handles);
        let button = ui::plain_text_icon_button(
            "document-new-symbolic",
            "Create rule",
            "Create a categorization rule from this transaction",
        );
        register_config_widget(ui_handles, &button);
        button.connect_clicked(move |_| {
            show_transaction_rule_dialog(&tx_for_rule, &state_for_rule, &ui_for_rule, None);
        });
        advanced_actions.append(&button);

        let tx_for_budget = tx.clone();
        let state_for_budget = Rc::clone(state);
        let ui_for_budget = Rc::clone(ui_handles);
        let button = ui::plain_text_icon_button(
            "document-edit-symbolic",
            "Budget code",
            "Create or edit the budget code for this transaction",
        );
        register_exclusive_config_widget(ui_handles, &button);
        button.connect_clicked(move |_| {
            let code = suggested_budget_code(&tx_for_budget, None);
            let category = suggested_category(&tx_for_budget, None);
            if config_operation_is_active(
                &ui_for_budget,
                "Another edit or save is already running.",
            ) {
                return;
            }
            show_budget_edit_dialog(&code, &category, &state_for_budget, &ui_for_budget);
        });
        advanced_actions.append(&button);
    }

    if safe_actions.first_child().is_some() {
        actions.append(&safe_actions);
    }
    if search_actions.first_child().is_some() {
        actions.append(&search_actions);
    }
    if advanced_actions.first_child().is_some() {
        actions.append(&advanced_actions);
    }
    actions
}

fn show_transaction_rule_dialog(
    tx: &Transaction,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    direction_override: Option<&str>,
) {
    let initial = editable_rule_for_transaction(tx, direction_override);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header = ui::cancelable_dialog_header("Create Rule", &transaction_title(tx));

    let cancel_button = gtk::Button::with_label(&tr("Cancel"));
    cancel_button.add_css_class("flat");
    let save_button = ui::primary_text_icon_button("document-save-symbolic", "Save", "Save rule");
    register_config_widget(ui_handles, &save_button);
    header.pack_start(&cancel_button);
    header.pack_end(&save_button);
    root.append(&header);

    let page = ui::page_box();
    page.append(&ui::section_title(
        "Create Rule",
        "Create a categorization rule from this transaction.",
    ));

    let grid = ui::form_grid();
    let active = gtk::Switch::builder()
        .active(initial.active)
        .valign(gtk::Align::Center)
        .build();
    let priority = gtk::SpinButton::with_range(0.0, 1000.0, 1.0);
    priority.set_value(initial.priority as f64);
    let field = ui::combo_from_options(
        &[
            ("any", "Everything"),
            ("counterparty", "Counterparty"),
            ("description", "Description"),
            ("tags", "Tags"),
            ("account", "Account"),
            ("transaction_id", "Transaction ID"),
        ],
        &initial.field,
    );
    let search = rule_search_combo(tx, &initial.search);
    let is_regex = gtk::Switch::builder()
        .active(initial.is_regex)
        .valign(gtk::Align::Center)
        .build();
    let category = category_combo(&state.borrow(), &initial.category);
    let budget_code = budget_code_combo(&state.borrow(), &initial.budget_code);
    let direction = ui::combo_from_options(
        &[
            ("any", "All transactions"),
            ("expense", "Expenses"),
            ("income", "Income"),
            ("transfer", "Transfers"),
        ],
        &initial.direction,
    );
    connect_budget_fields_autofill(
        &category,
        &budget_code,
        &direction,
        app_budget_autofill_entries(&state.borrow()),
        &ui_handles.advanced_autofill,
    );
    let amount_min = ui::entry(&initial.amount_min, "Optional");
    let amount_max = ui::entry(&initial.amount_max, "Optional");
    let notes = ui::entry(&initial.notes, "Note");

    ui::add_labeled(&grid, 0, "Active", &active);
    ui::add_labeled(&grid, 1, "Priority", &priority);
    ui::add_labeled(&grid, 2, "Field", &field);
    ui::add_labeled(&grid, 3, "Search Text", &search);
    ui::add_labeled(&grid, 4, "Regex", &is_regex);
    ui::add_labeled(&grid, 5, "Category", &category);
    ui::add_labeled(&grid, 6, "Budget code", &budget_code);
    ui::add_labeled(&grid, 7, "Direction", &direction);
    ui::add_labeled(&grid, 8, "Min amount", &amount_min);
    ui::add_labeled(&grid, 9, "Max amount", &amount_max);
    ui::add_labeled(&grid, 10, "Note", &notes);
    page.append(&grid);

    let status = ui::wrapped_label(&tr("Save adds this rule to the processing queue."));
    status.add_css_class("dim-label");
    page.append(&status);
    root.append(&ui::scroll(&page));

    let dialog = adw::Dialog::builder()
        .title(tr("Create Rule"))
        .content_width(680)
        .content_height(620)
        .default_widget(&save_button)
        .child(&root)
        .build();

    let dialog_for_cancel = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_for_cancel.close();
    });

    let ui_for_save = Rc::clone(ui_handles);
    let dialog_for_save = dialog.clone();
    let cancel_button_for_save = cancel_button.clone();
    save_button.connect_clicked(move |button| {
        let search_text = ui::combo_text(&search);
        let category_text = ui::combo_text(&category);
        if search_text.is_empty() {
            status.set_text(&tr("Enter search text first."));
            search.grab_focus();
            return;
        }
        if category_text.is_empty() {
            status.set_text(&tr("Enter a category first."));
            category.grab_focus();
            return;
        }

        let rule = EditableRule {
            priority: priority.value_as_int(),
            active: active.is_active(),
            field: ui::combo_active_id(&field),
            search: search_text,
            is_regex: is_regex.is_active(),
            category: category_text,
            budget_code: ui::combo_text(&budget_code),
            direction: ui::combo_active_id(&direction),
            amount_min: amount_min.text().trim().to_string(),
            amount_max: amount_max.text().trim().to_string(),
            notes: notes.text().trim().to_string(),
        };

        enqueue_rule_operation(&ui_for_save, rule, true, OperationSource::CreateRule);
        button.set_sensitive(false);
        cancel_button_for_save.set_label(&tr("Close"));
        status.set_text(&tr("Rule added to processing queue."));
        dialog_for_save.close();
    });

    dialog.present(Some(&ui_handles.window));
}

fn apply_transaction_direction_rule(tx: &Transaction, direction: &str, ui_handles: &Rc<UiHandles>) {
    let rule = editable_rule_for_transaction(tx, Some(direction));
    enqueue_rule_operation(ui_handles, rule, true, OperationSource::MarkTransfer);
}

fn show_transaction_budget_code_dialog(
    tx: &Transaction,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let initial = editable_rule_for_transaction(tx, None);
    let advanced_features = ui_handles.advanced_features.get();

    let dialog_title = if advanced_features {
        "Move Budget Code"
    } else {
        "Move Category"
    };
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header = ui::action_dialog_header();

    let cancel_button = gtk::Button::with_label(&tr("Cancel"));
    cancel_button.add_css_class("flat");
    let save_button = ui::primary_text_icon_button(
        "document-save-symbolic",
        "Save",
        if advanced_features {
            "Save budget code move"
        } else {
            "Save category move"
        },
    );
    register_config_widget(ui_handles, &save_button);
    header.pack_start(&cancel_button);
    header.pack_end(&save_button);
    root.append(&header);

    let page = ui::page_box();
    page.append(&ui::section_title(
        dialog_title,
        if advanced_features {
            "Move this transaction to another category or budget code."
        } else {
            "Move this transaction to another category."
        },
    ));

    let match_summary = trf(
        "Matching by {field}: {value}",
        &[
            ("field", tr(rule_field_label(&initial.field))),
            ("value", truncate(&initial.search, 80)),
        ],
    );
    let form = ui::form_box();
    ui::add_labeled_stacked(&form, "Rule match", &ui::wrapped_label(&match_summary));

    let budget_targets = Rc::new(transaction_budget_move_targets(
        tx,
        &state.borrow().budgets,
        advanced_features,
    ));
    let category = category_combo(&state.borrow(), &initial.category);
    let budget_code = if advanced_features {
        budget_code_combo(&state.borrow(), &initial.budget_code)
    } else {
        transaction_budget_target_combo(budget_targets.as_ref(), &initial.budget_code)
    };
    let direction = ui::combo_from_options(
        &[
            ("expense", "Expenses"),
            ("income", "Income"),
            ("transfer", "Transfers"),
        ],
        &initial.direction,
    );
    if advanced_features {
        connect_budget_fields_autofill(
            &category,
            &budget_code,
            &direction,
            app_budget_autofill_entries(&state.borrow()),
            &ui_handles.advanced_autofill,
        );
        ui::add_labeled_stacked(&form, "Category", &category);
        ui::add_labeled_stacked(&form, "Budget code", &budget_code);
        ui::add_labeled_stacked(&form, "Direction", &direction);
    } else {
        category.set_visible(false);
        direction.set_visible(false);
        connect_transaction_budget_target_autofill(
            &budget_code,
            &category,
            &direction,
            Rc::clone(&budget_targets),
        );
        ui::add_labeled_stacked(&form, "Category", &budget_code);
    }
    page.append(&form);

    let status = ui::wrapped_label(&tr(if advanced_features {
        "Save adds a categorization rule to the processing queue. The original bank CSV is not changed."
    } else {
        "Save moves matching transactions to this category. The original bank CSV is not changed."
    }));
    status.add_css_class("dim-label");
    page.append(&status);
    root.append(&ui::scroll(&page));

    let dialog = adw::Dialog::builder()
        .title(tr(dialog_title))
        .content_width(680)
        .content_height(620)
        .default_widget(&save_button)
        .child(&root)
        .build();

    let dialog_for_cancel = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_for_cancel.close();
    });

    let state_for_save = Rc::clone(state);
    let ui_for_save = Rc::clone(ui_handles);
    let dialog_for_save = dialog.clone();
    let tx_for_save = tx.clone();
    let initial_field = initial.field.clone();
    let initial_search = initial.search.clone();
    let initial_is_regex = initial.is_regex;
    let initial_amount_min = initial.amount_min.clone();
    let initial_amount_max = initial.amount_max.clone();
    let cancel_button_for_save = cancel_button.clone();
    let budget_targets_for_save = Rc::clone(&budget_targets);
    save_button.connect_clicked(move |button| {
        let category_text = ui::combo_text(&category);
        let budget_code_text = if advanced_features {
            ui::combo_text(&budget_code)
        } else {
            ui::combo_active_id(&budget_code)
        };
        if budget_code_text.is_empty() {
            status.set_text(&tr(if advanced_features {
                "Enter a budget code first."
            } else {
                "Choose a category first."
            }));
            budget_code.grab_focus();
            return;
        }
        if category_text.is_empty() {
            status.set_text(&tr(if advanced_features {
                "Choose a budget code first."
            } else {
                "Choose a category first."
            }));
            budget_code.grab_focus();
            return;
        }

        let direction_text = if advanced_features {
            ui::combo_active_id(&direction)
        } else {
            let Some(target) = transaction_budget_target_for_code(
                budget_targets_for_save.as_ref(),
                &budget_code_text,
            ) else {
                status.set_text(&tr(
                    "This category is not available in simple mode. Enable Advanced Features to change direction.",
                ));
                budget_code.grab_focus();
                return;
            };
            if !transaction_budget_target_allowed(
                &tx_for_save,
                &state_for_save.borrow().budgets,
                target,
                false,
            ) {
                status.set_text(&tr(
                    "This move changes direction. Enable Advanced Features to continue.",
                ));
                budget_code.grab_focus();
                return;
            }
            target.direction.as_str().to_string()
        };

        let direction_changes = if advanced_features {
            transaction_budget_direction_change(
                &tx_for_save,
                &state_for_save.borrow().budgets,
                &budget_code_text,
                &category_text,
                &direction_text,
            )
            .into_iter()
            .collect()
        } else {
            Vec::new()
        };
        let rule = EditableRule {
            priority: 140,
            active: true,
            field: initial_field.clone(),
            search: initial_search.clone(),
            is_regex: initial_is_regex,
            category: category_text,
            budget_code: budget_code_text,
            direction: direction_text,
            amount_min: initial_amount_min.clone(),
            amount_max: initial_amount_max.clone(),
            notes: tr(if advanced_features {
                "Generated from transaction budget code change."
            } else {
                "Generated from transaction category change."
            }),
        };

        let button = button.clone();
        let cancel_button = cancel_button_for_save.clone();
        let status = status.clone();
        let ui_for_save = Rc::clone(&ui_for_save);
        let dialog_for_confirm = dialog_for_save.clone();
        let dialog_for_save = dialog_for_save.clone();
        confirm_budget_direction_changes(&dialog_for_confirm, direction_changes, move || {
            enqueue_rule_operation(&ui_for_save, rule, true, OperationSource::ChangeBudgetCode);
            button.set_sensitive(false);
            cancel_button.set_label(&tr("Close"));
            status.set_text(&tr(if advanced_features {
                "Budget code move added to processing queue."
            } else {
                "Category move added to processing queue."
            }));
            dialog_for_save.close();
        });
    });

    dialog.present(Some(&ui_handles.window));
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct TransactionBudgetTarget {
    code: String,
    category: String,
    direction: BudgetDirection,
}

fn transaction_budget_move_targets(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
    advanced_features: bool,
) -> Vec<TransactionBudgetTarget> {
    let simple_direction = transaction_budget_simple_move_direction(tx, budgets);
    budgets
        .iter()
        .filter_map(|budget| {
            let code = budget.code.trim();
            if code.is_empty() {
                return None;
            }
            let target = TransactionBudgetTarget {
                code: code.to_string(),
                category: budget.category.trim().to_string(),
                direction: budget.direction,
            };
            transaction_budget_target_allowed(tx, budgets, &target, advanced_features)
                .then_some(target)
        })
        .filter(|target| advanced_features || target.direction == simple_direction)
        .collect()
}

fn transaction_budget_simple_move_direction(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
) -> BudgetDirection {
    if analytics::transaction_is_transfer(tx, budgets) {
        BudgetDirection::Transfer
    } else if tx.amount > Decimal::ZERO {
        BudgetDirection::Income
    } else {
        BudgetDirection::Expense
    }
}

fn transaction_budget_target_allowed(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
    target: &TransactionBudgetTarget,
    advanced_features: bool,
) -> bool {
    advanced_features || target.direction == transaction_budget_simple_move_direction(tx, budgets)
}

fn transaction_budget_move_available(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
    advanced_features: bool,
) -> bool {
    transaction_budget_move_targets(tx, budgets, advanced_features)
        .iter()
        .any(|target| !target.code.eq_ignore_ascii_case(tx.budget_code.trim()))
}

fn transaction_budget_target_for_code<'a>(
    targets: &'a [TransactionBudgetTarget],
    code: &str,
) -> Option<&'a TransactionBudgetTarget> {
    let code = code.trim();
    targets
        .iter()
        .find(|target| target.code.eq_ignore_ascii_case(code))
}

fn transaction_budget_target_combo(
    targets: &[TransactionBudgetTarget],
    active: &str,
) -> gtk::ComboBoxText {
    let combo = gtk::ComboBoxText::new();
    for target in targets {
        combo.append(Some(&target.code), &target.category);
    }
    if !active.trim().is_empty() {
        combo.set_active_id(Some(active.trim()));
    }
    if combo.active_id().is_none() {
        combo.set_active(Some(0));
    }
    combo
}

fn connect_transaction_budget_target_autofill(
    budget_code: &gtk::ComboBoxText,
    category: &gtk::ComboBoxText,
    direction: &gtk::ComboBoxText,
    targets: Rc<Vec<TransactionBudgetTarget>>,
) {
    apply_transaction_budget_target(budget_code, category, direction, targets.as_ref());
    let category = category.clone();
    let direction = direction.clone();
    budget_code.connect_changed(move |combo| {
        apply_transaction_budget_target(combo, &category, &direction, targets.as_ref());
    });
}

fn apply_transaction_budget_target(
    budget_code: &gtk::ComboBoxText,
    category: &gtk::ComboBoxText,
    direction: &gtk::ComboBoxText,
    targets: &[TransactionBudgetTarget],
) {
    let code = ui::combo_active_id(budget_code);
    let Some(target) = transaction_budget_target_for_code(targets, &code) else {
        return;
    };
    set_text_combo_value(category, &target.category);
    direction.set_active_id(Some(target.direction.as_str()));
}

fn set_text_combo_value(combo: &gtk::ComboBoxText, value: &str) {
    combo.set_active_id(if value.trim().is_empty() {
        None
    } else {
        Some(value.trim())
    });
    if let Some(entry) = combo
        .child()
        .and_then(|child| child.downcast::<gtk::Entry>().ok())
    {
        entry.set_text(value.trim());
    }
}

fn rule_search_combo(tx: &Transaction, selected: &str) -> gtk::ComboBoxText {
    ui::text_combo(selected, transaction_rule_search_values(tx))
}

fn category_combo(data: &AppData, selected: &str) -> gtk::ComboBoxText {
    ui::text_combo(selected, app_category_values(data))
}

fn budget_code_combo(data: &AppData, selected: &str) -> gtk::ComboBoxText {
    ui::text_combo(selected, app_budget_code_values(data))
}

fn transaction_is_markable_as_transfer(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
) -> bool {
    !analytics::transaction_is_transfer(tx, budgets)
}

fn transaction_budget_direction_change(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
    new_code: &str,
    new_category: &str,
    new_direction: &str,
) -> Option<BudgetDirectionChange> {
    let from = transaction_budget_direction(tx, budgets);
    let to = BudgetDirection::parse(new_direction, new_code, new_category);
    budget_direction_change(
        &budget_code_change_label(&tx.budget_code, new_code),
        from,
        to,
    )
}

fn transaction_budget_direction(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
) -> BudgetDirection {
    if analytics::transaction_is_transfer(tx, budgets) {
        return BudgetDirection::Transfer;
    }

    let code = tx.budget_code.trim();
    if let Some(budget) = budgets
        .iter()
        .find(|budget| budget.code.trim().eq_ignore_ascii_case(code))
    {
        return budget.direction;
    }

    if code.is_empty() {
        if tx.amount > Decimal::ZERO {
            BudgetDirection::Income
        } else {
            BudgetDirection::Expense
        }
    } else {
        BudgetDirection::parse("", code, &tx.category)
    }
}

fn budget_code_change_label(old_code: &str, new_code: &str) -> String {
    let old_code = old_code.trim();
    let new_code = new_code.trim();
    match (old_code.is_empty(), new_code.is_empty()) {
        (true, true) => tr("this transaction"),
        (true, false) => trf(
            "this transaction -> {new}",
            &[("new", new_code.to_string())],
        ),
        (false, true) => trf("{old} -> no budget code", &[("old", old_code.to_string())]),
        (false, false) => trf(
            "{old} -> {new}",
            &[("old", old_code.to_string()), ("new", new_code.to_string())],
        ),
    }
}

fn rule_field_label(field: &str) -> &'static str {
    match field {
        "counterparty" => "Counterparty",
        "description" => "Description",
        "tags" => "Tags",
        "account" => "Account",
        "transaction_id" => "Transaction ID",
        _ => "Everything",
    }
}

fn editable_rule_for_transaction(
    tx: &Transaction,
    direction_override: Option<&str>,
) -> EditableRule {
    let direction = direction_override.unwrap_or_else(|| transaction_direction_id(tx));
    let (field, search) = transaction_rule_match(tx);
    let category = suggested_category(tx, Some(direction));
    let budget_code = suggested_budget_code(tx, Some(direction));

    EditableRule {
        priority: 140,
        active: true,
        field,
        search,
        is_regex: false,
        category,
        budget_code,
        direction: direction.to_string(),
        amount_min: String::new(),
        amount_max: String::new(),
        notes: tr("Generated from transaction detail."),
    }
}

fn transaction_rule_match(tx: &Transaction) -> (String, String) {
    for (field, value) in [
        ("counterparty", tx.counterparty.trim()),
        ("tags", tx.tags.trim()),
        ("description", tx.description.trim()),
        ("account", tx.account.trim()),
        ("transaction_id", tx.transaction_id.trim()),
    ] {
        if !value.is_empty() {
            return (field.to_string(), value.to_string());
        }
    }
    ("any".to_string(), transaction_search_text(tx))
}

fn transaction_direction_id(tx: &Transaction) -> &'static str {
    if tx.budget_code.trim().eq_ignore_ascii_case("TRANSFER") {
        "transfer"
    } else if tx.amount > Decimal::ZERO {
        "income"
    } else {
        "expense"
    }
}

fn suggested_category(tx: &Transaction, direction: Option<&str>) -> String {
    match direction.unwrap_or_else(|| transaction_direction_id(tx)) {
        "transfer" => tr("Transfers"),
        "income" => non_empty_transaction_text(&tx.category).unwrap_or_else(|| tr("Other income")),
        _ => non_empty_transaction_text(&tx.category).unwrap_or_else(|| tr("Other")),
    }
}

fn suggested_budget_code(tx: &Transaction, direction: Option<&str>) -> String {
    let direction = direction.unwrap_or_else(|| transaction_direction_id(tx));
    let current = tx.budget_code.trim();
    if !current.is_empty()
        && !matches!(current, "OTHER" | "INC-OTHER")
        && !current.eq_ignore_ascii_case("TRANSFER")
    {
        return current.to_string();
    }
    match direction {
        "transfer" => "TRANSFER".to_string(),
        "income" => "INC-OTHER".to_string(),
        _ => suggested_expense_code(tx),
    }
}

fn suggested_expense_code(tx: &Transaction) -> String {
    for value in [
        tx.category.trim(),
        tx.counterparty.trim(),
        tx.description.trim(),
    ] {
        let code = value
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .take(12)
            .collect::<String>()
            .to_ascii_uppercase();
        if code.len() >= 3 && !matches!(code.as_str(), "OTHER" | "UNCATEGORIZ") {
            return code;
        }
    }
    "OTHER".to_string()
}

fn non_empty_transaction_text(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty() && value != "Uncategorized").then(|| value.to_string())
}

fn similar_transaction_query(tx: &Transaction) -> String {
    [
        tx.counterparty.trim(),
        tx.tags.trim(),
        tx.description.trim(),
        tx.budget_code.trim(),
    ]
    .into_iter()
    .find(|value| !value.is_empty())
    .unwrap_or_else(|| tx.transaction_id.trim())
    .to_string()
}

fn show_transactions_text_search(
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    query: &str,
    status: &str,
) {
    ui_handles.stack.set_visible_child_name("transactions");
    *ui_handles.active_transaction_filter.borrow_mut() = None;
    *ui_handles.search_query.borrow_mut() = query.to_string();
    ui_handles.search_bar.set_search_mode(true);
    if ui_handles.search_entry.text().as_str() != query {
        ui_handles.search_entry.set_text(query);
    }
    render_views(&state.borrow(), ui_handles, state);
    show_status(ui_handles, status);
}

fn show_diagnostics_text_search(
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    query: &str,
) {
    if !smart_pattern_detection_enabled(ui_handles.show_predictions.get()) {
        show_status(
            ui_handles,
            "Smart Insights are disabled. Enable Smart Insights to search detected transaction patterns.",
        );
        return;
    }
    ui_handles.stack.set_visible_child_name("debug");
    *ui_handles.active_transaction_filter.borrow_mut() = None;
    *ui_handles.search_query.borrow_mut() = query.to_string();
    ui_handles.search_bar.set_search_mode(true);
    if ui_handles.search_entry.text().as_str() != query {
        ui_handles.search_entry.set_text(query);
    }
    render_views(&state.borrow(), ui_handles, state);
    show_status(ui_handles, "Searching Diagnostics for related patterns.");
}

fn transaction_detail_row(label: &str, value: &str) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 2);
    row.set_hexpand(true);

    let label = gtk::Label::new(Some(&tr(label)));
    label.add_css_class("caption");
    label.add_css_class("dim-label");
    label.set_xalign(0.0);
    row.append(&label);

    let value = if value.trim().is_empty() {
        tr("Not set")
    } else {
        value.trim().to_string()
    };
    let value = gtk::Label::new(Some(&value));
    value.set_xalign(0.0);
    value.set_hexpand(true);
    value.set_selectable(true);
    value.set_wrap(true);
    value.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    row.append(&value);

    row
}

pub(in crate::app) fn search_empty_page(title: &str, description: &str) -> adw::StatusPage {
    empty_page("edit-find-symbolic", title, description)
}

pub(in crate::app) fn markup_escape(text: &str) -> String {
    adw::glib::markup_escape_text(text).to_string()
}

pub(in crate::app) fn append_page_header(
    container: &gtk::Box,
    ui_handles: &UiHandles,
    title: &str,
    subtitle: &str,
    _page_text: String,
    transactions: &[Transaction],
) {
    ui_handles.mobile_header_title.set_title(&tr(title));
    ui_handles.mobile_header_title.set_subtitle(&tr(subtitle));
    let copy_button = ui::icon_button("edit-copy-symbolic", "Copy this page");
    copy_button.set_action_name(Some("app.copy-page"));
    register_page_copy_feedback_button(ui_handles, &copy_button);

    let print_button = ui::icon_button("document-print-symbolic", "Print this page");
    print_button.set_action_name(Some("app.print-page"));

    let export_button = ui::icon_button("document-save-symbolic", "Export CSV");
    export_button.set_sensitive(transactions.iter().any(|tx| !transaction_is_fake(tx)));
    export_button.set_action_name(Some("app.export-csv"));

    let actions = ui::linked_button_group();
    actions.append(&copy_button);
    actions.append(&print_button);
    actions.append(&export_button);

    let page_header = ui::section_title_with_action(title, subtitle, &actions);
    ui_handles
        .mobile_header_title
        .bind_property("visible", &page_header, "visible")
        .sync_create()
        .invert_boolean()
        .build();
    container.append(&page_header);
}

#[derive(Clone)]
pub(in crate::app) struct PageSnapshot {
    pub(in crate::app) text: String,
    pub(in crate::app) transactions: Vec<Transaction>,
}

pub(in crate::app) fn current_page_snapshot(data: &AppData, ui: &UiHandles) -> PageSnapshot {
    page_snapshot(data, ui, true)
}

pub(in crate::app) fn current_real_page_snapshot(data: &AppData, ui: &UiHandles) -> PageSnapshot {
    page_snapshot(data, ui, false)
}

fn page_snapshot(data: &AppData, ui: &UiHandles, include_fake: bool) -> PageSnapshot {
    let runtime_data;
    let data = if include_fake {
        runtime_data = data_with_fake_transactions(data.clone(), ui.fake_transactions.list());
        &runtime_data
    } else {
        data
    };
    let visible = filtered_app_data(data, ui);
    let visible_data = visible.as_ref().unwrap_or(data);
    let transactions = if include_fake {
        visible_data.transactions.clone()
    } else {
        real_transactions(&visible_data.transactions)
    };
    match ui.stack.visible_child_name().as_deref() {
        Some("overview") => PageSnapshot {
            text: summary::render_overview(visible_data),
            transactions: transactions.clone(),
        },
        Some("transactions") => PageSnapshot {
            text: summary::render_transactions(visible_data),
            transactions: transactions.clone(),
        },
        Some("debug") => PageSnapshot {
            text: summary::render_debug(visible_data),
            transactions: transactions.clone(),
        },
        _ => {
            let text = selected_budget_month(visible_data, ui)
                .map(|month| summary::render_categories_for_month(visible_data, month))
                .unwrap_or_else(|| summary::render_categories(visible_data));
            PageSnapshot { text, transactions }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn tx(amount: i64, budget_code: &str, category: &str) -> Transaction {
        Transaction {
            date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            amount: Decimal::new(amount, 0),
            description: "Test transaction".to_string(),
            counterparty: String::new(),
            tags: String::new(),
            account: String::new(),
            transaction_id: String::new(),
            currency: "EUR".to_string(),
            source_file: "test.csv".to_string(),
            source_row: 2,
            category: category.to_string(),
            budget_code: budget_code.to_string(),
            notes: String::new(),
            strict_key: String::new(),
            loose_key: String::new(),
        }
    }

    fn budget(code: &str, direction: BudgetDirection) -> crate::model::BudgetCode {
        crate::model::BudgetCode {
            code: code.to_string(),
            category: code.to_string(),
            monthly_budget: None,
            yearly_budget: None,
            direction,
            income_basis: crate::model::BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        }
    }

    #[test]
    fn transfer_transactions_do_not_show_mark_transfer_action() {
        let transfer = tx(-20, "TRANSFER", "Transfers");
        assert!(!transaction_is_markable_as_transfer(&transfer, &[]));

        let configured_transfer = tx(-20, "MOVE", "Internal move");
        assert!(!transaction_is_markable_as_transfer(
            &configured_transfer,
            &[budget("MOVE", BudgetDirection::Transfer)],
        ));

        let expense = tx(-20, "FOOD", "Groceries");
        assert!(transaction_is_markable_as_transfer(&expense, &[]));
    }

    #[test]
    fn simple_mode_hides_rule_and_budget_editing_transaction_actions() {
        let simple_actions = visible_transaction_detail_actions(false, true, true, true);
        assert!(!simple_actions.contains(&TransactionDetailAction::CreateRule));
        assert!(!simple_actions.contains(&TransactionDetailAction::EditBudgetCode));
        assert!(simple_actions.contains(&TransactionDetailAction::MarkTransfer));
        assert!(simple_actions.contains(&TransactionDetailAction::MoveBudgetCode));
        assert!(simple_actions.contains(&TransactionDetailAction::DuplicateAsFake));
        assert!(simple_actions.contains(&TransactionDetailAction::Similar));
        assert!(simple_actions.contains(&TransactionDetailAction::FindPattern));

        let advanced_actions = visible_transaction_detail_actions(true, true, true, true);
        assert!(advanced_actions.contains(&TransactionDetailAction::CreateRule));
        assert!(advanced_actions.contains(&TransactionDetailAction::EditBudgetCode));
        assert!(advanced_actions.contains(&TransactionDetailAction::MarkTransfer));
        assert!(
            !visible_transaction_detail_actions(false, true, false, true)
                .contains(&TransactionDetailAction::MarkTransfer)
        );
        assert!(!visible_transaction_detail_actions(true, true, false, true)
            .contains(&TransactionDetailAction::MarkTransfer));
        assert!(
            !visible_transaction_detail_actions(false, false, true, true)
                .contains(&TransactionDetailAction::FindPattern)
        );
    }

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
            !visible_transaction_detail_actions(true, true, false, false)
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
        assert!(visible_transaction_detail_actions(false, true, true, true)
            .contains(&TransactionDetailAction::MoveBudgetCode));
    }

    #[test]
    fn simple_mode_blocks_targets_that_do_not_match_transaction_amount_direction() {
        let expense_transaction = tx(-100, "FOOD", "Groceries");
        let income_transaction = tx(100, "OTHER", "Other");
        let budgets = vec![budget("FOOD", BudgetDirection::Expense)];
        let income_target = TransactionBudgetTarget {
            code: "SALARY".to_string(),
            category: "Salary".to_string(),
            direction: BudgetDirection::Income,
        };
        let expense_target = TransactionBudgetTarget {
            code: "FOOD".to_string(),
            category: "Groceries".to_string(),
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
    fn transaction_budget_direction_change_warns_for_inc_other_to_other() {
        let transaction = tx(100, "INC-OTHER", "Other income");
        assert_eq!(
            transaction_budget_direction_change(&transaction, &[], "OTHER", "Other", "expense"),
            Some(BudgetDirectionChange {
                item: "INC-OTHER -> OTHER".to_string(),
            })
        );
    }

    #[test]
    fn transaction_budget_direction_change_warns_for_other_to_inc_other() {
        let transaction = tx(-100, "OTHER", "Other");
        assert_eq!(
            transaction_budget_direction_change(
                &transaction,
                &[],
                "INC-OTHER",
                "Other income",
                "income",
            ),
            Some(BudgetDirectionChange {
                item: "OTHER -> INC-OTHER".to_string(),
            })
        );
    }

    #[test]
    fn transaction_budget_direction_change_uses_configured_budget_directions() {
        let transaction = tx(-100, "FOOD", "Groceries");
        assert_eq!(
            transaction_budget_direction_change(
                &transaction,
                &[
                    budget("FOOD", BudgetDirection::Expense),
                    budget("SALARY", BudgetDirection::Income),
                ],
                "SALARY",
                "Salary",
                "income",
            ),
            Some(BudgetDirectionChange {
                item: "FOOD -> SALARY".to_string(),
            })
        );
    }

    #[test]
    fn transaction_budget_direction_change_ignores_same_direction_and_transfers() {
        let transaction = tx(-100, "FOOD", "Groceries");
        assert!(transaction_budget_direction_change(
            &transaction,
            &[budget("FOOD", BudgetDirection::Expense)],
            "OTHER",
            "Other",
            "expense",
        )
        .is_none());
        assert!(transaction_budget_direction_change(
            &transaction,
            &[budget("FOOD", BudgetDirection::Expense)],
            "TRANSFER",
            "Transfers",
            "transfer",
        )
        .is_none());
    }
}
