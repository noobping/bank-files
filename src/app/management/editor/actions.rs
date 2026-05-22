use super::*;
use crate::model::BudgetAmount;

pub(in crate::app::management::editor) struct ManagementDialogActions<'a> {
    pub(in crate::app::management::editor) management_dialog: &'a adw::Dialog,
    pub(in crate::app::management::editor) add_button: &'a gtk::Button,
    pub(in crate::app::management::editor) add_rule_button: &'a gtk::Button,
    pub(in crate::app::management::editor) group_rules_button: &'a gtk::Button,
    pub(in crate::app::management::editor) combine_rules_button: &'a gtk::Button,
    pub(in crate::app::management::editor) add_budget_button: &'a gtk::Button,
    pub(in crate::app::management::editor) move_budget_code_button: &'a gtk::Button,
    pub(in crate::app::management::editor) use_real_income_button: &'a gtk::Button,
    pub(in crate::app::management::editor) use_planned_income_button: &'a gtk::Button,
    pub(in crate::app::management::editor) use_monthly_values_button: &'a gtk::Button,
    pub(in crate::app::management::editor) use_yearly_values_button: &'a gtk::Button,
    pub(in crate::app::management::editor) add_alias_button: &'a gtk::Button,
    pub(in crate::app::management::editor) cancel_button: &'a gtk::Button,
    pub(in crate::app::management::editor) save_button: &'a gtk::Button,
    pub(in crate::app::management::editor) page_actions_button: &'a gtk::MenuButton,
    pub(in crate::app::management::editor) stack: &'a adw::ViewStack,
    pub(in crate::app::management::editor) filter_entry: &'a gtk::SearchEntry,
    pub(in crate::app::management::editor) filter_search_bar: &'a gtk::SearchBar,
    pub(in crate::app::management::editor) rules_list: &'a gtk::Box,
    pub(in crate::app::management::editor) rules_forms: &'a Rc<RefCell<Vec<RuleForm>>>,
    pub(in crate::app::management::editor) rules_scroll: &'a gtk::ScrolledWindow,
    pub(in crate::app::management::editor) budgets_list: &'a gtk::Box,
    pub(in crate::app::management::editor) budgets_forms: &'a Rc<RefCell<Vec<BudgetForm>>>,
    pub(in crate::app::management::editor) budgets_scroll: &'a gtk::ScrolledWindow,
    pub(in crate::app::management::editor) aliases_list: &'a gtk::Box,
    pub(in crate::app::management::editor) aliases_forms: &'a Rc<RefCell<Vec<AliasForm>>>,
    pub(in crate::app::management::editor) aliases_scroll: &'a gtk::ScrolledWindow,
    pub(in crate::app::management::editor) status: &'a gtk::Label,
    pub(in crate::app::management::editor) dialog_closed: Rc<Cell<bool>>,
    pub(in crate::app::management::editor) save_running: Rc<Cell<bool>>,
    pub(in crate::app::management::editor) finish_management_dialog: Rc<dyn Fn()>,
    pub(in crate::app::management::editor) state: &'a Rc<RefCell<AppData>>,
    pub(in crate::app::management::editor) ui_handles: &'a Rc<UiHandles>,
}

