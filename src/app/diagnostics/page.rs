use super::*;
use std::collections::HashMap;

pub(in crate::app) fn render_diagnostics_page(
    data: &AppData,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    ui::clear_box(&ui_handles.debug);
    let search = active_search(ui_handles.as_ref());
    let subtitle = search
        .as_ref()
        .map(|filter| {
            trf(
                "Filter “{query}” searches import settings, CSV files, and warnings.",
                &[("query", filter.raw.clone())],
            )
        })
        .unwrap_or_else(|| {
            "Import quality, detected fields, and warnings. Everything is local and copyable here."
                .to_string()
        });
    append_page_header(
        &ui_handles.debug,
        ui_handles.as_ref(),
        "Diagnostics",
        &subtitle,
        summary::render_debug(data),
        &data.transactions,
    );

    let rows_seen: usize = data.reports.iter().map(|report| report.rows_seen).sum();
    let rows_imported: usize = data.reports.iter().map(|report| report.rows_imported).sum();
    let rows_skipped: usize = data.reports.iter().map(|report| report.rows_skipped).sum();
    let unconfigured_budget_count =
        analytics::unconfigured_expense_budget_count(&data.transactions, &data.budgets);
    let other_category_count = analytics::other_category_count(&data.transactions);
    let csv_status = if data.reports.is_empty() {
        tr("No CSV files opened. Choose CSV files or drop bank files onto the window.")
    } else {
        let names = data
            .reports
            .iter()
            .filter_map(|report| report.source.file_name())
            .map(|name| name.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        trf(
            "{count} stored CSV file(s): {files}",
            &[
                ("count", data.reports.len().to_string()),
                ("files", truncate(&names, 160)),
            ],
        )
    };
    let rows_status = trf(
        "{seen} rows seen. {imported} imported, {skipped} skipped.",
        &[
            ("seen", rows_seen.to_string()),
            ("imported", rows_imported.to_string()),
            ("skipped", rows_skipped.to_string()),
        ],
    );
    let mut has_search_results = false;
    let ui_for_csv = Rc::clone(ui_handles);
    let ui_for_rows = Rc::clone(ui_handles);
    let state_for_imported = Rc::clone(state);
    let ui_for_imported = Rc::clone(ui_handles);
    let state_for_unconfigured = Rc::clone(state);
    let ui_for_unconfigured = Rc::clone(ui_handles);
    let state_for_other = Rc::clone(state);
    let ui_for_other = Rc::clone(ui_handles);
    let app_for_duplicates = ui_handles.window.application();
    ui_handles.debug.append(&ui::metric_grid(
        vec![
            ui::activatable_metric_card(
                "CSV files",
                &data.reports.len().to_string(),
                "Stored",
                move || show_status(&ui_for_csv, &csv_status),
            ),
            ui::activatable_metric_card(
                "Rows seen",
                &rows_seen.to_string(),
                "For import checks",
                move || show_status(&ui_for_rows, &rows_status),
            ),
            ui::activatable_metric_card(
                "Imported",
                &rows_imported.to_string(),
                &trf("{count} skipped", &[("count", rows_skipped.to_string())]),
                move || {
                    show_transactions_filter(
                        &state_for_imported,
                        &ui_for_imported,
                        TransactionFilter::all(),
                    );
                },
            ),
            duplicate_filtering_card(data, move || {
                if let Some(action) = app_for_duplicates
                    .as_ref()
                    .and_then(|app| app.lookup_action("dedupe-enabled"))
                {
                    let enabled = action
                        .state()
                        .and_then(|state| state.get::<bool>())
                        .unwrap_or(true);
                    action.change_state(&(!enabled).to_variant());
                }
            }),
            ui::activatable_metric_card(
                "Unconfigured budgets",
                &unconfigured_budget_count.to_string(),
                TransactionFilter::UnconfiguredBudgets.description(),
                move || {
                    show_transactions_filter(
                        &state_for_unconfigured,
                        &ui_for_unconfigured,
                        TransactionFilter::UnconfiguredBudgets,
                    );
                },
            ),
            ui::activatable_metric_card(
                "Other categories",
                &other_category_count.to_string(),
                TransactionFilter::OtherCategories.description(),
                move || {
                    show_transactions_filter(
                        &state_for_other,
                        &ui_for_other,
                        TransactionFilter::OtherCategories,
                    );
                },
            ),
        ],
        4,
    ));

    if append_orphaned_config_section(search.as_ref(), state, ui_handles) {
        has_search_results = true;
    }

    if transaction_patterns_section_visible(
        search.as_ref(),
        smart_pattern_detection_enabled(ui_handles.show_predictions.get()),
    ) {
        has_search_results = true;
        append_transaction_patterns_section_async(
            data,
            search.clone(),
            selected_year(data, ui_handles.as_ref()),
            state,
            ui_handles,
        );
    }

    if data.reports.is_empty() && search.is_none() {
        ui_handles.debug.append(&empty_page(
            "dialog-information-symbolic",
            "No CSV files opened",
            "Choose CSV files or drop bank files onto the window to see import diagnostics.",
        ));
    } else {
        let reports = data
            .reports
            .iter()
            .filter(|report| {
                search
                    .as_ref()
                    .map(|filter| import_report_matches(report, filter))
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>();
        if !reports.is_empty() {
            has_search_results = true;
            let reload_all_button = ui::plain_text_icon_button(
                "view-refresh-symbolic",
                "Reload All",
                "Force reload all CSV files",
            );
            reload_all_button.set_action_name(Some("app.reload-all"));
            ui_handles.debug.append(&ui::section_title_with_action(
                "CSV files",
                "These are app copies of files you opened through the portal or drag-and-drop. Unloading only removes the stored copy.",
                &reload_all_button,
            ));
            let files = gtk::Box::new(gtk::Orientation::Vertical, 8);
            for report in reports {
                files.append(&diagnostic_file_card(report, state, ui_handles));
            }
            ui_handles.debug.append(&files);
        }
    }

    let warnings = data
        .warnings
        .iter()
        .filter(|warning| {
            search
                .as_ref()
                .map(|filter| filter.matches(warning))
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();
    if !warnings.is_empty() {
        has_search_results = true;
        ui_handles.debug.append(&ui::section_title(
            "Warnings",
            "You can select these messages or include them through Copy Page.",
        ));
        let warnings_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        for warning in warnings {
            warnings_box.append(&ui::text_card(warning));
        }
        ui_handles.debug.append(&warnings_box);
    }

    if search.is_some() && !has_search_results {
        ui_handles.debug.append(&search_empty_page(
            "No diagnostic results",
            "No CSV files or warnings match this search term.",
        ));
    }
}

fn append_orphaned_config_section(
    search: Option<&SearchFilter>,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> bool {
    let orphaned_rules = match data::orphaned_rules() {
        Ok(rules) => rules,
        Err(error) => {
            if search
                .map(|filter| filter.matches("orphaned configuration missing budget codes rules"))
                .unwrap_or(true)
            {
                ui_handles.debug.append(&ui::section_title(
                    "Orphaned Configuration",
                    "Rules that point to missing budget codes can create ghost categories or budget codes.",
                ));
                ui_handles.debug.append(&ui::text_card(&trf(
                    "Could not inspect configuration: {error}",
                    &[("error", format!("{error:#}"))],
                )));
                return true;
            }
            return false;
        }
    };
    let section_matches = search
        .map(|filter| {
            filter.matches(
                "orphaned configuration missing budget codes ghost categories rules cleanup",
            )
        })
        .unwrap_or(!orphaned_rules.is_empty());
    let visible_rules = orphaned_rules
        .iter()
        .filter(|rule| {
            search
                .map(|filter| orphaned_rule_matches(rule, filter))
                .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();
    if !section_matches && visible_rules.is_empty() {
        return false;
    }

    let remove_button = ui::plain_text_icon_button(
        "user-trash-symbolic",
        "Remove Orphaned Rules",
        "Remove rules that point to missing budget codes",
    );
    remove_button.add_css_class("destructive-action");
    remove_button.set_sensitive(!orphaned_rules.is_empty());
    register_config_widget(ui_handles, &remove_button);
    let state_for_remove = Rc::clone(state);
    let ui_for_remove = Rc::clone(ui_handles);
    remove_button.connect_clicked(move |button| {
        remove_orphaned_config_rules(&state_for_remove, &ui_for_remove, button);
    });

    ui_handles.debug.append(&ui::section_title_with_action(
        "Orphaned Configuration",
        "Rules that point to missing budget codes can create ghost categories or budget codes.",
        &remove_button,
    ));

    let rows_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    if visible_rules.is_empty() {
        rows_box.append(&ui::text_card(&tr("No orphaned configuration found.")));
    } else {
        let show_all = ui_handles.show_all.get() || search.is_some();
        let preview_limit = if show_all {
            usize::MAX
        } else {
            CATEGORY_PREVIEW_LIMIT
        };
        append_orphaned_rule_rows(&rows_box, visible_rules.iter().take(preview_limit));
        if !show_all && visible_rules.len() > preview_limit {
            let more_button =
                ui::plain_text_icon_button("view-more-symbolic", "More", "Show all orphaned rules");
            let rows_box_for_more = rows_box.clone();
            more_button.connect_clicked(move |button| {
                ui::clear_box(&rows_box_for_more);
                append_orphaned_rule_rows(&rows_box_for_more, visible_rules.iter());
                button.set_visible(false);
            });
            rows_box.append(&more_button);
        }
    }
    ui_handles.debug.append(&rows_box);
    true
}

fn append_orphaned_rule_rows<'a>(
    container: &gtk::Box,
    rules: impl IntoIterator<Item = &'a data::OrphanedRule>,
) {
    for orphan in rules {
        container.append(&ui::text_card(&orphaned_rule_text(orphan)));
    }
}

fn orphaned_rule_matches(orphan: &data::OrphanedRule, filter: &SearchFilter) -> bool {
    let rule = &orphan.rule;
    filter.matches(&format!(
        "{} {} {} {} {} {} orphan missing budget code",
        orphan.budget_code, rule.category, rule.search, rule.field, rule.direction, rule.notes,
    ))
}

fn orphaned_rule_text(orphan: &data::OrphanedRule) -> String {
    trf(
        "Rule “{search}” uses missing budget code “{code}” in category “{category}”.",
        &[
            ("search", truncate(&orphan.rule.search, 80)),
            ("code", orphan.budget_code.clone()),
            ("category", orphan.rule.category.clone()),
        ],
    )
}

fn transaction_patterns_section_visible(
    search: Option<&SearchFilter>,
    smart_patterns_enabled: bool,
) -> bool {
    smart_patterns_enabled && transaction_patterns_section_matches(search)
}

fn transaction_patterns_section_matches(search: Option<&SearchFilter>) -> bool {
    search
        .map(|filter| {
            filter.matches(
                "Transaction Patterns repeating transactions refunds fully offsetting groups tags categorization rules",
            )
        })
        .unwrap_or(true)
}

fn remove_orphaned_config_rules(
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    button: &gtk::Button,
) {
    if !try_begin_config_operation(ui_handles, "Another edit or save is already running.") {
        return;
    }

    let button = button.clone();
    let mode = state.borrow().dedupe_mode;
    let auto_clean_config = ui_handles.preferences.auto_clean_config();
    let scope = current_transaction_load_scope(&state.borrow(), ui_handles.as_ref());
    let state_for_remove = Rc::clone(state);
    let ui_for_remove = Rc::clone(ui_handles);
    button.set_sensitive(false);
    show_status(ui_handles, "Removing orphaned rules...");
    begin_background_operation(ui_handles.as_ref());

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            let removed = data::remove_orphaned_rules()?;
            let new_data = data::load_app_data_with_config_cleanup(mode, auto_clean_config, scope)?;
            anyhow::Ok((removed, new_data))
        });

        match task.await {
            Ok(Ok((removed, new_data))) => {
                *state_for_remove.borrow_mut() = new_data;
                render_views(
                    &state_for_remove.borrow(),
                    &ui_for_remove,
                    &state_for_remove,
                );
                show_status(
                    &ui_for_remove,
                    &trf(
                        "{count} orphaned rule(s) removed.",
                        &[("count", removed.to_string())],
                    ),
                );
            }
            Ok(Err(error)) => show_status(
                &ui_for_remove,
                &trf(
                    "Could not remove orphaned rules: {error}",
                    &[("error", format!("{error:#}"))],
                ),
            ),
            Err(_) => show_status(
                &ui_for_remove,
                "Removing orphaned rules canceled: the background task stopped unexpectedly.",
            ),
        }
        finish_background_operation(ui_for_remove.as_ref());
        finish_config_operation(&ui_for_remove);
    });
}

fn append_transaction_patterns_section_async(
    data: &AppData,
    search: Option<SearchFilter>,
    selected_year: Option<i32>,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let subtitle = selected_year
        .map(|year| {
            trf(
                "Repeating payments, possible transfers, refunds, and offsetting groups detected in {year}.",
                &[("year", year.to_string())],
            )
        })
        .unwrap_or_else(|| {
            tr("Repeating payments, possible transfers, refunds, and fully offsetting groups detected from imported transactions.")
        });
    ui_handles
        .debug
        .append(&ui::section_title("Transaction Patterns", &subtitle));
    if let Some(year) = selected_year {
        ui_handles.debug.append(&year_selector_row(
            &data.available_years,
            year,
            ui_handles,
            state,
        ));
    }
    let patterns_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    append_transaction_patterns_loading(&patterns_box);
    ui_handles.debug.append(&patterns_box);

    let data = data.clone();
    let mode = data.dedupe_mode;
    let needs_all_reload = transaction_patterns_need_all_reload(data.loaded_scope);
    let fake_transactions = ui_handles.fake_transactions.list();
    let show_all = ui_handles.show_all.get() || search.is_some();
    let hide_canceled = ui_handles.hide_canceled_transactions.get();
    let generation = ui_handles.render_generation.get();
    let state_for_patterns = Rc::clone(state);
    let ui_for_patterns = Rc::clone(ui_handles);
    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            let data = if needs_all_reload {
                let (data, _) =
                    data::load_app_data_read_only_aware(mode, false, TransactionLoadScope::All)?;
                data_with_fake_transactions(data, fake_transactions)
            } else {
                data
            };
            anyhow::Ok(transaction_patterns_render_data(
                data,
                search,
                show_all,
                hide_canceled,
                selected_year,
            ))
        });

        match task.await {
            Ok(Ok(render_data)) => {
                if ui_for_patterns.render_generation.get() != generation
                    || ui_for_patterns.stack.visible_child_name().as_deref() != Some("debug")
                {
                    return;
                }
                ui::clear_box(&patterns_box);
                append_transaction_patterns_result(
                    &patterns_box,
                    render_data,
                    &state_for_patterns,
                    &ui_for_patterns,
                );
            }
            Ok(Err(error)) => {
                if ui_for_patterns.render_generation.get() == generation {
                    ui::clear_box(&patterns_box);
                    patterns_box.append(&ui::text_card(&trf(
                        "Could not detect transaction patterns: {error}",
                        &[("error", format!("{error:#}"))],
                    )));
                }
            }
            Err(_) => {
                if ui_for_patterns.render_generation.get() == generation {
                    ui::clear_box(&patterns_box);
                    patterns_box.append(&ui::text_card(&tr(
                        "Transaction pattern detection stopped unexpectedly.",
                    )));
                }
            }
        }
    });
}

