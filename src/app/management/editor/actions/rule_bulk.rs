use super::*;
use std::collections::HashSet;

#[derive(Clone)]
struct RuleBulkActions {
    group_rules: gtk::gio::SimpleAction,
    combine_rules: gtk::gio::SimpleAction,
    clean_orphaned_rules: gtk::gio::SimpleAction,
}

pub(super) fn connect_rule_bulk_actions(actions: &ManagementDialogActions<'_>) {
    let bulk_actions = RuleBulkActions {
        group_rules: actions.group_rules_action.clone(),
        combine_rules: actions.combine_rules_action.clone(),
        clean_orphaned_rules: actions.clean_orphaned_rules_action.clone(),
    };
    connect_group_rules_action(actions, &bulk_actions);
    connect_combine_rules_action(actions, &bulk_actions);
    connect_clean_orphaned_rules_action(actions, &bulk_actions);
}

fn connect_group_rules_action(
    actions: &ManagementDialogActions<'_>,
    bulk_actions: &RuleBulkActions,
) {
    let rules_list = actions.rules_list.clone();
    let rules_forms = Rc::clone(actions.rules_forms);
    let rules_scroll = actions.rules_scroll.clone();
    let filter_entry = actions.filter_entry.clone();
    let status = actions.status.clone();
    let advanced_autofill = Rc::clone(&actions.ui_handles.advanced_autofill);
    let bulk_actions_for_group = bulk_actions.clone();

    bulk_actions.group_rules.connect_activate(move |_, _| {
        set_rule_bulk_actions_enabled(&bulk_actions_for_group, false);
        status.set_text(&tr("Grouping compatible rules..."));

        let rules_list = rules_list.clone();
        let rules_forms = Rc::clone(&rules_forms);
        let rules_scroll = rules_scroll.clone();
        let filter_entry = filter_entry.clone();
        let status = status.clone();
        let advanced_autofill = Rc::clone(&advanced_autofill);
        let bulk_actions = bulk_actions_for_group.clone();
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(30), move || {
            let report = data::group_editable_rules_for_combining(&collect_rule_forms(
                &rules_forms.borrow(),
            ));
            if report.grouped_groups == 0 {
                status.set_text(&tr("No compatible rules to group."));
                set_rule_bulk_actions_enabled(&bulk_actions, true);
                return;
            }
            if !report.changed {
                status.set_text(&tr("Compatible rules are already grouped. Use Combine."));
                set_rule_bulk_actions_enabled(&bulk_actions, true);
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
            set_rule_bulk_actions_enabled(&bulk_actions, true);
        });
    });
}

fn connect_combine_rules_action(
    actions: &ManagementDialogActions<'_>,
    bulk_actions: &RuleBulkActions,
) {
    let rules_list = actions.rules_list.clone();
    let rules_forms = Rc::clone(actions.rules_forms);
    let rules_scroll = actions.rules_scroll.clone();
    let filter_entry = actions.filter_entry.clone();
    let status = actions.status.clone();
    let advanced_autofill = Rc::clone(&actions.ui_handles.advanced_autofill);
    let bulk_actions_for_combine = bulk_actions.clone();

    bulk_actions.combine_rules.connect_activate(move |_, _| {
        set_rule_bulk_actions_enabled(&bulk_actions_for_combine, false);
        status.set_text(&tr("Combining compatible rules..."));

        let rules_list = rules_list.clone();
        let rules_forms = Rc::clone(&rules_forms);
        let rules_scroll = rules_scroll.clone();
        let filter_entry = filter_entry.clone();
        let status = status.clone();
        let advanced_autofill = Rc::clone(&advanced_autofill);
        let bulk_actions = bulk_actions_for_combine.clone();
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(30), move || {
            let report = data::combine_editable_rules(&collect_rule_forms(
                &rules_forms.borrow(),
            ));
            if report.before_count == report.after_count {
                status.set_text(&tr(
                    "No adjacent compatible rules to combine. Use Group first if compatible rules are spread out.",
                ));
                set_rule_bulk_actions_enabled(&bulk_actions, true);
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
            set_rule_bulk_actions_enabled(&bulk_actions, true);
        });
    });
}

fn connect_clean_orphaned_rules_action(
    actions: &ManagementDialogActions<'_>,
    bulk_actions: &RuleBulkActions,
) {
    let rules_forms = Rc::clone(actions.rules_forms);
    let budgets_forms = Rc::clone(actions.budgets_forms);
    let filter_entry = actions.filter_entry.clone();
    let status = actions.status.clone();
    let bulk_actions_for_clean = bulk_actions.clone();

    bulk_actions
        .clean_orphaned_rules
        .connect_activate(move |_, _| {
            set_rule_bulk_actions_enabled(&bulk_actions_for_clean, false);
            status.set_text(&tr("Cleaning orphaned rules..."));

            let removed =
                mark_orphaned_rule_forms_deleted(&rules_forms.borrow(), &budgets_forms.borrow());
            filter_rule_forms(&filter_entry.text(), &rules_forms.borrow());

            if removed == 0 {
                status.set_text(&tr("No orphaned rules to clean."));
            } else {
                status.set_text(&trf(
                    "Cleaned {count} orphaned rule(s). Review, then Save.",
                    &[("count", removed.to_string())],
                ));
            }
            set_rule_bulk_actions_enabled(&bulk_actions_for_clean, true);
        });
}

fn set_rule_bulk_actions_enabled(actions: &RuleBulkActions, enabled: bool) {
    actions.group_rules.set_enabled(enabled);
    actions.combine_rules.set_enabled(enabled);
    actions.clean_orphaned_rules.set_enabled(enabled);
}

fn mark_orphaned_rule_forms_deleted(rules: &[RuleForm], budgets: &[BudgetForm]) -> usize {
    let budget_codes = active_budget_code_keys(budgets);
    let mut removed = 0;
    for form in rules.iter().filter(|form| !form.deleted.get()) {
        if budget_code_is_orphaned(&ui::combo_text(&form.budget_code), &budget_codes) {
            form.deleted.set(true);
            form.form_box.set_visible(false);
            removed += 1;
        }
    }
    removed
}

fn active_budget_code_keys(forms: &[BudgetForm]) -> HashSet<String> {
    forms
        .iter()
        .filter(|form| !form.deleted.get())
        .map(|form| crate::util::normalize_key(&ui::combo_text(&form.code)))
        .filter(|code| !code.is_empty())
        .collect()
}

fn budget_code_is_orphaned(budget_code: &str, budget_codes: &HashSet<String>) -> bool {
    let code = budget_code.trim();
    !code.is_empty() && !budget_codes.contains(&crate::util::normalize_key(code))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orphan_detection_ignores_empty_and_known_budget_codes() {
        let budget_codes = HashSet::from([
            crate::util::normalize_key("Food"),
            crate::util::normalize_key("INC-OTHER"),
        ]);

        assert!(!budget_code_is_orphaned("", &budget_codes));
        assert!(!budget_code_is_orphaned(" food ", &budget_codes));
        assert!(!budget_code_is_orphaned("inc other", &budget_codes));
        assert!(budget_code_is_orphaned("ghost", &budget_codes));
    }
}