pub(in crate::app::management::editor) fn connect_management_dialog_actions(
    actions: ManagementDialogActions<'_>,
) {
    let ManagementDialogActions {
        management_dialog,
        add_button,
        add_rule_button,
        group_rules_button,
        combine_rules_button,
        add_budget_button,
        move_budget_code_button,
        use_real_income_button,
        use_planned_income_button,
        use_monthly_values_button,
        use_yearly_values_button,
        add_alias_button,
        cancel_button,
        save_button,
        page_actions_button,
        stack,
        filter_entry,
        filter_search_bar,
        rules_list,
        rules_forms,
        rules_scroll,
        budgets_list,
        budgets_forms,
        budgets_scroll,
        aliases_list,
        aliases_forms,
        aliases_scroll,
        status,
        dialog_closed,
        save_running,
        finish_management_dialog,
        state,
        ui_handles,
    } = actions;
    connect_management_page_actions(
        page_actions_button,
        stack,
        rules_forms,
        budgets_forms,
        aliases_forms,
        status,
        ui_handles,
    );
    let management_dialog_for_header_add = management_dialog.clone();
    let rules_list_for_header_add = rules_list.clone();
    let rules_forms_for_header_add = Rc::clone(rules_forms);
    let rules_scroll_for_header_add = rules_scroll.clone();
    let budgets_list_for_header_add = budgets_list.clone();
    let budgets_forms_for_header_add = Rc::clone(budgets_forms);
    let budgets_scroll_for_header_add = budgets_scroll.clone();
    let aliases_list_for_header_add = aliases_list.clone();
    let aliases_forms_for_header_add = Rc::clone(aliases_forms);
    let aliases_scroll_for_header_add = aliases_scroll.clone();
    let status_for_header_add = status.clone();
    let stack_for_add = stack.clone();
    let filter_entry_for_header_add = filter_entry.clone();
    let advanced_autofill_for_header_add = Rc::clone(&ui_handles.advanced_autofill);
    let ui_for_header_add = Rc::clone(ui_handles);
    add_button.connect_clicked(
        move |_| match stack_for_add.visible_child_name().as_deref() {
            Some("budgets") => show_new_budget_dialog(
                &management_dialog_for_header_add,
                &budgets_list_for_header_add,
                &budgets_forms_for_header_add,
                &budgets_scroll_for_header_add,
                &status_for_header_add,
                &filter_entry_for_header_add,
                &advanced_autofill_for_header_add,
                ui_for_header_add.advanced_features.get(),
            ),
            Some("aliases") => show_new_alias_dialog(
                &management_dialog_for_header_add,
                &aliases_list_for_header_add,
                &aliases_forms_for_header_add,
                &aliases_scroll_for_header_add,
                &status_for_header_add,
                &filter_entry_for_header_add,
            ),
            _ => show_new_rule_dialog(
                &management_dialog_for_header_add,
                &rules_list_for_header_add,
                &rules_forms_for_header_add,
                &rules_scroll_for_header_add,
                &status_for_header_add,
                &filter_entry_for_header_add,
                &advanced_autofill_for_header_add,
            ),
        },
    );

    let management_dialog_for_rule_add = management_dialog.clone();
    let rules_list_for_rule_add = rules_list.clone();
    let rules_forms_for_rule_add = Rc::clone(rules_forms);
    let rules_scroll_for_rule_add = rules_scroll.clone();
    let status_for_rule_add = status.clone();
    let filter_entry_for_rule_add = filter_entry.clone();
    let advanced_autofill_for_rule_add = Rc::clone(&ui_handles.advanced_autofill);
    add_rule_button.connect_clicked(move |_| {
        show_new_rule_dialog(
            &management_dialog_for_rule_add,
            &rules_list_for_rule_add,
            &rules_forms_for_rule_add,
            &rules_scroll_for_rule_add,
            &status_for_rule_add,
            &filter_entry_for_rule_add,
            &advanced_autofill_for_rule_add,
        );
    });

    let rules_list_for_group = rules_list.clone();
    let rules_forms_for_group = Rc::clone(rules_forms);
    let rules_scroll_for_group = rules_scroll.clone();
    let filter_entry_for_group = filter_entry.clone();
    let status_for_group = status.clone();
    let group_button_for_group = group_rules_button.clone();
    let combine_button_for_group = combine_rules_button.clone();
    let advanced_autofill_for_group = Rc::clone(&ui_handles.advanced_autofill);
    group_rules_button.connect_clicked(move |_| {
        set_rule_bulk_buttons_sensitive(&group_button_for_group, &combine_button_for_group, false);
        status_for_group.set_text(&tr("Grouping compatible rules..."));

        let rules_list = rules_list_for_group.clone();
        let rules_forms = Rc::clone(&rules_forms_for_group);
        let rules_scroll = rules_scroll_for_group.clone();
        let filter_entry = filter_entry_for_group.clone();
        let status = status_for_group.clone();
        let group_button = group_button_for_group.clone();
        let combine_button = combine_button_for_group.clone();
        let advanced_autofill = Rc::clone(&advanced_autofill_for_group);
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(30), move || {
            let report = data::group_editable_rules_for_combining(&collect_rule_forms(
                &rules_forms.borrow(),
            ));
            if report.grouped_groups == 0 {
                status.set_text(&tr("No compatible rules to group."));
                set_rule_bulk_buttons_sensitive(&group_button, &combine_button, true);
                return;
            }
            if !report.changed {
                status.set_text(&tr("Compatible rules are already grouped. Use Combine."));
                set_rule_bulk_buttons_sensitive(&group_button, &combine_button, true);
                return;
            }

            let group_count = report.grouped_groups;
            replace_rule_forms(
                &rules_list,
                &rules_forms,
                report.rules,
                &advanced_autofill,
                &filter_entry,
                &rules_scroll,
            );
            status.set_text(&trf(
                "Grouped compatible rules into {group_count} group(s). Review order, then Combine or Save.",
                &[("group_count", group_count.to_string())],
            ));
            set_rule_bulk_buttons_sensitive(&group_button, &combine_button, true);
        });
    });

    let rules_list_for_combine = rules_list.clone();
    let rules_forms_for_combine = Rc::clone(rules_forms);
    let rules_scroll_for_combine = rules_scroll.clone();
    let filter_entry_for_combine = filter_entry.clone();
    let status_for_combine = status.clone();
    let group_button_for_combine = group_rules_button.clone();
    let combine_button_for_combine = combine_rules_button.clone();
    let advanced_autofill_for_combine = Rc::clone(&ui_handles.advanced_autofill);
    combine_rules_button.connect_clicked(move |_| {
        set_rule_bulk_buttons_sensitive(&group_button_for_combine, &combine_button_for_combine, false);
        status_for_combine.set_text(&tr("Combining compatible rules..."));

        let rules_list = rules_list_for_combine.clone();
        let rules_forms = Rc::clone(&rules_forms_for_combine);
        let rules_scroll = rules_scroll_for_combine.clone();
        let filter_entry = filter_entry_for_combine.clone();
        let status = status_for_combine.clone();
        let group_button = group_button_for_combine.clone();
        let combine_button = combine_button_for_combine.clone();
        let advanced_autofill = Rc::clone(&advanced_autofill_for_combine);
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(30), move || {
            let report = data::combine_editable_rules(&collect_rule_forms(
                &rules_forms.borrow(),
            ));
            if report.before_count == report.after_count {
                status.set_text(&tr(
                    "No adjacent compatible rules to combine. Use Group first if compatible rules are spread out.",
                ));
                set_rule_bulk_buttons_sensitive(&group_button, &combine_button, true);
                return;
            }

            let before_count = report.before_count;
            let after_count = report.after_count;
            let group_count = report.combined_groups;
            replace_rule_forms(
                &rules_list,
                &rules_forms,
                report.rules,
                &advanced_autofill,
                &filter_entry,
                &rules_scroll,
            );
            status.set_text(&trf(
                "Combined {before_count} rules into {after_count} rules across {group_count} group(s). Review, then Save.",
                &[
                    ("before_count", before_count.to_string()),
                    ("after_count", after_count.to_string()),
                    ("group_count", group_count.to_string()),
                ],
            ));
            set_rule_bulk_buttons_sensitive(&group_button, &combine_button, true);
        });
    });

    let management_dialog_for_budget_add = management_dialog.clone();
    let budgets_list_for_budget_add = budgets_list.clone();
    let budgets_forms_for_budget_add = Rc::clone(budgets_forms);
    let budgets_scroll_for_budget_add = budgets_scroll.clone();
    let status_for_budget_add = status.clone();
    let filter_entry_for_budget_add = filter_entry.clone();
    let advanced_autofill_for_budget_add = Rc::clone(&ui_handles.advanced_autofill);
    let ui_for_budget_add = Rc::clone(ui_handles);
    add_budget_button.connect_clicked(move |_| {
        show_new_budget_dialog(
            &management_dialog_for_budget_add,
            &budgets_list_for_budget_add,
            &budgets_forms_for_budget_add,
            &budgets_scroll_for_budget_add,
            &status_for_budget_add,
            &filter_entry_for_budget_add,
            &advanced_autofill_for_budget_add,
            ui_for_budget_add.advanced_features.get(),
        );
    });

    let management_dialog_for_budget_move = management_dialog.clone();
    let rules_forms_for_budget_move = Rc::clone(rules_forms);
    let budgets_forms_for_budget_move = Rc::clone(budgets_forms);
    let filter_entry_for_budget_move = filter_entry.clone();
    let status_for_budget_move = status.clone();
    let ui_for_budget_move = Rc::clone(ui_handles);
    move_budget_code_button.connect_clicked(move |_| {
        show_move_budget_code_dialog(
            &management_dialog_for_budget_move,
            &rules_forms_for_budget_move,
            &budgets_forms_for_budget_move,
            &filter_entry_for_budget_move,
            &status_for_budget_move,
            ui_for_budget_move.advanced_features.get(),
        );
    });

    let budgets_forms_for_real_income = Rc::clone(budgets_forms);
    let status_for_real_income = status.clone();
    use_real_income_button.connect_clicked(move |_| {
        let changed =
            set_budget_forms_income_basis(&budgets_forms_for_real_income.borrow(), "real");
        set_budget_bulk_status(
            &status_for_real_income,
            changed,
            0,
            "budget(s) set to real income basis",
        );
    });

    let budgets_forms_for_planned_income = Rc::clone(budgets_forms);
    let status_for_planned_income = status.clone();
    use_planned_income_button.connect_clicked(move |_| {
        let changed =
            set_budget_forms_income_basis(&budgets_forms_for_planned_income.borrow(), "planned");
        set_budget_bulk_status(
            &status_for_planned_income,
            changed,
            0,
            "budget(s) set to planned income basis",
        );
    });

    let budgets_forms_for_monthly_values = Rc::clone(budgets_forms);
    let status_for_monthly_values = status.clone();
    use_monthly_values_button.connect_clicked(move |_| {
        let result = set_budget_forms_value_period(
            &budgets_forms_for_monthly_values.borrow(),
            BudgetValuePeriod::Monthly,
        );
        set_budget_bulk_status(
            &status_for_monthly_values,
            result.changed,
            result.skipped,
            "budget(s) converted to monthly values",
        );
    });

    let budgets_forms_for_yearly_values = Rc::clone(budgets_forms);
    let status_for_yearly_values = status.clone();
    use_yearly_values_button.connect_clicked(move |_| {
        let result = set_budget_forms_value_period(
            &budgets_forms_for_yearly_values.borrow(),
            BudgetValuePeriod::Yearly,
        );
        set_budget_bulk_status(
            &status_for_yearly_values,
            result.changed,
            result.skipped,
            "budget(s) converted to yearly values",
        );
    });

    let management_dialog_for_alias_add = management_dialog.clone();
    let aliases_list_for_alias_add = aliases_list.clone();
    let aliases_forms_for_alias_add = Rc::clone(aliases_forms);
    let aliases_scroll_for_alias_add = aliases_scroll.clone();
    let status_for_alias_add = status.clone();
    let filter_entry_for_alias_add = filter_entry.clone();
    add_alias_button.connect_clicked(move |_| {
        show_new_alias_dialog(
            &management_dialog_for_alias_add,
            &aliases_list_for_alias_add,
            &aliases_forms_for_alias_add,
            &aliases_scroll_for_alias_add,
            &status_for_alias_add,
            &filter_entry_for_alias_add,
        );
    });

    let rules_forms_for_filter = Rc::clone(rules_forms);
    let budgets_forms_for_filter = Rc::clone(budgets_forms);
    let aliases_forms_for_filter = Rc::clone(aliases_forms);
    let status_for_filter = status.clone();
    filter_entry.connect_search_changed(move |entry| {
        apply_management_filter(
            &entry.text(),
            &rules_forms_for_filter.borrow(),
            &budgets_forms_for_filter.borrow(),
            &aliases_forms_for_filter.borrow(),
            &status_for_filter,
        );
    });

    let filter_search_bar_for_stop = filter_search_bar.clone();
    filter_entry.connect_stop_search(move |entry| {
        entry.set_text("");
        filter_search_bar_for_stop.set_search_mode(false);
    });

    let filter_entry_for_close = filter_entry.clone();
    filter_search_bar.connect_search_mode_enabled_notify(move |search_bar| {
        if !search_bar.is_search_mode() && !filter_entry_for_close.text().is_empty() {
            filter_entry_for_close.set_text("");
        }
    });

    let filter_search_bar_for_shortcut = filter_search_bar.clone();
    let filter_entry_for_shortcut = filter_entry.clone();
    let key_controller = gtk::EventControllerKey::new();
    key_controller.connect_key_pressed(move |_, key, _, modifier| {
        let is_find_shortcut = modifier.contains(gtk::gdk::ModifierType::CONTROL_MASK)
            && matches!(key.to_unicode(), Some('f') | Some('F'));
        if !is_find_shortcut {
            return gtk::glib::Propagation::Proceed;
        }

        if filter_search_bar_for_shortcut.is_search_mode() {
            filter_search_bar_for_shortcut.set_search_mode(false);
        } else {
            filter_search_bar_for_shortcut.set_search_mode(true);
            filter_entry_for_shortcut.grab_focus();
            filter_entry_for_shortcut.select_region(0, -1);
        }
        gtk::glib::Propagation::Stop
    });
    management_dialog.add_controller(key_controller);

    let management_dialog_for_save = management_dialog.clone();
    let rules_forms_for_save = Rc::clone(rules_forms);
    let budgets_forms_for_save = Rc::clone(budgets_forms);
    let aliases_forms_for_save = Rc::clone(aliases_forms);
    let state_for_save = Rc::clone(state);
    let ui_for_save = Rc::clone(ui_handles);
    let status_for_save = status.clone();
    let cancel_button_for_save = cancel_button.clone();
    let dialog_closed_for_save = Rc::clone(&dialog_closed);
    let save_running_for_save = Rc::clone(&save_running);
    let finish_for_save = Rc::clone(&finish_management_dialog);
    save_button.connect_clicked(move |button| {
        let mut direction_changes =
            collect_budget_direction_changes(&budgets_forms_for_save.borrow());
        direction_changes.extend(collect_rule_direction_changes(
            &rules_forms_for_save.borrow(),
        ));

        let management_dialog_for_confirm = management_dialog_for_save.clone();
        let rules_forms_for_save = Rc::clone(&rules_forms_for_save);
        let budgets_forms_for_save = Rc::clone(&budgets_forms_for_save);
        let aliases_forms_for_save = Rc::clone(&aliases_forms_for_save);
        let state_for_save = Rc::clone(&state_for_save);
        let ui_for_save = Rc::clone(&ui_for_save);
        let status_for_save = status_for_save.clone();
        let cancel_button = cancel_button_for_save.clone();
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
                let mode = state_for_save.borrow().dedupe_mode;
                let auto_clean_config = ui_for_save.preferences.auto_clean_config();
                let scope =
                    current_transaction_load_scope(&state_for_save.borrow(), ui_for_save.as_ref());
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
                cancel_button.set_label(&tr("Close"));
                status_for_save.set_text(&tr("Saving changes..."));
                show_status(&ui_for_save, "Saving changes...");

                gtk::glib::MainContext::default().spawn_local(async move {
                    let task = gtk::gio::spawn_blocking(move || {
                        data::write_editable_rules(&rules)?;
                        data::write_editable_budgets(&budgets)?;
                        data::write_editable_aliases(&aliases)?;
                        let new_data = data::load_app_data_with_config_cleanup(
                            mode,
                            auto_clean_config,
                            scope,
                        )?;
                        anyhow::Ok(new_data)
                    });

                    let mut save_succeeded = false;
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
                            save_succeeded = true;
                        }
                        Ok(Err(err)) => {
                            let message =
                                trf("Save failed: {error}", &[("error", format!("{err:#}"))]);
                            status_for_save.set_text(&message);
                            show_status(&ui_for_save, &message);
                        }
                        Err(_) => {
                            let message =
                                tr("Save canceled: the background task stopped unexpectedly.");
                            status_for_save.set_text(&message);
                            show_status(&ui_for_save, &message);
                        }
                    }
                    save_running_for_save.set(false);
                    if dialog_closed_for_save.get() {
                        finish_for_save();
                    } else {
                        button.set_sensitive(true);
                        cancel_button.set_label(&tr(if save_succeeded {
                            "Close"
                        } else {
                            "Cancel"
                        }));
                    }
                });
            },
        );
    });
}