fn transaction_patterns_need_all_reload(scope: TransactionLoadScope) -> bool {
    !matches!(scope, TransactionLoadScope::All)
}

fn append_transaction_patterns_loading(container: &gtk::Box) {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 8);
    card.add_css_class("card");
    card.set_margin_top(4);
    card.set_margin_bottom(4);
    card.set_margin_start(4);
    card.set_margin_end(4);
    card.set_halign(gtk::Align::Fill);
    card.set_hexpand(true);

    let content = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let spinner = ui::loading_spinner();
    content.append(&spinner);

    let text = ui::wrapped_label(&tr("Detecting transaction patterns..."));
    text.add_css_class("dim-label");
    text.set_hexpand(true);
    content.append(&text);

    card.append(&content);
    container.append(&card);
}

fn append_transaction_patterns_result(
    patterns_box: &gtk::Box,
    render_data: TransactionPatternsRenderData,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    if render_data.hidden_count > 0 {
        let message = if render_data.hide_canceled {
            trf(
                "{count} refund or offsetting transactions are hidden from normal views.",
                &[("count", render_data.hidden_count.to_string())],
            )
        } else {
            trf(
                "{count} refund or offsetting transactions were detected. Enable Hide Refunded Transactions to exclude them from normal views.",
                &[("count", render_data.hidden_count.to_string())],
            )
        };
        patterns_box.append(&ui::text_card(&message));
    }

    if render_data.patterns.is_empty() {
        patterns_box.append(&ui::text_card(&tr(
            "No transaction patterns detected yet. Import more transactions or adjust tags and categorization rules.",
        )));
        return;
    }

    let rows_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    if render_data.preview_patterns.is_empty() {
        let message = if render_data.hidden_patterns.len() == render_data.patterns.len() {
            "All detected transaction patterns are hidden. Use Show Hidden to review them."
        } else {
            "All detected transaction patterns are already covered. Use More to review them."
        };
        rows_box.append(&ui::text_card(&tr(message)));
    } else {
        append_transaction_pattern_rows(
            &rows_box,
            &render_data.preview_patterns,
            state,
            ui_handles,
        );
    }
    patterns_box.append(&rows_box);

    let hidden_button = if render_data.hidden_patterns.is_empty() {
        None
    } else {
        let button = transaction_patterns_hidden_button(
            &rows_box,
            render_data.hidden_patterns,
            state,
            ui_handles,
        );
        if render_data.more_patterns.len() > render_data.preview_patterns.len() {
            button.set_visible(false);
        }
        Some(button)
    };

    if render_data.more_patterns.len() > render_data.preview_patterns.len() {
        append_transaction_patterns_more_button(
            patterns_box,
            &rows_box,
            render_data.more_patterns,
            hidden_button.as_ref(),
            state,
            ui_handles,
        );
    }
    if let Some(hidden_button) = hidden_button {
        patterns_box.append(&hidden_button);
    }
}

