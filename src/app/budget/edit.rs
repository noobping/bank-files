use super::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(in crate::app) struct BudgetDirectionChange {
    pub(in crate::app) item: String,
}

pub(in crate::app) fn budget_direction_change(
    item: &str,
    from: BudgetDirection,
    to: BudgetDirection,
) -> Option<BudgetDirectionChange> {
    if budget_direction_change_needs_confirmation(from, to) {
        Some(BudgetDirectionChange {
            item: item.trim().to_string(),
        })
    } else {
        None
    }
}

fn budget_direction_change_needs_confirmation(from: BudgetDirection, to: BudgetDirection) -> bool {
    matches!(
        (from, to),
        (BudgetDirection::Expense, BudgetDirection::Income)
            | (BudgetDirection::Income, BudgetDirection::Expense)
    )
}

pub(in crate::app) fn confirm_budget_direction_changes<F>(
    parent: &impl IsA<gtk::Widget>,
    changes: Vec<BudgetDirectionChange>,
    on_confirm: F,
) where
    F: FnOnce() + 'static,
{
    if changes.is_empty() {
        on_confirm();
        return;
    }

    let heading = if changes.len() == 1 {
        tr("Change budget direction?")
    } else {
        tr("Change budget directions?")
    };
    let body = budget_direction_change_confirmation_body(&changes);
    let dialog = adw::AlertDialog::new(Some(&heading), Some(&body));
    dialog.add_responses(&[
        ("cancel", &tr("Cancel")),
        ("change", &tr("Change Direction")),
    ]);
    dialog.set_close_response("cancel");
    dialog.set_default_response(Some("cancel"));
    dialog.set_response_appearance("change", adw::ResponseAppearance::Destructive);
    dialog.choose(
        Some(parent),
        None::<&gtk::gio::Cancellable>,
        move |response| {
            if response.as_str() == "change" {
                on_confirm();
            }
        },
    );
}

fn budget_direction_change_confirmation_body(changes: &[BudgetDirectionChange]) -> String {
    if let [change] = changes {
        if change.item.is_empty() {
            tr("This item will switch between expenses and income. This can move related transactions between spending and income totals.")
        } else {
            trf(
                "{item} will switch between expenses and income. This can move related transactions between spending and income totals.",
                &[("item", change.item.clone())],
            )
        }
    } else {
        trf(
            "{count} items will switch between expenses and income. This can move related transactions between spending and income totals.",
            &[("count", changes.len().to_string())],
        )
    }
}

struct BudgetSaveUi {
    button: gtk::Button,
    delete_button: Option<gtk::Button>,
    delete_button_sensitive: bool,
    status: gtk::Label,
    dialog: adw::Dialog,
}

fn save_budget_with_reload(
    budget: EditableBudget,
    save_ui: BudgetSaveUi,
    state: Rc<RefCell<AppData>>,
    ui_handles: Rc<UiHandles>,
) {
    if !try_begin_config_operation(&ui_handles, "Another edit or save is already running.") {
        return;
    }

    let BudgetSaveUi {
        button,
        delete_button,
        delete_button_sensitive,
        status,
        dialog,
    } = save_ui;
    button.set_sensitive(false);
    if let Some(delete_button) = &delete_button {
        delete_button.set_sensitive(false);
    }
    status.set_text(&tr("Saving budget..."));

    let borrowed = state.borrow();
    let mode = borrowed.dedupe_mode;
    let remember_mode = ui_handles.remember_mode.get();
    let sources = current_sources_for_reload(&borrowed, remember_mode);
    let scope = current_transaction_load_scope(&borrowed, ui_handles.as_ref());
    drop(borrowed);
    let auto_clean_config = ui_handles.preferences.auto_clean_config();
    let smart_insights_enabled = ui_handles.show_predictions.get();

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            upsert_budget(budget)?;
            let new_data = data::load_app_data_with_sources(
                mode,
                auto_clean_config,
                scope,
                remember_mode,
                &sources,
                smart_insights_enabled,
            )?
            .0;
            anyhow::Ok(new_data)
        });

        match task.await {
            Ok(Ok(new_data)) => {
                *state.borrow_mut() = new_data;
                render_views(&state.borrow(), &ui_handles, &state);
                show_status(&ui_handles, &tr("Budget saved"));
                dialog.close();
            }
            Ok(Err(err)) => {
                status.set_text(&trf(
                    "Could not save budget: {error}",
                    &[("error", format!("{err:#}"))],
                ));
                button.set_sensitive(true);
                if let Some(delete_button) = &delete_button {
                    delete_button.set_sensitive(delete_button_sensitive);
                }
            }
            Err(_) => {
                status.set_text(&tr(
                    "Budget save canceled: the background task stopped unexpectedly.",
                ));
                button.set_sensitive(true);
                if let Some(delete_button) = &delete_button {
                    delete_button.set_sensitive(delete_button_sensitive);
                }
            }
        }
        finish_config_operation(&ui_handles);
    });
}