fn connect_management_page_actions(
    page_actions_button: &gtk::MenuButton,
    stack: &adw::ViewStack,
    rules_forms: &Rc<RefCell<Vec<RuleForm>>>,
    budgets_forms: &Rc<RefCell<Vec<BudgetForm>>>,
    aliases_forms: &Rc<RefCell<Vec<AliasForm>>>,
    status: &gtk::Label,
    ui_handles: &Rc<UiHandles>,
) {
    let action_group = gtk::gio::SimpleActionGroup::new();

    let stack_for_copy = stack.clone();
    let rules_forms_for_copy = Rc::clone(rules_forms);
    let budgets_forms_for_copy = Rc::clone(budgets_forms);
    let aliases_forms_for_copy = Rc::clone(aliases_forms);
    let status_for_copy = status.clone();
    let ui_for_copy = Rc::clone(ui_handles);
    let copy_action = gtk::gio::SimpleAction::new("copy-page", None);
    copy_action.connect_activate(move |_, _| {
        match current_management_page_snapshot(
            &stack_for_copy,
            &rules_forms_for_copy.borrow(),
            &budgets_forms_for_copy.borrow(),
            &aliases_forms_for_copy.borrow(),
        ) {
            Ok(snapshot) => {
                ui_for_copy.window.clipboard().set_text(&snapshot.text);
                status_for_copy.set_text(&trf("Copied {page}.", &[("page", tr(&snapshot.title))]));
            }
            Err(err) => status_for_copy.set_text(&trf(
                "Copy failed: {error}",
                &[("error", format!("{err:#}"))],
            )),
        }
    });
    action_group.add_action(&copy_action);

    let stack_for_print = stack.clone();
    let rules_forms_for_print = Rc::clone(rules_forms);
    let budgets_forms_for_print = Rc::clone(budgets_forms);
    let aliases_forms_for_print = Rc::clone(aliases_forms);
    let status_for_print = status.clone();
    let ui_for_print = Rc::clone(ui_handles);
    let print_action = gtk::gio::SimpleAction::new("print-page", None);
    print_action.connect_activate(move |_, _| {
        match current_management_page_snapshot(
            &stack_for_print,
            &rules_forms_for_print.borrow(),
            &budgets_forms_for_print.borrow(),
            &aliases_forms_for_print.borrow(),
        ) {
            Ok(snapshot) => {
                status_for_print
                    .set_text(&trf("Printing {page}...", &[("page", tr(&snapshot.title))]));
                let report = table_print_report(
                    &snapshot.title,
                    &snapshot.subtitle,
                    &snapshot.columns,
                    &snapshot.rows,
                );
                print_report(&ui_for_print, report);
            }
            Err(err) => status_for_print.set_text(&trf(
                "Printing failed: {error}",
                &[("error", format!("{err:#}"))],
            )),
        }
    });
    action_group.add_action(&print_action);

    let stack_for_export = stack.clone();
    let rules_forms_for_export = Rc::clone(rules_forms);
    let budgets_forms_for_export = Rc::clone(budgets_forms);
    let aliases_forms_for_export = Rc::clone(aliases_forms);
    let status_for_export = status.clone();
    let export_action = gtk::gio::SimpleAction::new("export-csv", None);
    export_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        match current_management_page_snapshot(
            &stack_for_export,
            &rules_forms_for_export.borrow(),
            &budgets_forms_for_export.borrow(),
            &aliases_forms_for_export.borrow(),
        ) {
            Ok(snapshot) => export_management_snapshot(action, &status_for_export, snapshot),
            Err(err) => status_for_export.set_text(&trf(
                "Export error: {error}",
                &[("error", format!("{err:#}"))],
            )),
        }
    });
    action_group.add_action(&export_action);

    page_actions_button.insert_action_group("management", Some(&action_group));
}