#[derive(Debug, Clone)]
struct TransactionPatternsRenderData {
    hidden_count: usize,
    hide_canceled: bool,
    patterns: Vec<(analytics::TransactionPattern, TransactionPatternInfo)>,
    preview_patterns: Vec<(analytics::TransactionPattern, TransactionPatternInfo)>,
    more_patterns: Vec<(analytics::TransactionPattern, TransactionPatternInfo)>,
    hidden_patterns: Vec<(analytics::TransactionPattern, TransactionPatternInfo)>,
}

fn transaction_patterns_render_data(
    mut data: AppData,
    search: Option<SearchFilter>,
    show_all_patterns: bool,
    hide_canceled: bool,
    selected_year: Option<i32>,
) -> TransactionPatternsRenderData {
    if let Some(year) = selected_year {
        data.transactions
            .retain(|transaction| transaction.year() == year);
    }

    let pattern_rules = data::load_editable_rules().unwrap_or_default();
    let hidden_pattern_keys = data::ignored_transaction_pattern_keys().unwrap_or_default();
    let pattern_analysis =
        analytics::transaction_pattern_analysis(&data.transactions, data.dedupe_mode.is_enabled());
    let hidden_count = pattern_analysis.hidden_canceled_transaction_count();
    let mut patterns = pattern_analysis
        .patterns
        .into_iter()
        .filter(|pattern| {
            search
                .as_ref()
                .map(|filter| transaction_pattern_matches(pattern, filter))
                .unwrap_or(true)
        })
        .map(|pattern| {
            let hidden =
                hidden_pattern_keys.contains(&analytics::transaction_pattern_key(&pattern));
            let info = transaction_pattern_info(&pattern, &data, &pattern_rules, hidden);
            (pattern, info)
        })
        .collect::<Vec<_>>();
    patterns.sort_by(transaction_pattern_render_order);
    let non_hidden_patterns = patterns
        .iter()
        .filter(|(_, info)| !info.hidden)
        .cloned()
        .collect::<Vec<_>>();
    let hidden_patterns = patterns
        .iter()
        .filter(|(_, info)| info.hidden)
        .cloned()
        .collect::<Vec<_>>();
    let preview_source = if show_all_patterns {
        non_hidden_patterns.clone()
    } else {
        non_hidden_patterns
            .iter()
            .filter(|(_, info)| info.needs_rule)
            .cloned()
            .collect::<Vec<_>>()
    };
    let preview_limit = if show_all_patterns {
        usize::MAX
    } else {
        CATEGORY_PREVIEW_LIMIT
    };
    let preview_patterns = preview_source
        .iter()
        .take(preview_limit)
        .cloned()
        .collect::<Vec<_>>();
    let more_patterns = if show_all_patterns {
        Vec::new()
    } else {
        non_hidden_patterns.clone()
    };

    TransactionPatternsRenderData {
        hidden_count,
        hide_canceled,
        patterns,
        preview_patterns,
        more_patterns,
        hidden_patterns,
    }
}