pub(in crate::app) fn budget_direction_editable(advanced_features: bool, persisted: bool) -> bool {
    advanced_features || !persisted
}

pub(in crate::app) fn budget_edit_button(
    code: &str,
    category: &str,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) -> gtk::Button {
    let button = ui::icon_button("document-edit-symbolic", "Edit budget");
    button.add_css_class("flat");
    button.set_halign(gtk::Align::End);
    button.set_valign(gtk::Align::Center);
    button.set_sensitive(!code.trim().is_empty());
    register_exclusive_config_widget(ui_handles, &button);

    let code = code.trim().to_string();
    let category = category.trim().to_string();
    let ui_for_edit = Rc::clone(ui_handles);
    let state_for_edit = Rc::clone(state);
    button.connect_clicked(move |_| {
        if config_operation_is_active(&ui_for_edit, "Another edit or save is already running.") {
            return;
        }
        show_budget_edit_dialog(&code, &category, &state_for_edit, &ui_for_edit);
    });

    button
}

pub(in crate::app) fn show_budget_edit_dialog(
    code: &str,
    fallback_category: &str,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let (initial, can_delete_budget) = editable_budget_for(code, fallback_category);
    if planned_income::is_budget_code(&initial.code) {
        show_planned_income_budget_edit_dialog(initial, can_delete_budget, state, ui_handles);
        return;
    }

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header = ui::cancelable_dialog_header("Edit Budget", code);

    let delete_button = ui::icon_button("user-trash-symbolic", "Delete budget");
    delete_button.add_css_class("destructive-action");
    delete_button.set_sensitive(can_delete_budget);
    let save_button = ui::primary_text_icon_button("document-save-symbolic", "Save", "Save budget");
    register_exclusive_config_widget(ui_handles, &save_button);
    register_exclusive_config_widget(ui_handles, &delete_button);
    header.pack_start(&delete_button);
    header.pack_end(&save_button);
    root.append(&header);

    let page = ui::page_box();
    page.append(&ui::section_title(
        "Edit Budget",
        "Use monthly budgets, yearly budgets, or both. Percentage budgets can use real or planned income.",
    ));

    let grid = ui::form_grid();
    let advanced_features = ui_handles.advanced_features.get();
    if advanced_features {
        ui::add_labeled(&grid, 0, "Budget code", &ui::wrapped_label(&initial.code));
    }
    let category = ui::text_combo(&initial.category, app_category_values(&state.borrow()));
    let monthly_budget = ui::entry(&initial.monthly_budget, "500 or 10% of income");
    let yearly_budget = ui::entry(&initial.yearly_budget, "5000 or 10% of yearly income");
    let direction = ui::combo_from_options(
        &[
            ("expense", "Expenses"),
            ("income", "Income"),
            ("transfer", "Transfers"),
        ],
        ui::budget_direction_id(&initial.direction),
    );
    let income_basis = ui::combo_from_options(
        &[("real", "Real income"), ("planned", "Planned income")],
        ui::budget_income_basis_id(&initial.income_basis),
    );
    if !budget_direction_editable(advanced_features, can_delete_budget) {
        direction.set_sensitive(false);
    }
    let notes = ui::entry(&initial.notes, "Note");
    let first_row = if advanced_features { 1 } else { 0 };
    ui::add_labeled(&grid, first_row, "Category", &category);
    ui::add_labeled(&grid, first_row + 1, "Monthly budget", &monthly_budget);
    ui::add_labeled(&grid, first_row + 2, "Yearly budget", &yearly_budget);
    ui::add_labeled(&grid, first_row + 3, "Direction", &direction);
    let income_basis_label =
        ui::add_labeled(&grid, first_row + 4, "Percentage basis", &income_basis);
    ui::add_labeled(&grid, first_row + 5, "Note", &notes);
    bind_percentage_basis_visibility(
        &monthly_budget,
        &yearly_budget,
        &income_basis_label,
        &income_basis,
    );
    page.append(&grid);

    let status = ui::wrapped_label(&tr("Changes are saved to your budget configuration."));
    status.add_css_class("dim-label");
    page.append(&status);
    root.append(&ui::action_dialog_scroll(&page));

    let dialog = adw::Dialog::builder()
        .title(tr("Edit Budget"))
        .content_width(620)
        .default_widget(&save_button)
        .child(&root)
        .build();

    let state_for_save = Rc::clone(state);
    let ui_for_save = Rc::clone(ui_handles);
    let dialog_for_save = dialog.clone();
    let code_for_save = initial.code.clone();
    let category_for_save = initial.category.clone();
    let direction_for_save = initial.direction.clone();
    let status_for_save = status.clone();
    let delete_button_for_save = delete_button.clone();
    save_button.connect_clicked(move |button| {
        let category_text = ui::combo_text(&category);
        if category_text.is_empty() {
            status_for_save.set_text(&tr("Enter a category first."));
            category.grab_focus();
            return;
        }

        let budget = EditableBudget {
            code: code_for_save.clone(),
            category: category_text,
            monthly_budget: monthly_budget.text().trim().to_string(),
            yearly_budget: yearly_budget.text().trim().to_string(),
            direction: ui::combo_active_id(&direction),
            income_basis: ui::combo_active_id(&income_basis),
            notes: notes.text().trim().to_string(),
        };

        let direction_changes = if can_delete_budget {
            let from =
                BudgetDirection::parse(&direction_for_save, &code_for_save, &category_for_save);
            let to = BudgetDirection::parse(&budget.direction, &budget.code, &budget.category);
            budget_direction_change(&budget.code, from, to)
                .into_iter()
                .collect()
        } else {
            Vec::new()
        };

        let save_ui = BudgetSaveUi {
            button: button.clone(),
            delete_button: Some(delete_button_for_save.clone()),
            delete_button_sensitive: can_delete_budget,
            status: status_for_save.clone(),
            dialog: dialog_for_save.clone(),
        };
        let state_for_save = Rc::clone(&state_for_save);
        let ui_for_save = Rc::clone(&ui_for_save);
        let dialog_for_confirm = save_ui.dialog.clone();
        confirm_budget_direction_changes(&dialog_for_confirm, direction_changes, move || {
            save_budget_with_reload(budget, save_ui, state_for_save, ui_for_save);
        });
    });

    connect_budget_delete_action(BudgetDeleteAction {
        delete_button: &delete_button,
        save_button: &save_button,
        status: &status,
        dialog: &dialog,
        code: initial.code.clone(),
        state: Rc::clone(state),
        ui_handles: Rc::clone(ui_handles),
    });

    dialog.present(Some(&ui_handles.window));
}