#[derive(Clone)]
struct ManagementPageSnapshot {
    key: &'static str,
    title: String,
    subtitle: String,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    text: String,
    csv: String,
}

fn current_management_page_snapshot(
    stack: &adw::ViewStack,
    rules_forms: &[RuleForm],
    budgets_forms: &[BudgetForm],
    aliases_forms: &[AliasForm],
) -> anyhow::Result<ManagementPageSnapshot> {
    match stack.visible_child_name().as_deref() {
        Some("rules") => rules_management_snapshot(rules_forms),
        Some("aliases") => aliases_management_snapshot(aliases_forms),
        _ => budgets_management_snapshot(budgets_forms),
    }
}

fn rules_management_snapshot(forms: &[RuleForm]) -> anyhow::Result<ManagementPageSnapshot> {
    let rules = visible_collected_rules(forms);
    let columns = strings(&[
        "Active",
        "Priority",
        "Field",
        "Search",
        "Regex",
        "Category",
        "Budget Code",
        "Direction",
        "Min",
        "Max",
        "Notes",
    ]);
    let rows = rules
        .iter()
        .map(|rule| {
            vec![
                rule.active.to_string(),
                rule.priority.to_string(),
                rule.field.clone(),
                rule.search.clone(),
                rule.is_regex.to_string(),
                rule.category.clone(),
                rule.budget_code.clone(),
                rule.direction.clone(),
                rule.amount_min.clone(),
                rule.amount_max.clone(),
                rule.notes.clone(),
            ]
        })
        .collect::<Vec<_>>();
    management_page_snapshot(
        "rules",
        "Rules",
        "Categorization rules visible in the management window.",
        columns,
        rows,
        data::editable_rules_to_csv(&rules)?,
    )
}