fn transaction_pattern_render_order(
    left: &(analytics::TransactionPattern, TransactionPatternInfo),
    right: &(analytics::TransactionPattern, TransactionPatternInfo),
) -> std::cmp::Ordering {
    right
        .1
        .needs_rule
        .cmp(&left.1.needs_rule)
        .then(right.1.affects_totals.cmp(&left.1.affects_totals))
        .then(right.0.last_date.cmp(&left.0.last_date))
        .then(right.0.count.cmp(&left.0.count))
        .then(left.0.label.cmp(&right.0.label))
}

fn append_transaction_pattern_rows(
    container: &gtk::Box,
    patterns: &[(analytics::TransactionPattern, TransactionPatternInfo)],
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    for (pattern, info) in patterns {
        let filter = TransactionFilter::Pattern(pattern.clone());
        let state_for_pattern = Rc::clone(state);
        let ui_for_pattern = Rc::clone(ui_handles);
        let card = transaction_pattern_card(pattern, &info.badges, move || {
            show_transactions_filter(&state_for_pattern, &ui_for_pattern, filter.clone());
        });
        container.append(&transaction_pattern_edit_row(
            card,
            pattern,
            info.hidden,
            state,
            ui_handles,
        ));
    }
}

fn append_transaction_patterns_more_button(
    section: &gtk::Box,
    rows_box: &gtk::Box,
    patterns: Vec<(analytics::TransactionPattern, TransactionPatternInfo)>,
    hidden_button: Option<&gtk::Button>,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let more_button = ui::plain_text_icon_button(
        "view-more-symbolic",
        "More",
        "Show all transaction patterns",
    );
    let rows_box = rows_box.clone();
    let state_for_more = Rc::clone(state);
    let ui_for_more = Rc::clone(ui_handles);
    let hidden_button = hidden_button.cloned();
    more_button.connect_clicked(move |button| {
        ui::clear_box(&rows_box);
        append_transaction_pattern_rows(&rows_box, &patterns, &state_for_more, &ui_for_more);
        if let Some(hidden_button) = &hidden_button {
            hidden_button.set_visible(true);
        }
        button.set_visible(false);
    });
    section.append(&more_button);
}

