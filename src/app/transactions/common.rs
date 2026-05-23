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

    let shell = build_action_dialog_shell(
        "Create Rule",
        "Create a categorization rule from this transaction.",
        "Save",
        "document-save-symbolic",
        "Save rule",
        "Search rule fields",
    );
    shell.set_form_only();
    let save_button = shell.submit_button.clone();
    register_config_widget(ui_handles, &save_button);

    let page = ui::page_box();
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
    ui::focus_button_after_combo_selections(
        &save_button,
        &[&field, &search, &category, &budget_code, &direction],
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
    shell.add_form_page(&ui::action_dialog_scroll(&page));

    let dialog = adw::Dialog::builder()
        .title(tr("Create Rule"))
        .content_width(680)
        .default_widget(&save_button)
        .child(&shell.root)
        .build();

    let ui_for_save = Rc::clone(ui_handles);
    let dialog_for_save = dialog.clone();
    ui::connect_button_activation(&save_button, move |button| {
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
    let dialog_subtitle = if advanced_features {
        "Move this transaction to another category or budget code."
    } else {
        "Move this transaction to another category."
    };
    let match_name = truncate(&initial.search, 60);
    let header_title = transaction_budget_move_dialog_title(&match_name, dialog_title);
    let submit_tooltip = if advanced_features {
        "Save budget code move"
    } else {
        "Save category move"
    };
    let shell = build_action_dialog_shell(
        &header_title,
        dialog_subtitle,
        "Save",
        "document-save-symbolic",
        submit_tooltip,
        "Search categories",
    );
    register_config_widget(ui_handles, &shell.submit_button);
    shell.submit_button.set_sensitive(false);

    let match_summary = transaction_budget_move_match_summary(&initial);
    let budget_targets =
        transaction_budget_move_targets(tx, &state.borrow().budgets, advanced_features);

    let list_page = adw::PreferencesPage::new();
    let choices_group = adw::PreferencesGroup::new();
    let selected_target: Rc<RefCell<Option<TransactionBudgetTarget>>> = Rc::new(RefCell::new(None));
    let mut row_widgets = Vec::new();
    let mut search_rows = Vec::<SearchableActionRow>::new();

    for target in &budget_targets {
        let title = transaction_budget_target_title(target, advanced_features);
        let subtitle = transaction_budget_target_subtitle(target, advanced_features);
        let row = adw::ActionRow::builder()
            .title(title.as_str())
            .subtitle(subtitle.as_str())
            .build();
        row.set_title_lines(1);
        row.set_subtitle_lines(2);
        row.set_activatable(true);
        row.set_focusable(true);

        let check = gtk::Image::from_icon_name("object-select-symbolic");
        check.add_css_class("accent");
        check.set_visible(false);
        row.add_suffix(&check);

        choices_group.add(&row);
        search_rows.push(searchable_action_row(
            &row,
            &title,
            &subtitle,
            &transaction_budget_target_search_keywords(target, advanced_features, &match_summary),
        ));
        row_widgets.push(TransactionBudgetTargetRow {
            row,
            check,
            target: target.clone(),
        });
    }

    let empty_search = ui::wrapped_label(&tr("No matching categories."));
    empty_search.add_css_class("dim-label");
    empty_search.set_visible(false);
    choices_group.add(&empty_search);
    list_page.add(&choices_group);

    if transaction_budget_more_options_visible(advanced_features) {
        let advanced_group = adw::PreferencesGroup::new();
        let more_row = adw::ActionRow::builder()
            .title(tr("More Options"))
            .subtitle(tr(
                "Edit the category, budget code, direction, and confirmation details.",
            ))
            .build();
        more_row.set_activatable(true);
        more_row.add_prefix(&gtk::Image::from_icon_name("emblem-system-symbolic"));
        more_row.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));
        advanced_group.add(&more_row);
        list_page.add(&advanced_group);

        let shell_for_more = shell.page_handle();
        more_row.connect_activated(move |_| {
            shell_for_more.set_form_page();
        });
    }

    let status = ui::wrapped_label("");
    status.add_css_class("dim-label");
    status.set_visible(false);
    choices_group.add(&status);

    let rows = Rc::new(row_widgets);
    for index in 0..rows.len() {
        let row = rows[index].row.clone();
        let rows_for_click = Rc::clone(&rows);
        let selected_for_click = Rc::clone(&selected_target);
        let save_for_click = shell.submit_button.clone();
        let tx_for_click = tx.clone();
        let click = gtk::GestureClick::new();
        click.set_button(0);
        click.connect_pressed(move |_, n_press, _, _| {
            select_transaction_budget_target_row(
                rows_for_click.as_ref(),
                &selected_for_click,
                &save_for_click,
                &tx_for_click,
                advanced_features,
                index,
            );
            if n_press >= 2 && save_for_click.is_sensitive() {
                save_for_click.emit_clicked();
            }
        });
        row.add_controller(click);

        let rows_for_key = Rc::clone(&rows);
        let selected_for_key = Rc::clone(&selected_target);
        let save_for_key = shell.submit_button.clone();
        let tx_for_key = tx.clone();
        let key = gtk::EventControllerKey::new();
        key.connect_key_pressed(move |_, key, _, _| {
            let activates = matches!(
                key,
                gtk::gdk::Key::Return | gtk::gdk::Key::KP_Enter | gtk::gdk::Key::space
            );
            if !activates {
                return gtk::glib::Propagation::Proceed;
            }
            select_transaction_budget_target_row(
                rows_for_key.as_ref(),
                &selected_for_key,
                &save_for_key,
                &tx_for_key,
                advanced_features,
                index,
            );
            if save_for_key.is_sensitive() {
                save_for_key.emit_clicked();
            }
            gtk::glib::Propagation::Stop
        });
        row.add_controller(key);
    }

    let list_max_height = transaction_budget_move_list_max_height(&ui_handles.window);
    let list_min_height = transaction_budget_move_list_min_height(rows.len()).min(list_max_height);
    shell.add_list_page(&ui::action_dialog_scroll_with_limits(
        &list_page,
        list_min_height,
        list_max_height,
    ));
    if let Some(selected_index) = rows
        .iter()
        .position(|row| transaction_budget_target_is_current(tx, &row.target, advanced_features))
    {
        select_transaction_budget_target_row(
            rows.as_ref(),
            &selected_target,
            &shell.submit_button,
            tx,
            advanced_features,
            selected_index,
        );
    }
    connect_action_search(
        &shell.search_entry,
        search_rows,
        Some(empty_search.clone().upcast::<gtk::Widget>()),
    );

    let mut form_category: Option<gtk::ComboBoxText> = None;
    let mut form_budget_code: Option<gtk::ComboBoxText> = None;
    let mut form_direction: Option<gtk::ComboBoxText> = None;
    let mut form_status: Option<gtk::Label> = None;

    if advanced_features {
        let form_page = ui::page_box();
        let form = ui::form_box();
        ui::add_labeled_stacked(&form, "Rule match", &ui::wrapped_label(&match_summary));

        let category = category_combo(&state.borrow(), &initial.category);
        let budget_code = budget_code_combo(&state.borrow(), &initial.budget_code);
        let direction = ui::combo_from_options(
            &[
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
        connect_transaction_budget_move_form_save_sensitivity(
            &shell.stack,
            &shell.submit_button,
            tx,
            &selected_target,
            &initial,
            &category,
            &budget_code,
            &direction,
            advanced_features,
        );
        ui::focus_button_after_combo_selections(
            &shell.submit_button,
            &[&category, &budget_code, &direction],
        );
        ui::add_labeled_stacked(&form, "Category", &category);
        ui::add_labeled_stacked(&form, "Budget code", &budget_code);
        ui::add_labeled_stacked(&form, "Direction", &direction);
        form_page.append(&form);

        let advanced_status = ui::wrapped_label(&tr(
            "Save adds a categorization rule to the processing queue. The original bank CSV is not changed.",
        ));
        advanced_status.add_css_class("dim-label");
        form_page.append(&advanced_status);
        shell.add_form_page(&ui::action_dialog_scroll(&form_page));

        form_category = Some(category);
        form_budget_code = Some(budget_code);
        form_direction = Some(direction);
        form_status = Some(advanced_status);
    }

    shell.set_list_page();

    let dialog = adw::Dialog::builder()
        .title(tr(dialog_title))
        .content_width(680)
        .default_widget(&shell.submit_button)
        .child(&shell.root)
        .build();
    ui::connect_search_shortcut(&shell.root, &shell.search_bar, &shell.search_entry);

    let shell_for_back = shell.page_handle();
    shell.back_button.connect_clicked(move |_| {
        shell_for_back.set_list_page();
    });

    let state_for_save = Rc::clone(state);
    let ui_for_save = Rc::clone(ui_handles);
    let dialog_for_save = dialog.clone();
    let tx_for_save = tx.clone();
    let initial_for_save = initial.clone();
    let selected_for_save = Rc::clone(&selected_target);
    let stack_for_save = shell.stack.clone();
    let list_status_for_save = status.clone();
    let form_status_for_save = form_status.clone();
    ui::connect_button_activation(&shell.submit_button, move |button| {
        let using_form = stack_for_save.visible_child_name().as_deref() == Some("form");
        let active_status = if using_form {
            form_status_for_save
                .as_ref()
                .unwrap_or(&list_status_for_save)
                .clone()
        } else {
            list_status_for_save.clone()
        };

        let move_values = if using_form {
            let Some(category) = form_category.as_ref() else {
                set_action_status(&active_status, "Open More Options first.");
                return;
            };
            let Some(budget_code) = form_budget_code.as_ref() else {
                set_action_status(&active_status, "Open More Options first.");
                return;
            };
            let Some(direction) = form_direction.as_ref() else {
                set_action_status(&active_status, "Open More Options first.");
                return;
            };
            let budget_code_text = ui::combo_text(budget_code);
            if budget_code_text.is_empty() {
                set_action_status(&active_status, "Enter a budget code first.");
                budget_code.grab_focus();
                return;
            }
            let category_text = ui::combo_text(category);
            if category_text.is_empty() {
                set_action_status(&active_status, "Choose a budget code first.");
                budget_code.grab_focus();
                return;
            }
            let direction_text = ui::combo_active_id(direction);
            if !transaction_budget_move_form_values_changed(
                &initial_for_save,
                &category_text,
                &budget_code_text,
                &direction_text,
            ) {
                set_action_status(&active_status, "Choose a different category first.");
                return;
            }
            TransactionBudgetMoveValues {
                category: category_text,
                budget_code: budget_code_text,
                direction: direction_text,
            }
        } else {
            let Some(target) = selected_for_save.borrow().clone() else {
                set_action_status(&active_status, "Choose a category first.");
                return;
            };
            if !transaction_budget_target_is_changed(&tx_for_save, &target, advanced_features) {
                set_action_status(&active_status, "Choose a different category first.");
                return;
            }
            if !transaction_budget_target_allowed(
                &tx_for_save,
                &state_for_save.borrow().budgets,
                &target,
                advanced_features,
            ) {
                set_action_status(
                    &active_status,
                    "This move changes direction. Enable Advanced Features to continue.",
                );
                return;
            }
            TransactionBudgetMoveValues {
                category: target.category,
                budget_code: target.code,
                direction: target.direction.as_str().to_string(),
            }
        };

        let direction_changes = if advanced_features {
            transaction_budget_direction_change(
                &tx_for_save,
                &state_for_save.borrow().budgets,
                &move_values.budget_code,
                &move_values.category,
                &move_values.direction,
            )
            .into_iter()
            .collect()
        } else {
            Vec::new()
        };
        let rule = transaction_budget_move_rule(&initial_for_save, move_values, advanced_features);

        let button = button.clone();
        let status = active_status.clone();
        let ui_for_save = Rc::clone(&ui_for_save);
        let dialog_for_confirm = dialog_for_save.clone();
        let dialog_for_save = dialog_for_save.clone();
        confirm_budget_direction_changes(&dialog_for_confirm, direction_changes, move || {
            enqueue_rule_operation(&ui_for_save, rule, true, OperationSource::ChangeBudgetCode);
            button.set_sensitive(false);
            set_action_status(
                &status,
                if advanced_features {
                    "Budget code move added to processing queue."
                } else {
                    "Category move added to processing queue."
                },
            );
            dialog_for_save.close();
        });
    });

    dialog.present(Some(&ui_handles.window));
}

fn transaction_budget_move_dialog_title(match_name: &str, fallback_title: &str) -> String {
    if match_name.trim().is_empty() {
        tr(fallback_title)
    } else {
        match_name.to_string()
    }
}

fn transaction_budget_move_match_summary(initial: &EditableRule) -> String {
    trf(
        "Matching by {field}: {value}",
        &[
            ("field", tr(rule_field_label(&initial.field))),
            ("value", truncate(&initial.search, 80)),
        ],
    )
}

fn transaction_budget_move_list_min_height(row_count: usize) -> i32 {
    let visible_rows = row_count.clamp(3, 7) as i32;
    72 + visible_rows * 54
}

fn transaction_budget_move_list_max_height(window: &impl IsA<gtk::Widget>) -> i32 {
    transaction_budget_move_list_max_height_for_window(window.as_ref().allocated_height())
}

fn transaction_budget_move_list_max_height_for_window(window_height: i32) -> i32 {
    if window_height <= 0 {
        return 620;
    }
    window_height.saturating_sub(96).clamp(180, 900)
}

fn set_action_status(status: &gtk::Label, message: &str) {
    status.set_text(&tr(message));
    status.set_visible(true);
}

fn connect_transaction_budget_move_form_save_sensitivity(
    stack: &gtk::Stack,
    save_button: &gtk::Button,
    tx: &Transaction,
    selected_target: &Rc<RefCell<Option<TransactionBudgetTarget>>>,
    initial: &EditableRule,
    category: &gtk::ComboBoxText,
    budget_code: &gtk::ComboBoxText,
    direction: &gtk::ComboBoxText,
    advanced_features: bool,
) {
    let update: Rc<dyn Fn()> = Rc::new({
        let stack = stack.clone();
        let save_button = save_button.clone();
        let tx = tx.clone();
        let selected_target = Rc::clone(selected_target);
        let initial = initial.clone();
        let category = category.clone();
        let budget_code = budget_code.clone();
        let direction = direction.clone();
        move || {
            let using_form = stack.visible_child_name().as_deref() == Some("form");
            if using_form {
                save_button.set_sensitive(transaction_budget_move_form_is_changed(
                    &initial,
                    &category,
                    &budget_code,
                    &direction,
                ));
            } else {
                save_button.set_sensitive(selected_target.borrow().as_ref().is_some_and(
                    |target| transaction_budget_target_is_changed(&tx, target, advanced_features),
                ));
            }
        }
    });

    for combo in [category, budget_code, direction] {
        let update_for_change = Rc::clone(&update);
        combo.connect_changed(move |_| update_for_change());
    }

    let update_for_page = Rc::clone(&update);
    stack.connect_visible_child_name_notify(move |_| update_for_page());
    update();
}

fn transaction_budget_move_form_is_changed(
    initial: &EditableRule,
    category: &gtk::ComboBoxText,
    budget_code: &gtk::ComboBoxText,
    direction: &gtk::ComboBoxText,
) -> bool {
    transaction_budget_move_form_values_changed(
        initial,
        &ui::combo_text(category),
        &ui::combo_text(budget_code),
        &ui::combo_active_id(direction),
    )
}

fn transaction_budget_move_form_values_changed(
    initial: &EditableRule,
    category: &str,
    budget_code: &str,
    direction: &str,
) -> bool {
    !same_form_text(category, &initial.category)
        || !same_form_text(budget_code, &initial.budget_code)
        || !same_form_text(direction, &initial.direction)
}

fn same_form_text(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

fn select_transaction_budget_target_row(
    rows: &[TransactionBudgetTargetRow],
    selected_target: &Rc<RefCell<Option<TransactionBudgetTarget>>>,
    save_button: &gtk::Button,
    tx: &Transaction,
    advanced_features: bool,
    selected_index: usize,
) {
    let mut selected = None;
    for (index, row) in rows.iter().enumerate() {
        let row_selected = index == selected_index;
        row.check.set_visible(row_selected);
        if row_selected {
            selected = Some(row.target.clone());
        }
    }
    save_button.set_sensitive(
        selected.as_ref().is_some_and(|target| {
            transaction_budget_target_is_changed(tx, target, advanced_features)
        }),
    );
    *selected_target.borrow_mut() = selected;
}

#[derive(Clone)]
struct TransactionBudgetTargetRow {
    row: adw::ActionRow,
    check: gtk::Image,
    target: TransactionBudgetTarget,
}

#[derive(Clone)]
struct TransactionBudgetMoveValues {
    category: String,
    budget_code: String,
    direction: String,
}

fn transaction_budget_move_rule(
    initial: &EditableRule,
    values: TransactionBudgetMoveValues,
    advanced_features: bool,
) -> EditableRule {
    EditableRule {
        priority: 140,
        active: true,
        field: initial.field.clone(),
        search: initial.search.clone(),
        is_regex: initial.is_regex,
        category: values.category,
        budget_code: values.budget_code,
        direction: values.direction,
        amount_min: initial.amount_min.clone(),
        amount_max: initial.amount_max.clone(),
        notes: tr(if advanced_features {
            "Generated from transaction budget code change."
        } else {
            "Generated from transaction category change."
        }),
    }
}

fn transaction_budget_target_title(
    target: &TransactionBudgetTarget,
    advanced_features: bool,
) -> String {
    if !target.category.trim().is_empty() {
        target.category.clone()
    } else if advanced_features {
        target.code.clone()
    } else {
        tr("Uncategorized")
    }
}

fn transaction_budget_target_subtitle(
    target: &TransactionBudgetTarget,
    advanced_features: bool,
) -> String {
    let description = target.description.trim();
    if advanced_features {
        if description.is_empty() {
            target.code.clone()
        } else {
            trf(
                "{code} · {description}",
                &[
                    ("code", target.code.clone()),
                    ("description", description.to_string()),
                ],
            )
        }
    } else {
        description.to_string()
    }
}

fn transaction_budget_direction_label(direction: BudgetDirection) -> String {
    tr(match direction {
        BudgetDirection::Income => "Income",
        BudgetDirection::Transfer => "Transfers",
        BudgetDirection::Expense => "Expenses",
    })
}

fn transaction_budget_target_search_keywords(
    target: &TransactionBudgetTarget,
    advanced_features: bool,
    match_summary: &str,
) -> Vec<String> {
    let mut keywords = vec![
        target.category.clone(),
        target.description.clone(),
        transaction_budget_direction_label(target.direction),
        match_summary.to_string(),
    ];
    if advanced_features {
        keywords.push(target.code.clone());
    }
    keywords
}

fn transaction_budget_target_is_current(
    tx: &Transaction,
    target: &TransactionBudgetTarget,
    advanced_features: bool,
) -> bool {
    if !advanced_features
        && !tx.category.trim().is_empty()
        && target
            .category
            .trim()
            .eq_ignore_ascii_case(tx.category.trim())
    {
        return true;
    }

    target
        .code
        .trim()
        .eq_ignore_ascii_case(tx.budget_code.trim())
}

fn transaction_budget_target_is_changed(
    tx: &Transaction,
    target: &TransactionBudgetTarget,
    advanced_features: bool,
) -> bool {
    !transaction_budget_target_is_current(tx, target, advanced_features)
}

fn transaction_budget_more_options_visible(advanced_features: bool) -> bool {
    advanced_features
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct TransactionBudgetTarget {
    code: String,
    category: String,
    description: String,
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
                description: budget.notes.trim().to_string(),
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
        .any(|target| transaction_budget_target_is_changed(tx, target, advanced_features))
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
    fn budget_move_list_subtitle_uses_budget_description() {
        let target = TransactionBudgetTarget {
            code: "SHOP".to_string(),
            category: "Groceries".to_string(),
            description: "Food and household shopping".to_string(),
            direction: BudgetDirection::Expense,
        };

        assert_eq!(
            transaction_budget_target_subtitle(&target, false),
            "Food and household shopping"
        );
        assert_eq!(
            transaction_budget_target_subtitle(&target, true),
            trf(
                "{code} · {description}",
                &[
                    ("code", "SHOP".to_string()),
                    ("description", "Food and household shopping".to_string()),
                ],
            )
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
        let advanced_keywords = transaction_budget_target_search_keywords(
            &target,
            true,
            "Matching by Counterparty: Store",
        )
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