fn budgets_management_snapshot(forms: &[BudgetForm]) -> anyhow::Result<ManagementPageSnapshot> {
    let budgets = visible_collected_budgets(forms);
    let columns = strings(&[
        "Code",
        "Category",
        "Monthly",
        "Yearly",
        "Direction",
        "Income basis",
        "Notes",
    ]);
    let rows = budgets
        .iter()
        .map(|budget| {
            vec![
                budget.code.clone(),
                budget.category.clone(),
                budget.monthly_budget.clone(),
                budget.yearly_budget.clone(),
                budget.direction.clone(),
                budget.income_basis.clone(),
                budget.notes.clone(),
            ]
        })
        .collect::<Vec<_>>();
    management_page_snapshot(
        "budgets",
        "Budgets",
        "Budgets visible in the management window.",
        columns,
        rows,
        data::editable_budgets_to_csv(&budgets)?,
    )
}

fn aliases_management_snapshot(forms: &[AliasForm]) -> anyhow::Result<ManagementPageSnapshot> {
    let aliases = visible_collected_aliases(forms);
    let columns = strings(&["Canonical", "Alias"]);
    let rows = aliases
        .iter()
        .map(|alias| vec![alias.canonical.clone(), alias.alias.clone()])
        .collect::<Vec<_>>();
    management_page_snapshot(
        "aliases",
        "Normalize",
        "Field names visible in the management window.",
        columns,
        rows,
        data::editable_aliases_to_csv(&aliases)?,
    )
}

fn management_page_snapshot(
    key: &'static str,
    title: &str,
    subtitle: &str,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    csv: String,
) -> anyhow::Result<ManagementPageSnapshot> {
    let text = management_page_text(title, subtitle, &columns, &rows);
    Ok(ManagementPageSnapshot {
        key,
        title: title.to_string(),
        subtitle: subtitle.to_string(),
        columns,
        rows,
        text,
        csv,
    })
}

fn visible_collected_rules(forms: &[RuleForm]) -> Vec<EditableRule> {
    let rules = collect_rule_forms(forms);
    forms
        .iter()
        .filter(|form| !form.deleted.get())
        .zip(rules)
        .filter(|(form, _)| form.form_box.is_visible())
        .map(|(_, rule)| rule)
        .collect()
}

fn visible_collected_budgets(forms: &[BudgetForm]) -> Vec<EditableBudget> {
    let budgets = collect_budget_forms(forms);
    forms
        .iter()
        .filter(|form| !form.deleted.get())
        .zip(budgets)
        .filter(|(form, _)| form.form_box.is_visible())
        .map(|(_, budget)| budget)
        .collect()
}

fn visible_collected_aliases(forms: &[AliasForm]) -> Vec<EditableAlias> {
    let aliases = collect_alias_forms(forms);
    forms
        .iter()
        .filter(|form| !form.deleted.get())
        .zip(aliases)
        .filter(|(form, _)| form.form_box.is_visible())
        .map(|(_, alias)| alias)
        .collect()
}

fn management_page_text(
    title: &str,
    subtitle: &str,
    columns: &[String],
    rows: &[Vec<String>],
) -> String {
    let mut lines = vec![tr(title), tr(subtitle), String::new()];
    lines.push(
        columns
            .iter()
            .map(|column| tr(column))
            .collect::<Vec<_>>()
            .join("\t"),
    );
    lines.extend(rows.iter().map(|row| {
        row.iter()
            .map(|value| compact_management_cell(value))
            .collect::<Vec<_>>()
            .join("\t")
    }));
    lines.join("\n")
}