fn transaction_patterns_hidden_button(
    rows_box: &gtk::Box,
    patterns: Vec<(analytics::TransactionPattern, TransactionPatternInfo)>,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> gtk::Button {
    let button = ui::plain_text_icon_button(
        "view-reveal-symbolic",
        "Show Hidden",
        "Show hidden transaction patterns",
    );
    let rows_box = rows_box.clone();
    let state_for_hidden = Rc::clone(state);
    let ui_for_hidden = Rc::clone(ui_handles);
    button.connect_clicked(move |button| {
        ui::clear_box(&rows_box);
        append_transaction_pattern_rows(&rows_box, &patterns, &state_for_hidden, &ui_for_hidden);
        button.set_visible(false);
    });
    button
}

fn transaction_pattern_edit_row(
    card: gtk::Box,
    pattern: &analytics::TransactionPattern,
    hidden: bool,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> gtk::Overlay {
    card.set_hexpand(true);
    if let Some(content) = card
        .first_child()
        .and_then(|child| child.downcast::<gtk::Box>().ok())
    {
        content.set_margin_end(content.margin_end() + 84);
    }

    let overlay = gtk::Overlay::new();
    overlay.set_hexpand(true);
    overlay.set_child(Some(&card));

    let actions = ui::linked_button_group();
    actions.set_halign(gtk::Align::End);
    actions.set_valign(gtk::Align::Center);
    actions.set_margin_end(12);

    let visibility_button = if hidden {
        ui::icon_button("edit-undo-symbolic", "Reopen detected pattern")
    } else {
        ui::icon_button("window-close-symbolic", "Hide detected pattern")
    };
    visibility_button.add_css_class("flat");
    register_config_widget(ui_handles, &visibility_button);

    actions.append(&visibility_button);
    let edit_button = if ui_handles.advanced_features.get() {
        let edit_button = ui::icon_button("document-edit-symbolic", "Create rule from pattern");
        edit_button.add_css_class("flat");
        register_config_widget(ui_handles, &edit_button);
        actions.append(&edit_button);
        Some(edit_button)
    } else {
        None
    };

    let pattern_for_visibility = pattern.clone();
    let state_for_visibility = Rc::clone(state);
    let ui_for_visibility = Rc::clone(ui_handles);
    visibility_button.connect_clicked(move |_| {
        toggle_transaction_pattern_visibility(
            &pattern_for_visibility,
            hidden,
            &state_for_visibility,
            &ui_for_visibility,
        );
    });

    if let Some(edit_button) = edit_button {
        let pattern = pattern.clone();
        let state_for_edit = Rc::clone(state);
        let ui_for_edit = Rc::clone(ui_handles);
        edit_button.connect_clicked(move |_| {
            show_transaction_pattern_rule_dialog(&pattern, &state_for_edit, &ui_for_edit);
        });
    }

    overlay.add_overlay(&actions);
    overlay
}

fn toggle_transaction_pattern_visibility(
    pattern: &analytics::TransactionPattern,
    hidden: bool,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    if config_operation_is_active(ui_handles, "Another edit or save is already running.") {
        return;
    }

    let key = analytics::transaction_pattern_key(pattern);
    let result = if hidden {
        data::reopen_transaction_pattern(&key)
    } else {
        data::ignore_transaction_pattern(&key, &pattern.label)
    };

    match result {
        Ok(_) => {
            show_status(
                ui_handles,
                if hidden {
                    "Pattern reopened."
                } else {
                    "Pattern hidden. Use Show Hidden to reopen it."
                },
            );
            render_views(&state.borrow(), ui_handles, state);
        }
        Err(error) => show_status(
            ui_handles,
            &trf(
                "Could not update hidden pattern: {error}",
                &[("error", format!("{error:#}"))],
            ),
        ),
    }
}

fn show_transaction_pattern_rule_dialog(
    pattern: &analytics::TransactionPattern,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let initial = editable_rule_for_pattern(pattern, &state.borrow());

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header = ui::cancelable_dialog_header("Create Rule", &pattern.label);

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
        "Create a categorization rule from this detected transaction pattern.",
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
            ("tags", "Tags"),
            ("description", "Description"),
            ("counterparty", "Counterparty"),
            ("account", "Account"),
            ("transaction_id", "Transaction ID"),
        ],
        &initial.field,
    );
    let search = ui::text_combo(&initial.search, pattern_rule_search_values(pattern));
    let is_regex = gtk::Switch::builder()
        .active(initial.is_regex)
        .valign(gtk::Align::Center)
        .build();
    let category = ui::text_combo(&initial.category, app_category_values(&state.borrow()));
    let budget_code = ui::text_combo(
        &initial.budget_code,
        app_budget_code_values(&state.borrow()),
    );
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

    let status = ui::wrapped_label(&tr("Changes are saved to rules.csv."));
    status.add_css_class("dim-label");
    page.append(&status);
    root.append(&ui::scroll(&page));

    let dialog = adw::Dialog::builder()
        .title(tr("Create Rule"))
        .content_width(650)
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

        if save_rule_in_background(&state_for_save, &ui_for_save, rule, true) {
            button.set_sensitive(false);
            cancel_button_for_save.set_label(&tr("Close"));
            status.set_text(&tr("Saving rule..."));
            dialog_for_save.close();
        }
    });

    dialog.present(Some(&ui_handles.window));
}