struct BudgetDeleteAction<'a> {
    delete_button: &'a gtk::Button,
    save_button: &'a gtk::Button,
    status: &'a gtk::Label,
    dialog: &'a adw::Dialog,
    code: String,
    state: Rc<RefCell<AppData>>,
    ui_handles: Rc<UiHandles>,
}

fn connect_budget_delete_action(action: BudgetDeleteAction<'_>) {
    let BudgetDeleteAction {
        delete_button,
        save_button,
        status,
        dialog,
        code,
        state,
        ui_handles,
    } = action;

    let save_button_for_delete = save_button.clone();
    let status_for_delete = status.clone();
    let dialog_for_delete = dialog.clone();
    delete_button.connect_clicked(move |button| {
        if !try_begin_config_operation(&ui_handles, "Another edit or save is already running.") {
            return;
        }

        let button = button.clone();
        let save_button = save_button_for_delete.clone();
        let code = code.clone();
        let borrowed = state.borrow();
        let mode = borrowed.dedupe_mode;
        let remember_mode = ui_handles.remember_mode.get();
        let sources = current_sources_for_reload(&borrowed, remember_mode);
        let scope = current_transaction_load_scope(&borrowed, ui_handles.as_ref());
        drop(borrowed);
        let auto_clean_config = ui_handles.preferences.auto_clean_config();
        let smart_insights_enabled = ui_handles.show_predictions.get();
        let state = Rc::clone(&state);
        let ui_handles = Rc::clone(&ui_handles);
        let dialog_for_delete = dialog_for_delete.clone();
        let status_for_delete = status_for_delete.clone();
        button.set_sensitive(false);
        save_button.set_sensitive(false);
        status_for_delete.set_text(&tr("Removing budget..."));

        gtk::glib::MainContext::default().spawn_local(async move {
            let task = gtk::gio::spawn_blocking(move || {
                if !delete_budget(&code)? {
                    return anyhow::Ok(None);
                }
                let new_data = data::load_app_data_with_sources(
                    mode,
                    auto_clean_config,
                    scope,
                    remember_mode,
                    &sources,
                    smart_insights_enabled,
                )?
                .0;
                anyhow::Ok(Some(new_data))
            });

            match task.await {
                Ok(Ok(Some(new_data))) => {
                    *state.borrow_mut() = new_data;
                    render_views(&state.borrow(), &ui_handles, &state);
                    show_status(&ui_handles, &tr("Budget removed"));
                    dialog_for_delete.close();
                }
                Ok(Ok(None)) => {
                    status_for_delete.set_text(&tr("Budget was already removed."));
                    button.set_sensitive(false);
                    save_button.set_sensitive(true);
                }
                Ok(Err(err)) => {
                    status_for_delete.set_text(&trf(
                        "Could not remove budget: {error}",
                        &[("error", format!("{err:#}"))],
                    ));
                    button.set_sensitive(true);
                    save_button.set_sensitive(true);
                }
                Err(_) => {
                    status_for_delete.set_text(&tr(
                        "Budget removal canceled: the background task stopped unexpectedly.",
                    ));
                    button.set_sensitive(true);
                    save_button.set_sensitive(true);
                }
            }
            finish_config_operation(&ui_handles);
        });
    });
}

