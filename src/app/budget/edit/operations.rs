use super::*;

pub(in crate::app::budget::edit) struct BudgetSaveUi {
    pub(in crate::app::budget::edit) button: gtk::Button,
    pub(in crate::app::budget::edit) delete_button: Option<gtk::Button>,
    pub(in crate::app::budget::edit) delete_button_sensitive: bool,
    pub(in crate::app::budget::edit) status: gtk::Label,
    pub(in crate::app::budget::edit) dialog: adw::Dialog,
}

pub(in crate::app::budget::edit) fn save_budget_with_reload(
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

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            upsert_budget(budget)?;
            let new_data = data::load_app_data_with_sources(
                mode,
                auto_clean_config,
                scope,
                remember_mode,
                &sources,
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

pub(in crate::app::budget::edit) struct BudgetDeleteAction<'a> {
    pub(in crate::app::budget::edit) delete_button: &'a gtk::Button,
    pub(in crate::app::budget::edit) save_button: &'a gtk::Button,
    pub(in crate::app::budget::edit) status: &'a gtk::Label,
    pub(in crate::app::budget::edit) dialog: &'a adw::Dialog,
    pub(in crate::app::budget::edit) code: String,
    pub(in crate::app::budget::edit) state: Rc<RefCell<AppData>>,
    pub(in crate::app::budget::edit) ui_handles: Rc<UiHandles>,
}

pub(in crate::app::budget::edit) fn connect_budget_delete_action(action: BudgetDeleteAction<'_>) {
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

pub(in crate::app::budget::edit) fn editable_budget_for(
    code: &str,
    fallback_category: &str,
) -> (EditableBudget, bool) {
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