fn editable_rule_for_pattern(
    pattern: &analytics::TransactionPattern,
    data: &AppData,
) -> EditableRule {
    let matches = data
        .transactions
        .iter()
        .filter(|transaction| analytics::transaction_matches_pattern(transaction, pattern))
        .collect::<Vec<_>>();
    let is_transfer = matches!(pattern.kind, analytics::TransactionPatternKind::Transfer);
    let category = if is_transfer {
        tr("Transfers")
    } else {
        most_common_text(
            matches
                .iter()
                .map(|transaction| transaction.category.trim())
                .filter(|category| !category.is_empty() && *category != "Uncategorized"),
        )
        .unwrap_or_else(|| pattern.label.clone())
    };
    let budget_code = if is_transfer {
        "TRANSFER".to_string()
    } else {
        most_common_text(
            matches
                .iter()
                .map(|transaction| transaction.budget_code.trim())
                .filter(|code| !code.is_empty()),
        )
        .unwrap_or_else(|| {
            if pattern.amount > Decimal::ZERO {
                "INC-OTHER".to_string()
            } else {
                "OTHER".to_string()
            }
        })
    };
    let direction = if is_transfer {
        "transfer"
    } else if pattern.amount > Decimal::ZERO {
        "income"
    } else if pattern.amount < Decimal::ZERO {
        "expense"
    } else {
        "any"
    };

    EditableRule {
        priority: 130,
        active: true,
        field: "tags".to_string(),
        search: pattern.label.clone(),
        is_regex: false,
        category,
        budget_code,
        direction: direction.to_string(),
        amount_min: String::new(),
        amount_max: String::new(),
        notes: tr("Generated from detected transaction pattern."),
    }
}