fn show_planned_income_budget_edit_dialog(
    initial: EditableBudget,
    can_delete_budget: bool,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let advanced_features = ui_handles.advanced_features.get();
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header = ui::cancelable_dialog_header(
        "Edit Planned Income",
        if advanced_features {
            planned_income::BUDGET_CODE
        } else {
            "Planned income"
        },
    );

    let delete_button = ui::icon_button("user-trash-symbolic", "Delete planned income budget");
    delete_button.add_css_class("destructive-action");
    delete_button.set_sensitive(can_delete_budget);
    let save_button = ui::primary_text_icon_button(
        "document-save-symbolic",
        "Save",
        "Save planned income budget",
    );
    register_exclusive_config_widget(ui_handles, &save_button);
    register_exclusive_config_widget(ui_handles, &delete_button);
    header.pack_start(&delete_button);
    header.pack_end(&save_button);
    root.append(&header);

    let page = ui::page_box();
    page.append(&ui::section_title(
        "Planned Income",
        if advanced_features {
            "Set fixed monthly and yearly planned income for the INC budget code."
        } else {
            "Set fixed monthly and yearly planned income."
        },
    ));
    let reminder = ui::wrapped_label(&tr(planned_income::NET_INCOME_REMINDER));
    reminder.add_css_class("dim-label");
    page.append(&reminder);

    let grid = ui::form_grid();
    if advanced_features {
        ui::add_labeled(
            &grid,
            0,
            "Budget code",
            &ui::wrapped_label(planned_income::BUDGET_CODE),
        );
    }
    let category = ui::text_combo(&initial.category, app_category_values(&state.borrow()));
    let monthly_budget = ui::entry(
        &planned_income::fixed_budget_amount_text(&initial.monthly_budget),
        "500",
    );
    let yearly_budget = ui::entry(
        &planned_income::fixed_budget_amount_text(&initial.yearly_budget),
        "5000",
    );
    planned_income::connect_fixed_budget_entry(&monthly_budget);
    planned_income::connect_fixed_budget_entry(&yearly_budget);
    let notes = ui::entry(&initial.notes, "Note");
    let first_row = if advanced_features { 1 } else { 0 };
    ui::add_labeled(&grid, first_row, "Category", &category);
    ui::add_labeled(
        &grid,
        first_row + 1,
        "Monthly planned income",
        &monthly_budget,
    );
    ui::add_labeled(
        &grid,
        first_row + 2,
        "Yearly planned income",
        &yearly_budget,
    );
    ui::add_labeled(&grid, first_row + 3, "Note", &notes);
    page.append(&grid);

    let status = ui::wrapped_label(&tr("Changes are saved to your budget configuration."));
    status.add_css_class("dim-label");
    page.append(&status);
    root.append(&ui::action_dialog_scroll(&page));

    let dialog = adw::Dialog::builder()
        .title(tr("Edit Planned Income"))
        .content_width(620)
        .default_widget(&save_button)
        .child(&root)
        .build();

    let state_for_save = Rc::clone(state);
    let ui_for_save = Rc::clone(ui_handles);
    let dialog_for_save = dialog.clone();
    let status_for_save = status.clone();
    let delete_button_for_save = delete_button.clone();
    save_button.connect_clicked(move |button| {
        let category_text = ui::combo_text(&category);
        if category_text.is_empty() {
            status_for_save.set_text(&tr("Enter a category first."));
            category.grab_focus();
            return;
        }

        let budget = planned_income::editable_budget(
            category_text,
            monthly_budget.text().trim().to_string(),
            yearly_budget.text().trim().to_string(),
            notes.text().trim().to_string(),
        );
        save_budget_with_reload(
            budget,
            BudgetSaveUi {
                button: button.clone(),
                delete_button: Some(delete_button_for_save.clone()),
                delete_button_sensitive: can_delete_budget,
                status: status_for_save.clone(),
                dialog: dialog_for_save.clone(),
            },
            Rc::clone(&state_for_save),
            Rc::clone(&ui_for_save),
        );
    });

    connect_budget_delete_action(BudgetDeleteAction {
        delete_button: &delete_button,
        save_button: &save_button,
        status: &status,
        dialog: &dialog,
        code: planned_income::BUDGET_CODE.to_string(),
        state: Rc::clone(state),
        ui_handles: Rc::clone(ui_handles),
    });

    dialog.present(Some(&ui_handles.window));
}