fn compact_management_cell(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn export_management_snapshot(
    action: &gtk::gio::SimpleAction,
    status: &gtk::Label,
    snapshot: ManagementPageSnapshot,
) {
    action.set_enabled(false);
    status.set_text(&tr("Opening the file portal to save the CSV export..."));

    let action = action.clone();
    let status = status.clone();
    gtk::glib::MainContext::default().spawn_local(async move {
        let handle = rfd::AsyncFileDialog::new()
            .set_title(tr("Save CSV export"))
            .add_filter(tr("CSV files"), &["csv"])
            .set_file_name(management_export_file_name(snapshot.key))
            .save_file()
            .await;

        let Some(handle) = handle else {
            action.set_enabled(true);
            status.set_text(&tr("CSV export canceled."));
            return;
        };

        let path = handle.path().to_path_buf();
        let contents = snapshot.csv;
        status.set_text(&tr("Saving CSV export..."));
        let task = gtk::gio::spawn_blocking(move || {
            std::fs::write(&path, contents)?;
            anyhow::Ok(path)
        });
        match task.await {
            Ok(Ok(path)) => status.set_text(&trf(
                "Export saved: {path}",
                &[("path", path.display().to_string())],
            )),
            Ok(Err(err)) => status.set_text(&trf(
                "Export error: {error}",
                &[("error", format!("{err:#}"))],
            )),
            Err(_) => status.set_text(&tr(
                "CSV export canceled: the background task stopped unexpectedly.",
            )),
        }
        action.set_enabled(true);
    });
}

fn management_export_file_name(key: &str) -> String {
    format!(
        "bank_files_management_{key}_{}.csv",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    )
}

fn set_rule_bulk_buttons_sensitive(
    group_rules_button: &gtk::Button,
    combine_rules_button: &gtk::Button,
    sensitive: bool,
) {
    group_rules_button.set_sensitive(sensitive);
    combine_rules_button.set_sensitive(sensitive);
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct BudgetMoveTarget {
    code: String,
    category: String,
    direction: String,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
struct BudgetMoveResult {
    rules_changed: usize,
    source_budget_count: usize,
    budgets_removed: usize,
}

fn show_move_budget_code_dialog(
    parent: &adw::Dialog,
    rules_forms: &Rc<RefCell<Vec<RuleForm>>>,
    budgets_forms: &Rc<RefCell<Vec<BudgetForm>>>,
    filter_entry: &gtk::SearchEntry,
    status: &gtk::Label,
    advanced_features: bool,
) {
    let options = budget_move_targets(&budgets_forms.borrow());
    if options.len() < 2 {
        status.set_text(&tr("Need at least two budget codes to move rules."));
        return;
    }

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header =
        ui::cancelable_dialog_header("Move Budget Code", "Move rules to another budget code.");
    let cancel_button = gtk::Button::with_label(&tr("Cancel"));
    cancel_button.add_css_class("flat");
    let move_button = ui::primary_text_icon_button("send-to-symbolic", "Move", "Move budget code");
    header.pack_start(&cancel_button);
    header.pack_end(&move_button);
    root.append(&header);

    let page = ui::page_box();
    page.append(&ui::section_title(
        "Move Budget Code",
        "Move matching rules to the target budget code. Choose whether to keep the source budget code.",
    ));
    let grid = ui::form_grid();
    let source = budget_move_combo(&options, Some(&options[0].code));
    let target = budget_move_combo(&options, options.get(1).map(|option| option.code.as_str()));
    ui::add_labeled(&grid, 0, "From", &source);
    ui::add_labeled(&grid, 1, "To", &target);
    page.append(&grid);
    let keep_source = gtk::CheckButton::with_label(&tr("Keep old budget code"));
    keep_source.set_tooltip_text(Some(&tr(
        "Leave the source budget code in the budget list after moving rules",
    )));
    page.append(&keep_source);
    let dialog_status =
        ui::wrapped_label(&tr("Changes are staged here. Press Save to write them."));
    dialog_status.add_css_class("dim-label");
    page.append(&dialog_status);
    root.append(&ui::scroll(&page));

    let dialog = adw::Dialog::builder()
        .title(tr("Move Budget Code"))
        .content_width(680)
        .content_height(620)
        .default_widget(&move_button)
        .child(&root)
        .build();

    let dialog_for_cancel = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_for_cancel.close();
    });

    let dialog_for_move = dialog.clone();
    let rules_forms_for_move = Rc::clone(rules_forms);
    let budgets_forms_for_move = Rc::clone(budgets_forms);
    let filter_entry_for_move = filter_entry.clone();
    let status_for_move = status.clone();
    move_button.connect_clicked(move |_| {
        let from = combo_active_id(&source);
        let to = combo_active_id(&target);
        if from.trim().is_empty() || to.trim().is_empty() {
            dialog_status.set_text(&tr("Choose both budget codes first."));
            return;
        }
        if budget_code_matches(&from, &to) {
            dialog_status.set_text(&tr("Choose two different budget codes."));
            return;
        }
        if !advanced_features && budget_move_changes_direction(&options, &from, &to) {
            dialog_status.set_text(&tr(
                "This move changes direction. Enable Advanced Features to continue.",
            ));
            return;
        }

        let result = move_budget_code_between_forms(
            &rules_forms_for_move.borrow(),
            &budgets_forms_for_move.borrow(),
            &from,
            &to,
            !keep_source.is_active(),
        );
        filter_rule_forms(
            &filter_entry_for_move.text(),
            &rules_forms_for_move.borrow(),
        );
        filter_budget_forms(
            &filter_entry_for_move.text(),
            &budgets_forms_for_move.borrow(),
        );
        let message = move_budget_status_message(&from, &to, result);
        status_for_move.set_text(&message);
        dialog_for_move.close();
    });

    dialog.present(Some(parent));
}

fn budget_move_changes_direction(options: &[BudgetMoveTarget], from: &str, to: &str) -> bool {
    let Some(source) = budget_move_target_for_code(options, from) else {
        return false;
    };
    let Some(target) = budget_move_target_for_code(options, to) else {
        return false;
    };
    source.direction != target.direction
}

fn budget_move_target_for_code<'a>(
    options: &'a [BudgetMoveTarget],
    code: &str,
) -> Option<&'a BudgetMoveTarget> {
    options
        .iter()
        .find(|option| budget_code_matches(&option.code, code))
}

