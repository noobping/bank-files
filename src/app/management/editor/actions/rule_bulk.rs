use super::*;

pub(super) fn connect_rule_bulk_actions(actions: &ManagementDialogActions<'_>) {
    let group_rules_button = actions.group_rules_button;
    let combine_rules_button = actions.combine_rules_button;
    let filter_entry = actions.filter_entry;
    let rules_list = actions.rules_list;
    let rules_forms = actions.rules_forms;
    let rules_scroll = actions.rules_scroll;
    let status = actions.status;
    let ui_handles = actions.ui_handles;

    let rules_list_for_group = rules_list.clone();
    let rules_forms_for_group = Rc::clone(rules_forms);
    let rules_scroll_for_group = rules_scroll.clone();
    let filter_entry_for_group = filter_entry.clone();
    let status_for_group = status.clone();
    let group_button_for_group = group_rules_button.clone();
    let combine_button_for_group = combine_rules_button.clone();
    let advanced_autofill_for_group = Rc::clone(&ui_handles.advanced_autofill);
    let ui_for_group = Rc::clone(ui_handles);
    group_rules_button.connect_clicked(move |_| {
    set_rule_bulk_buttons_sensitive(&group_button_for_group, &combine_button_for_group, false);
    status_for_group.set_text(&tr("Grouping compatible rules..."));
    show_verbose_status(
        ui_for_group.as_ref(),
        format!(
            "management rule grouping started; rules={}",
            rules_forms_for_group.borrow().len()
        ),
    );

    let rules_list = rules_list_for_group.clone();
    let rules_forms = Rc::clone(&rules_forms_for_group);
    let rules_scroll = rules_scroll_for_group.clone();
    let filter_entry = filter_entry_for_group.clone();
    let status = status_for_group.clone();
    let group_button = group_button_for_group.clone();
    let combine_button = combine_button_for_group.clone();
    let advanced_autofill = Rc::clone(&advanced_autofill_for_group);
    let ui = Rc::clone(&ui_for_group);
    gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(30), move || {
        let report = data::group_editable_rules_for_combining(&collect_rule_forms(
            &rules_forms.borrow(),
        ));
        show_verbose_status(
            ui.as_ref(),
            format!(
                "management rule grouping finished; changed={}; groups={}; rules={}",
                report.changed,
                report.grouped_groups,
                report.rules.len()
            ),
        );
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
    let ui_for_combine = Rc::clone(ui_handles);
    combine_rules_button.connect_clicked(move |_| {
    set_rule_bulk_buttons_sensitive(&group_button_for_combine, &combine_button_for_combine, false);
    status_for_combine.set_text(&tr("Combining compatible rules..."));
    show_verbose_status(
        ui_for_combine.as_ref(),
        format!(
            "management rule combine started; rules={}",
            rules_forms_for_combine.borrow().len()
        ),
    );

    let rules_list = rules_list_for_combine.clone();
    let rules_forms = Rc::clone(&rules_forms_for_combine);
    let rules_scroll = rules_scroll_for_combine.clone();
    let filter_entry = filter_entry_for_combine.clone();
    let status = status_for_combine.clone();
    let group_button = group_button_for_combine.clone();
    let combine_button = combine_button_for_combine.clone();
    let advanced_autofill = Rc::clone(&advanced_autofill_for_combine);
    let ui = Rc::clone(&ui_for_combine);
    gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(30), move || {
        let report = data::combine_editable_rules(&collect_rule_forms(
            &rules_forms.borrow(),
        ));
        show_verbose_status(
            ui.as_ref(),
            format!(
                "management rule combine finished; before={}; after={}; groups={}",
                report.before_count,
                report.after_count,
                report.combined_groups
            ),
        );
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
}

fn set_rule_bulk_buttons_sensitive(
    group_rules_button: &gtk::Button,
    combine_rules_button: &gtk::Button,
    sensitive: bool,
) {
    group_rules_button.set_sensitive(sensitive);
    combine_rules_button.set_sensitive(sensitive);
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
