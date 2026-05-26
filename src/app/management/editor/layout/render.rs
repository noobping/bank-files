use super::super::*;
use super::loading::{
    ManagementFormsLoad, ManagementFormsRender, ManagementFormsRenderStage, ManagementLoadedForms,
};
use super::MANAGEMENT_FORM_RENDER_BATCH_SIZE;

pub(super) fn start_management_forms_render(
    load: ManagementFormsLoad,
    loaded: ManagementLoadedForms,
) {
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
                    render.stage = ManagementFormsRenderStage::Budgets;
                    return true;
                };
                append_rule_form(
                    &render.load.rules_list,
                    &render.load.rules_forms,
                    rule,
                    true,
                    &render.load.advanced_autofill,
                    render.load.advanced_features,
                );
                *remaining -= 1;
            }
            false
        }
        Err(err) => {
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
                    render.stage = ManagementFormsRenderStage::Done;
                    return true;
                };
                append_alias_form(&render.load.aliases_list, &render.load.aliases_forms, alias);
                *remaining -= 1;
            }
            false
        }
        Err(err) => {
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
    apply_management_filter(
        &load.filter_entry.text(),
        &load.rules_forms.borrow(),
        &load.budgets_forms.borrow(),
        &load.aliases_forms.borrow(),
        &load.status,
    );
    set_management_form_action_widgets_sensitive(&load.action_widgets, true);
    set_management_menu_actions_enabled(&load.menu_actions, true);
    set_special_budget_add_actions_enabled(&load.menu_actions, &load.budgets_forms.borrow());
    for widget in &load.action_widgets {
        register_loading_sensitive_widget(&load.ui_handles, widget);
    }
    load.page_actions_button.set_sensitive(true);
    register_loading_sensitive_widget(&load.ui_handles, &load.page_actions_button);
    load.status_handle.set_loading(false);
}

fn set_special_budget_add_actions_enabled(
    actions: &[gtk::gio::SimpleAction],
    forms: &[BudgetForm],
) {
    for action in actions {
        let Some(special) = special_budget_kind_for_action(action) else {
            continue;
        };
        action.set_enabled(!budget_forms_contain_special(forms, special));
    }
}

fn special_budget_kind_for_action(
    action: &gtk::gio::SimpleAction,
) -> Option<crate::model::BudgetSpecialKind> {
    match action.name().as_str() {
        "add-planned-income-budget" => Some(crate::model::BudgetSpecialKind::PlannedIncome),
        "add-transfer-budget" => Some(crate::model::BudgetSpecialKind::Transfer),
        "add-refunding-budget" => Some(crate::model::BudgetSpecialKind::Refunding),
        "add-refunded-budget" => Some(crate::model::BudgetSpecialKind::Refunded),
        _ => None,
    }
}

fn budget_forms_contain_special(
    forms: &[BudgetForm],
    special: crate::model::BudgetSpecialKind,
) -> bool {
    forms.iter().filter(|form| !form.deleted.get()).any(|form| {
        crate::model::budget_special_kind_for_config(&form.special, &ui::combo_text(&form.code))
            == special
    })
}

pub(super) fn set_management_form_action_widgets_sensitive(
    widgets: &[gtk::Widget],
    sensitive: bool,
) {
    for widget in widgets {
        widget.set_sensitive(sensitive);
    }
}

pub(super) fn set_management_menu_actions_enabled(
    actions: &[gtk::gio::SimpleAction],
    enabled: bool,
) {
    for action in actions {
        action.set_enabled(enabled);
    }
}