fn most_common_text<'a>(items: impl Iterator<Item = &'a str>) -> Option<String> {
    let mut counts = HashMap::<String, usize>::new();
    for item in items {
        *counts.entry(item.to_string()).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
        .map(|(item, _)| item)
}

#[derive(Debug, Clone)]
struct TransactionPatternInfo {
    badges: Vec<String>,
    hidden: bool,
    needs_rule: bool,
    affects_totals: bool,
}

fn transaction_pattern_info(
    pattern: &analytics::TransactionPattern,
    data: &AppData,
    rules: &[EditableRule],
    hidden: bool,
) -> TransactionPatternInfo {
    let cancels_out = matches!(
        pattern.kind,
        analytics::TransactionPatternKind::FullRefund
            | analytics::TransactionPatternKind::BillSplit
    );
    let is_transfer = matches!(pattern.kind, analytics::TransactionPatternKind::Transfer);
    let generated_rule = transaction_pattern_has_rule(pattern, rules);
    let covered_by_rule = transaction_pattern_is_covered(pattern, data);

    let mut badges = Vec::new();
    if cancels_out {
        badges.push(tr("Cancels out transactions"));
    }
    if is_transfer {
        badges.push(tr("Possible transfer"));
    }
    if generated_rule {
        badges.push(tr("Generated rule"));
    }
    if hidden {
        badges.push(tr("Hidden"));
    }
    if covered_by_rule {
        badges.push(tr("Covered by rule"));
    } else {
        badges.push(tr("Needs rule"));
    }

    TransactionPatternInfo {
        badges,
        hidden,
        needs_rule: !covered_by_rule,
        affects_totals: is_transfer || cancels_out,
    }
}

fn transaction_pattern_is_covered(pattern: &analytics::TransactionPattern, data: &AppData) -> bool {
    let matches = data
        .transactions
        .iter()
        .filter(|transaction| analytics::transaction_matches_pattern(transaction, pattern))
        .collect::<Vec<_>>();
    !matches.is_empty()
        && matches.iter().all(|transaction| {
            let category = transaction.category.trim();
            let code = transaction.budget_code.trim();
            !category.is_empty()
                && category != "Uncategorized"
                && !matches!(code, "" | "OTHER" | "INC-OTHER")
        })
}

fn transaction_pattern_has_rule(
    pattern: &analytics::TransactionPattern,
    rules: &[EditableRule],
) -> bool {
    rules.iter().filter(|rule| rule.active).any(|rule| {
        let search = rule.search.trim();
        !search.is_empty()
            && (search.eq_ignore_ascii_case(pattern.label.trim())
                || pattern
                    .match_labels
                    .iter()
                    .any(|label| search.eq_ignore_ascii_case(label)))
    })
}

fn duplicate_filtering_card<F>(data: &AppData, on_activate: F) -> gtk::Box
where
    F: Fn() + 'static,
{
    let enabled = data.dedupe_mode.is_enabled();
    let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
    card.add_css_class("card");
    card.set_margin_top(4);
    card.set_margin_bottom(4);
    card.set_margin_start(4);
    card.set_margin_end(4);
    card.set_hexpand(true);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 4);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header.set_hexpand(true);

    let title = gtk::Label::new(Some(&tr("Duplicates")));
    title.add_css_class("caption");
    title.set_xalign(0.0);
    title.set_hexpand(true);
    header.append(&title);

    let badge = gtk::Label::new(Some(&tr(if enabled { "Filtered" } else { "Unfiltered" })));
    badge.add_css_class("caption");
    badge.add_css_class(if enabled { "success" } else { "dim-label" });
    badge.set_tooltip_text(Some(&tr(if enabled {
        "Duplicate filtering is on"
    } else {
        "Duplicate filtering is off"
    })));
    badge.set_valign(gtk::Align::Start);
    header.append(&badge);
    content.append(&header);

    let value = gtk::Label::new(Some(&data.duplicate_count.to_string()));
    value.add_css_class("title-2");
    value.set_xalign(0.0);
    content.append(&value);

    let subtitle = ui::wrapped_label(&tr(data.dedupe_mode.description()));
    subtitle.add_css_class("dim-label");
    subtitle.set_width_chars(1);
    subtitle.set_max_width_chars(32);
    content.append(&subtitle);

    card.append(&content);
    ui::activatable_card(card, on_activate)
}

