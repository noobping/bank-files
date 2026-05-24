use super::*;

pub(super) fn connect_save_action(actions: &ManagementDialogActions<'_>) {
    let management_dialog = actions.management_dialog;
    let save_button = actions.save_button;
    let rules_forms = actions.rules_forms;
    let budgets_forms = actions.budgets_forms;
    let aliases_forms = actions.aliases_forms;
    let status = actions.status;
    let dialog_closed = Rc::clone(&actions.dialog_closed);
    let save_running = Rc::clone(&actions.save_running);
    let finish_management_dialog = Rc::clone(&actions.finish_management_dialog);
    let state = actions.state;
    let ui_handles = actions.ui_handles;

    let management_dialog_for_save = management_dialog.clone();
    let rules_forms_for_save = Rc::clone(rules_forms);
    let budgets_forms_for_save = Rc::clone(budgets_forms);
    let aliases_forms_for_save = Rc::clone(aliases_forms);
    let state_for_save = Rc::clone(state);
    let ui_for_save = Rc::clone(ui_handles);
    let status_for_save = status.clone();
    let dialog_closed_for_save = Rc::clone(&dialog_closed);
    let save_running_for_save = Rc::clone(&save_running);
    let finish_for_save = Rc::clone(&finish_management_dialog);
    save_button.connect_clicked(move |button| {
        let mut direction_changes =
            collect_budget_direction_changes(&budgets_forms_for_save.borrow());
        direction_changes.extend(collect_rule_direction_changes(
            &rules_forms_for_save.borrow(),
        ));
        show_verbose_status(
            ui_for_save.as_ref(),
            format!(
                "management save requested; direction_changes={}",
                direction_changes.len()
            ),
        );

        let management_dialog_for_confirm = management_dialog_for_save.clone();
        let rules_forms_for_save = Rc::clone(&rules_forms_for_save);
        let budgets_forms_for_save = Rc::clone(&budgets_forms_for_save);
        let aliases_forms_for_save = Rc::clone(&aliases_forms_for_save);
        let state_for_save = Rc::clone(&state_for_save);
        let ui_for_save = Rc::clone(&ui_for_save);
        let status_for_save = status_for_save.clone();
        let dialog_closed_for_save = Rc::clone(&dialog_closed_for_save);
        let save_running_for_save = Rc::clone(&save_running_for_save);
        let finish_for_save = Rc::clone(&finish_for_save);
        let button = button.clone();
        confirm_budget_direction_changes(
            &management_dialog_for_confirm,
            direction_changes,
            move || {
                let budget_renames = collect_budget_code_renames(&budgets_forms_for_save.borrow());
                let renamed_rule_count = apply_budget_code_renames_to_rule_forms(
                    &rules_forms_for_save.borrow(),
                    &budget_renames,
                );
                let rules = collect_rule_forms(&rules_forms_for_save.borrow());
                let budgets = collect_budget_forms(&budgets_forms_for_save.borrow());
                let aliases = collect_alias_forms(&aliases_forms_for_save.borrow());
                show_verbose_status(
                    ui_for_save.as_ref(),
                    format!(
                    "management save started; rules={}; budgets={}; aliases={}; renamed_rules={}",
                    rules.len(),
                    budgets.len(),
                    aliases.len(),
                    renamed_rule_count
                ),
                );
                let borrowed = state_for_save.borrow();
                let mode = borrowed.dedupe_mode;
                let remember_mode = ui_for_save.remember_mode.get();
                let sources = current_sources_for_reload(&borrowed, remember_mode);
                let scope = current_transaction_load_scope(&borrowed, ui_for_save.as_ref());
                drop(borrowed);
                let auto_clean_config = ui_for_save.preferences.auto_clean_config();
                let smart_insights_enabled = smart_pattern_detection_enabled(
                    ui_for_save.advanced_features.get(),
                    ui_for_save.show_predictions.get(),
                );
                let state_for_save = Rc::clone(&state_for_save);
                let ui_for_save = Rc::clone(&ui_for_save);
                let status_for_save = status_for_save.clone();
                let rules_forms_for_save = Rc::clone(&rules_forms_for_save);
                let budgets_forms_for_save = Rc::clone(&budgets_forms_for_save);
                let dialog_closed_for_save = Rc::clone(&dialog_closed_for_save);
                let save_running_for_save = Rc::clone(&save_running_for_save);
                let finish_for_save = Rc::clone(&finish_for_save);
                save_running_for_save.set(true);
                button.set_sensitive(false);
                status_for_save.set_text(&tr("Saving changes..."));
                show_status(&ui_for_save, "Saving changes...");

                gtk::glib::MainContext::default().spawn_local(async move {
                    let task = gtk::gio::spawn_blocking(move || {
                        data::write_editable_rules(&rules)?;
                        data::write_editable_budgets(&budgets)?;
                        data::write_editable_aliases(&aliases)?;
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
                            *state_for_save.borrow_mut() = new_data;
                            mark_rule_forms_saved(&rules_forms_for_save.borrow());
                            mark_budget_forms_saved(&budgets_forms_for_save.borrow());
                            render_views(&state_for_save.borrow(), &ui_for_save, &state_for_save);
                            let message = if renamed_rule_count == 0 {
                                tr("Changes saved.")
                            } else {
                                trf(
                            "Changes saved. {count} rule(s) updated for renamed budget codes.",
                            &[("count", renamed_rule_count.to_string())],
                        )
                            };
                            status_for_save.set_text(&message);
                            show_status(&ui_for_save, &message);
                            show_verbose_status(ui_for_save.as_ref(), "management save finished");
                        }
                        Ok(Err(err)) => {
                            let message =
                                trf("Save failed: {error}", &[("error", format!("{err:#}"))]);
                            status_for_save.set_text(&message);
                            show_status(&ui_for_save, &message);
                            show_verbose_status(
                                ui_for_save.as_ref(),
                                format!("management save failed; error={err:#}"),
                            );
                        }
                        Err(_) => {
                            let message =
                                tr("Save canceled: the background task stopped unexpectedly.");
                            status_for_save.set_text(&message);
                            show_status(&ui_for_save, &message);
                            show_verbose_status(
                                ui_for_save.as_ref(),
                                "management save task canceled",
                            );
                        }
                    }
                    save_running_for_save.set(false);
                    if dialog_closed_for_save.get() {
                        finish_for_save();
                    } else {
                        button.set_sensitive(true);
                    }
                });
            },
        );
    });
}