fn budget_move_combo(options: &[BudgetMoveTarget], active: Option<&str>) -> gtk::ComboBoxText {
    let combo = gtk::ComboBoxText::new();
    for option in options {
        combo.append(Some(&option.code), &budget_move_label(option));
    }
    if let Some(active) = active {
        combo.set_active_id(Some(active));
    }
    if combo.active_id().is_none() {
        combo.set_active(Some(0));
    }
    combo
}

fn budget_move_label(option: &BudgetMoveTarget) -> String {
    if option.category.trim().is_empty() {
        option.code.clone()
    } else {
        format!("{} · {}", option.code, option.category)
    }
}

fn budget_move_targets(forms: &[BudgetForm]) -> Vec<BudgetMoveTarget> {
    let mut options = Vec::new();
    for form in forms.iter().filter(|form| !form.deleted.get()) {
        let code = ui::combo_text(&form.code).trim().to_string();
        if code.is_empty()
            || planned_income::is_budget_code(&code)
            || options
                .iter()
                .any(|option: &BudgetMoveTarget| budget_code_matches(&option.code, &code))
        {
            continue;
        }
        options.push(BudgetMoveTarget {
            code,
            category: ui::combo_text(&form.category).trim().to_string(),
            direction: combo_active_id(&form.direction),
        });
    }
    options
}

fn move_budget_code_between_forms(
    rules: &[RuleForm],
    budgets: &[BudgetForm],
    from: &str,
    to: &str,
    remove_source_budget: bool,
) -> BudgetMoveResult {
    let Some(target) = find_budget_move_target(budgets, to) else {
        return BudgetMoveResult::default();
    };

    let mut result = BudgetMoveResult::default();
    for form in rules.iter().filter(|form| !form.deleted.get()) {
        if !budget_code_matches(&ui::combo_text(&form.budget_code), from) {
            continue;
        }
        set_text_combo(&form.budget_code, &target.code);
        set_text_combo(&form.category, &target.category);
        form.direction.set_active_id(Some(&target.direction));
        result.rules_changed += 1;
    }

    for form in budgets.iter().filter(|form| !form.deleted.get()) {
        if budget_code_matches(&ui::combo_text(&form.code), from) {
            result.source_budget_count += 1;
            if remove_source_budget {
                set_budget_form_deleted(form, true);
                result.budgets_removed += 1;
            }
        }
    }

    result
}

fn find_budget_move_target(forms: &[BudgetForm], code: &str) -> Option<BudgetMoveTarget> {
    budget_move_targets(forms)
        .into_iter()
        .find(|target| budget_code_matches(&target.code, code))
}

fn move_budget_status_message(from: &str, to: &str, result: BudgetMoveResult) -> String {
    if result.source_budget_count == 0 {
        return tr("No source budget code found to move.");
    }
    let kept_source = result.budgets_removed == 0;
    match (result.rules_changed, kept_source) {
        (0, true) => trf(
            "No rules used {from}; kept budget code {from}. Review, then Save.",
            &[("from", from.trim().to_string())],
        ),
        (0, false) => trf(
            "No rules used {from}; removed budget code {from}. Review, then Save.",
            &[("from", from.trim().to_string())],
        ),
        (_, true) => trf(
            "Moved {count} rule(s) from {from} to {to}. Kept {from}. Review, then Save.",
            &[
                ("count", result.rules_changed.to_string()),
                ("from", from.trim().to_string()),
                ("to", to.trim().to_string()),
            ],
        ),
        (_, false) => trf(
            "Moved {count} rule(s) from {from} to {to} and removed {from}. Review, then Save.",
            &[
                ("count", result.rules_changed.to_string()),
                ("from", from.trim().to_string()),
                ("to", to.trim().to_string()),
            ],
        ),
    }
}

fn budget_code_matches(left: &str, right: &str) -> bool {
    let left = left.trim();
    let right = right.trim();
    !left.is_empty() && left.eq_ignore_ascii_case(right)
}

fn replace_rule_forms(
    rules_list: &gtk::Box,
    rules_forms: &Rc<RefCell<Vec<RuleForm>>>,
    rules: Vec<EditableRule>,
    advanced_autofill: &Rc<Cell<bool>>,
    filter_entry: &gtk::SearchEntry,
    rules_scroll: &gtk::ScrolledWindow,
) {
    ui::clear_box(rules_list);
    rules_forms.borrow_mut().clear();
    for rule in rules {
        append_rule_form(rules_list, rules_forms, rule, true, advanced_autofill);
    }
    filter_rule_forms(&filter_entry.text(), &rules_forms.borrow());

    let rules_scroll = rules_scroll.clone();
    adw::glib::idle_add_local_once(move || {
        let adjustment = rules_scroll.vadjustment();
        adjustment.set_value(adjustment.lower());
    });
}