pub(in crate::app) fn import_report_matches(report: &ImportReport, filter: &SearchFilter) -> bool {
    let field_text = diagnostic_field_items(&report.guessed_fields)
        .into_iter()
        .map(|field| format!("{} {}", field.label, field.value.unwrap_or("Not detected")))
        .collect::<Vec<_>>()
        .join(" ");
    let header_text = report.headers.join(" ");
    let error_text = report.errors.join(" ");
    filter.matches(&format!(
        "{} {} {} {} {} {} {} {} {}",
        report.source.display(),
        delimiter_label(report.delimiter),
        header_text,
        report.rows_seen,
        report.rows_imported,
        report.rows_skipped,
        diagnostic_error_text(report.errors.len()),
        field_text,
        error_text,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_patterns_need_all_reload_only_for_partial_scopes() {
        let all = TransactionLoadScope::All;
        let year = TransactionLoadScope::Year(Some(2025));
        let month = TransactionLoadScope::Month(Some(MonthKey::new(2025, 5)));

        assert!(!transaction_patterns_need_all_reload(all));
        assert!(transaction_patterns_need_all_reload(year));
        assert!(transaction_patterns_need_all_reload(month));
    }

    #[test]
    fn transaction_patterns_section_requires_smart_insights() {
        assert!(transaction_patterns_section_visible(None, true));
        assert!(!transaction_patterns_section_visible(None, false));

        let pattern_search = SearchFilter::from_text("patterns").unwrap();
        assert!(transaction_patterns_section_visible(
            Some(&pattern_search),
            true
        ));
        assert!(!transaction_patterns_section_visible(
            Some(&pattern_search),
            false
        ));
    }

    #[test]
    fn transaction_patterns_section_still_respects_search_terms() {
        let warnings_search = SearchFilter::from_text("warnings").unwrap();

        assert!(!transaction_patterns_section_visible(
            Some(&warnings_search),
            true
        ));
    }
}
