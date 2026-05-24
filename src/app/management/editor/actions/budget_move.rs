use super::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct BudgetMoveTarget {
    pub(super) code: String,
    pub(super) category: String,
    pub(super) direction: String,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub(super) struct BudgetMoveResult {
    rules_changed: usize,
    source_budget_count: usize,
    budgets_removed: usize,
}

pub(super) fn show_move_budget_code_dialog(
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

    let shell = build_action_dialog_shell(
        "Move Budget Code",
        "Move matching rules to the target budget code. Choose whether to keep the source budget code.",
        "Move",
        "send-to-symbolic",
        "Move budget code",
        "Search budget codes",
    );
    shell.set_form_only();
    let move_button = shell.submit_button.clone();

    let page = ui::page_box();
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
    shell.add_form_page(&ui::action_dialog_scroll(&page));

    let dialog = ui::content_dialog(tr("Move Budget Code"), &shell.root)
        .content_width(MANAGEMENT_FORM_DIALOG_WIDTH)
        .default_widget(&move_button)
        .build();

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

pub(super) fn budget_move_changes_direction(
    options: &[BudgetMoveTarget],
    from: &str,
    to: &str,
) -> bool {
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
