use super::super::*;
use super::loading::{
    management_loaded_forms_summary, ManagementFormsLoad, ManagementFormsRender,
    ManagementFormsRenderStage, ManagementLoadedForms,
};
use super::MANAGEMENT_FORM_RENDER_BATCH_SIZE;

pub(super) fn start_management_forms_render(
    load: ManagementFormsLoad,
    loaded: ManagementLoadedForms,
) {
    show_verbose_status(
        load.ui_handles.as_ref(),
        format!(
            "management forms render started; batch_size={MANAGEMENT_FORM_RENDER_BATCH_SIZE}; {}",
            management_loaded_forms_summary(&loaded)
        ),
    );
    ui::clear_box(&load.rules_list);
    load.rules_forms.borrow_mut().clear();
    ui::clear_box(&load.budgets_list);
    load.budgets_forms.borrow_mut().clear();
    ui::clear_box(&load.aliases_list);
    load.aliases_forms.borrow_mut().clear();
    load.status_handle
        .set_text(&tr("Loading management data..."));
    let render = Rc::new(RefCell::new(ManagementFormsRender {
        load,
        loaded,
        stage: ManagementFormsRenderStage::Rules,
    }));
    schedule_management_forms_render(render);
}

fn schedule_management_forms_render(render: Rc<RefCell<ManagementFormsRender>>) {
    gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(1), move || {
        if render.borrow().load.dialog_closed.get() {
            return;
        }
        if render_management_forms_batch(&render) {
            schedule_management_forms_render(render);
        }
    });
}

fn render_management_forms_batch(render: &Rc<RefCell<ManagementFormsRender>>) -> bool {
    let mut render = render.borrow_mut();
    let mut remaining = MANAGEMENT_FORM_RENDER_BATCH_SIZE;
    while remaining > 0 {
        match render.stage {
            ManagementFormsRenderStage::Rules => {
                if render_rule_forms_batch(&mut render, &mut remaining) {
                    continue;
                }
            }
            ManagementFormsRenderStage::Budgets => {
                if render_budget_forms_batch(&mut render, &mut remaining) {
                    continue;
                }
            }
            ManagementFormsRenderStage::Aliases => {
                if render_alias_forms_batch(&mut render, &mut remaining) {
                    continue;
                }
            }
            ManagementFormsRenderStage::Done => {
                finish_management_forms_render(&render.load);
                return false;
            }
        }
    }
    true
}

fn render_rule_forms_batch(render: &mut ManagementFormsRender, remaining: &mut usize) -> bool {
    match &mut render.loaded.rules {
        Ok(rules) => {
            while *remaining > 0 {
                let Some(rule) = rules.pop_front() else {
                    show_verbose_status(
                        render.load.ui_handles.as_ref(),
                        format!(
                            "management rules render finished; forms={}",
                            render.load.rules_forms.borrow().len()
                        ),
                    );
                    render.stage = ManagementFormsRenderStage::Budgets;
                    return true;
                };
                append_rule_form(
                    &render.load.rules_list,
                    &render.load.rules_forms,
                    rule,
                    true,
                    &render.load.advanced_autofill,
                );
                *remaining -= 1;
            }
            false
        }
        Err(err) => {
            show_verbose_status(
                render.load.ui_handles.as_ref(),
                format!("management rules render failed; error={err}"),
            );
            render
                .load
                .rules_list
                .append(&ui::selectable_wrapped_label(&trf(
                    "Could not read rules: {error}",
                    &[("error", err.clone())],
                )));
            render.stage = ManagementFormsRenderStage::Budgets;
            true
        }
    }
}

fn render_budget_forms_batch(render: &mut ManagementFormsRender, remaining: &mut usize) -> bool {
    match &mut render.loaded.budgets {
        Ok(budgets) => {
            while *remaining > 0 {
                let Some(budget) = budgets.pop_front() else {
                    show_verbose_status(
                        render.load.ui_handles.as_ref(),
                        format!(
                            "management budgets render finished; forms={}",
                            render.load.budgets_forms.borrow().len()
                        ),
                    );
                    render.stage = ManagementFormsRenderStage::Aliases;
                    return true;
                };
                if planned_income::is_budget_code(&budget.code) {
                    append_planned_income_budget_form(
                        &render.load.budgets_list,
                        &render.load.budgets_forms,
                        budget,
                        render.load.advanced_features,
                    );
                } else {
                    append_budget_form(
                        &render.load.budgets_list,
                        &render.load.budgets_forms,
                        budget,
                        true,
                        &render.load.advanced_autofill,
                        render.load.advanced_features,
                    );
                }
                *remaining -= 1;
            }
            false
        }
        Err(err) => {
            show_verbose_status(
                render.load.ui_handles.as_ref(),
                format!("management budgets render failed; error={err}"),
            );
            let message = if render.load.advanced_features {
                "Could not read budget codes: {error}"
            } else {
                "Could not read budgets: {error}"
            };
            render
                .load
                .budgets_list
                .append(&ui::selectable_wrapped_label(&trf(
                    message,
                    &[("error", err.clone())],
                )));
            render.stage = ManagementFormsRenderStage::Aliases;
            true
        }
    }
}

fn render_alias_forms_batch(render: &mut ManagementFormsRender, remaining: &mut usize) -> bool {
    match &mut render.loaded.aliases {
        Ok(aliases) => {
            while *remaining > 0 {
                let Some(alias) = aliases.pop_front() else {
                    show_verbose_status(
                        render.load.ui_handles.as_ref(),
                        format!(
                            "management aliases render finished; forms={}",
                            render.load.aliases_forms.borrow().len()
                        ),
                    );
                    render.stage = ManagementFormsRenderStage::Done;
                    return true;
                };
                append_alias_form(&render.load.aliases_list, &render.load.aliases_forms, alias);
                *remaining -= 1;
            }
            false
        }
        Err(err) => {
            show_verbose_status(
                render.load.ui_handles.as_ref(),
                format!("management aliases render failed; error={err}"),
            );
            render
                .load
                .aliases_list
                .append(&ui::selectable_wrapped_label(&trf(
                    "Could not read field names: {error}",
                    &[("error", err.clone())],
                )));
            render.stage = ManagementFormsRenderStage::Done;
            true
        }
    }
}

fn finish_management_forms_render(load: &ManagementFormsLoad) {
    show_verbose_status(
        load.ui_handles.as_ref(),
        format!(
            "management forms render finished; rules={}; budgets={}; aliases={}",
            load.rules_forms.borrow().len(),
            load.budgets_forms.borrow().len(),
            load.aliases_forms.borrow().len()
        ),
    );
    apply_management_filter(
        &load.filter_entry.text(),
        &load.rules_forms.borrow(),
        &load.budgets_forms.borrow(),
        &load.aliases_forms.borrow(),
        &load.status,
    );
    set_management_form_action_buttons_sensitive(&load.buttons, true);
    for button in &load.buttons {
        register_loading_sensitive_widget(&load.ui_handles, button);
    }
    load.page_actions_button.set_sensitive(true);
    register_loading_sensitive_widget(&load.ui_handles, &load.page_actions_button);
    load.status_handle.set_loading(false);
}

pub(super) fn set_management_form_action_buttons_sensitive(
    buttons: &[gtk::Button],
    sensitive: bool,
) {
    for button in buttons {
        button.set_sensitive(sensitive);
    }
}