#[derive(Debug, Clone, Copy)]
enum BudgetValuePeriod {
    Monthly,
    Yearly,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
struct BudgetBulkResult {
    changed: usize,
    skipped: usize,
}

fn set_budget_forms_income_basis(forms: &[BudgetForm], basis: &str) -> usize {
    let mut changed = 0;
    for form in forms
        .iter()
        .filter(|form| !form.deleted.get() && !budget_form_is_planned_income(form))
    {
        let before = combo_active_id(&form.income_basis);
        form.income_basis.set_active_id(Some(basis));
        if form.income_basis.active_id().is_none() {
            form.income_basis.set_active(Some(0));
        }
        if before != combo_active_id(&form.income_basis) {
            changed += 1;
        }
    }
    changed
}

fn budget_form_is_planned_income(form: &BudgetForm) -> bool {
    planned_income::is_budget_code(&ui::combo_text(&form.code))
}

fn set_budget_forms_value_period(
    forms: &[BudgetForm],
    period: BudgetValuePeriod,
) -> BudgetBulkResult {
    let mut result = BudgetBulkResult::default();
    for form in forms.iter().filter(|form| !form.deleted.get()) {
        let monthly = form.monthly_budget.text().trim().to_string();
        let yearly = form.yearly_budget.text().trim().to_string();
        match budget_values_for_period(&monthly, &yearly, period) {
            BudgetValueUpdate::Changed { monthly, yearly } => {
                form.monthly_budget.set_text(&monthly);
                form.yearly_budget.set_text(&yearly);
                result.changed += 1;
            }
            BudgetValueUpdate::Unchanged => {}
            BudgetValueUpdate::Skipped => result.skipped += 1,
        }
    }
    result
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum BudgetValueUpdate {
    Changed { monthly: String, yearly: String },
    Unchanged,
    Skipped,
}

fn budget_values_for_period(
    monthly: &str,
    yearly: &str,
    period: BudgetValuePeriod,
) -> BudgetValueUpdate {
    let monthly = monthly.trim();
    let yearly = yearly.trim();
    let values = match period {
        BudgetValuePeriod::Monthly => match monthly_value_from(monthly, yearly) {
            Some(monthly) => Some((monthly, String::new())),
            None if monthly.is_empty() && !yearly.is_empty() => None,
            None => Some((String::new(), String::new())),
        },
        BudgetValuePeriod::Yearly => match yearly_value_from(monthly, yearly) {
            Some(yearly) => Some((String::new(), yearly)),
            None if yearly.is_empty() && !monthly.is_empty() => None,
            None => Some((String::new(), String::new())),
        },
    };

    let Some((new_monthly, new_yearly)) = values else {
        return BudgetValueUpdate::Skipped;
    };
    if new_monthly == monthly && new_yearly == yearly {
        BudgetValueUpdate::Unchanged
    } else {
        BudgetValueUpdate::Changed {
            monthly: new_monthly,
            yearly: new_yearly,
        }
    }
}

fn monthly_value_from(monthly: &str, yearly: &str) -> Option<String> {
    if !monthly.is_empty() {
        Some(monthly.to_string())
    } else if !yearly.is_empty() {
        convert_budget_amount_text(yearly, BudgetAmountConversion::YearlyToMonthly)
    } else {
        Some(String::new())
    }
}

fn yearly_value_from(monthly: &str, yearly: &str) -> Option<String> {
    if !yearly.is_empty() {
        Some(yearly.to_string())
    } else if !monthly.is_empty() {
        convert_budget_amount_text(monthly, BudgetAmountConversion::MonthlyToYearly)
    } else {
        Some(String::new())
    }
}

#[derive(Debug, Clone, Copy)]
enum BudgetAmountConversion {
    MonthlyToYearly,
    YearlyToMonthly,
}

fn convert_budget_amount_text(input: &str, conversion: BudgetAmountConversion) -> Option<String> {
    let amount = BudgetAmount::parse_optional(input)?;
    Some(match (amount, conversion) {
        (BudgetAmount::Fixed(amount), BudgetAmountConversion::MonthlyToYearly) => {
            format_budget_amount(BudgetAmount::Fixed(amount * Decimal::new(12, 0)))
        }
        (BudgetAmount::Fixed(amount), BudgetAmountConversion::YearlyToMonthly) => {
            format_budget_amount(BudgetAmount::Fixed(amount / Decimal::new(12, 0)))
        }
        (BudgetAmount::IncomePercent(percent), _) => {
            format_budget_amount(BudgetAmount::IncomePercent(percent))
        }
    })
}

fn format_budget_amount(amount: BudgetAmount) -> String {
    match amount {
        BudgetAmount::Fixed(amount) => amount.normalize().to_string(),
        BudgetAmount::IncomePercent(percent) => format!("{}%", percent.normalize()),
    }
}

fn set_budget_bulk_status(status: &gtk::Label, changed: usize, skipped: usize, action: &str) {
    let action = tr(action);
    let message = match (changed, skipped) {
        (0, 0) => tr("No budget rows changed."),
        (changed, 0) => trf(
            "Updated {count} {action}. Review, then Save.",
            &[("count", changed.to_string()), ("action", action.clone())],
        ),
        (changed, skipped) => trf(
            "Updated {count} {action}; skipped {skipped} invalid value(s). Review, then Save.",
            &[
                ("count", changed.to_string()),
                ("action", action.clone()),
                ("skipped", skipped.to_string()),
            ],
        ),
    };
    status.set_text(&message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_values_convert_monthly_fixed_values_to_yearly() {
        assert_eq!(
            budget_values_for_period("100", "", BudgetValuePeriod::Yearly),
            BudgetValueUpdate::Changed {
                monthly: String::new(),
                yearly: "1200".to_string(),
            }
        );
    }

    #[test]
    fn budget_values_convert_yearly_fixed_values_to_monthly() {
        assert_eq!(
            budget_values_for_period("", "1200", BudgetValuePeriod::Monthly),
            BudgetValueUpdate::Changed {
                monthly: "100".to_string(),
                yearly: String::new(),
            }
        );
    }

    #[test]
    fn budget_values_keep_percentages_when_switching_period() {
        assert_eq!(
            budget_values_for_period("10%", "", BudgetValuePeriod::Yearly),
            BudgetValueUpdate::Changed {
                monthly: String::new(),
                yearly: "10%".to_string(),
            }
        );
    }

    #[test]
    fn budget_values_skip_invalid_conversion_sources() {
        assert_eq!(
            budget_values_for_period("", "not money", BudgetValuePeriod::Monthly),
            BudgetValueUpdate::Skipped
        );
    }

    #[test]
    fn simple_budget_move_detects_direction_changes() {
        let options = vec![
            BudgetMoveTarget {
                code: "FOOD".to_string(),
                category: "Groceries".to_string(),
                direction: "expense".to_string(),
            },
            BudgetMoveTarget {
                code: "OTHER".to_string(),
                category: "Other".to_string(),
                direction: "expense".to_string(),
            },
            BudgetMoveTarget {
                code: "SALARY".to_string(),
                category: "Salary".to_string(),
                direction: "income".to_string(),
            },
        ];

        assert!(!budget_move_changes_direction(&options, "FOOD", "OTHER"));
        assert!(budget_move_changes_direction(&options, "FOOD", "SALARY"));
    }
}