fn editable_budget_for(code: &str, fallback_category: &str) -> (EditableBudget, bool) {
    let code = code.trim();
    let configured_budget = data::load_editable_budgets().ok().and_then(|budgets| {
        budgets
            .into_iter()
            .find(|budget| budget.code.trim().eq_ignore_ascii_case(code))
    });

    match configured_budget {
        Some(budget) => (budget, true),
        None => {
            let category = fallback_category.trim();
            let direction = BudgetDirection::parse("", code, category);
            (
                EditableBudget {
                    code: code.to_string(),
                    category: category.to_string(),
                    monthly_budget: "0".to_string(),
                    yearly_budget: String::new(),
                    direction: direction.as_str().to_string(),
                    income_basis: "real".to_string(),
                    notes: String::new(),
                },
                false,
            )
        }
    }
}

fn upsert_budget(budget: EditableBudget) -> anyhow::Result<()> {
    let code = budget.code.trim();
    if code.is_empty() {
        anyhow::bail!("Budget code is required");
    }

    let mut budgets = data::load_editable_budgets()?;
    let mut updated = false;
    for existing in &mut budgets {
        if existing.code.trim().eq_ignore_ascii_case(code) {
            *existing = budget.clone();
            updated = true;
        }
    }
    if !updated {
        budgets.push(budget);
    }

    data::write_editable_budgets(&budgets)?;
    Ok(())
}

fn delete_budget(code: &str) -> anyhow::Result<bool> {
    let code = code.trim();
    if code.is_empty() {
        anyhow::bail!("Budget code is required");
    }

    let mut budgets = data::load_editable_budgets()?;
    let original_len = budgets.len();
    budgets.retain(|budget| !budget.code.trim().eq_ignore_ascii_case(code));
    let removed = budgets.len() != original_len;
    if removed {
        data::write_editable_budgets(&budgets)?;
    }

    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_direction_editability_allows_simple_new_budgets() {
        assert!(budget_direction_editable(false, false));
        assert!(!budget_direction_editable(false, true));
        assert!(budget_direction_editable(true, true));
    }

    #[test]
    fn budget_direction_change_confirms_expense_income_crossing() {
        assert!(
            budget_direction_change("FOOD", BudgetDirection::Expense, BudgetDirection::Income,)
                .is_some()
        );
        assert!(budget_direction_change(
            "SALARY",
            BudgetDirection::Income,
            BudgetDirection::Expense,
        )
        .is_some());
    }

    #[test]
    fn budget_direction_change_ignores_same_direction_and_transfers() {
        assert!(budget_direction_change(
            "FOOD",
            BudgetDirection::Expense,
            BudgetDirection::Expense,
        )
        .is_none());
        assert!(budget_direction_change(
            "SAVE",
            BudgetDirection::Expense,
            BudgetDirection::Transfer,
        )
        .is_none());
        assert!(budget_direction_change(
            "SAVE",
            BudgetDirection::Transfer,
            BudgetDirection::Income,
        )
        .is_none());
    }
}
