use super::*;

struct ParentBudgetOption {
    code: String,
    label: String,
}

pub(in crate::app) fn app_budget_parent_combo(
    data: &AppData,
    selected: &str,
    current_code: &str,
    advanced_features: bool,
) -> gtk::ComboBoxText {
    let options = data.budgets.iter().filter_map(|budget| {
        parent_option(
            &budget.code,
            &budget.category,
            budget.special == crate::model::BudgetSpecialKind::None,
            current_code,
            advanced_features,
        )
    });
    budget_parent_combo(selected, options)
}

pub(in crate::app) fn editable_budget_parent_combo(
    selected: &str,
    current_code: &str,
    advanced_features: bool,
) -> gtk::ComboBoxText {
    let budgets = data::load_editable_budgets().unwrap_or_default();
    let options = budgets.iter().filter_map(|budget| {
        let normal_budget =
            crate::model::budget_special_kind_for_config(&budget.special, &budget.code)
                == crate::model::BudgetSpecialKind::None;
        parent_option(
            &budget.code,
            &budget.category,
            normal_budget,
            current_code,
            advanced_features,
        )
    });
    budget_parent_combo(selected, options)
}

fn parent_option(
    code: &str,
    category: &str,
    normal_budget: bool,
    current_code: &str,
    advanced_features: bool,
) -> Option<ParentBudgetOption> {
    let code = code.trim();
    if !normal_budget || code.is_empty() || code.eq_ignore_ascii_case(current_code.trim()) {
        return None;
    }

    Some(ParentBudgetOption {
        code: code.to_string(),
        label: budget_display_title(code, category, advanced_features),
    })
}

fn budget_parent_combo(
    selected: &str,
    options: impl IntoIterator<Item = ParentBudgetOption>,
) -> gtk::ComboBoxText {
    let combo = gtk::ComboBoxText::new();
    combo.append(None::<&str>, &tr("No parent budget"));

    let mut options = options.into_iter().collect::<Vec<_>>();
    options.sort_by_key(|option| option.label.to_ascii_uppercase());
    options.dedup_by(|left, right| left.code.eq_ignore_ascii_case(&right.code));

    for option in options {
        combo.append(Some(&option.code), &option.label);
    }

    let selected = selected.trim();
    if selected.is_empty() {
        combo.set_active(Some(0));
    } else {
        combo.set_active_id(Some(selected));
        if combo.active_id().is_none() {
            combo.append(Some(selected), selected);
            combo.set_active_id(Some(selected));
        }
    }

    combo
}

pub(in crate::app) fn bind_percentage_basis_visibility(
    monthly_budget: &gtk::Entry,
    yearly_budget: &gtk::Entry,
    income_basis_label: &gtk::Label,
    income_basis: &gtk::ComboBoxText,
) {
    let update_visibility: Rc<dyn Fn()> = {
        let monthly_budget = monthly_budget.clone();
        let yearly_budget = yearly_budget.clone();
        let income_basis_label = income_basis_label.clone();
        let income_basis = income_basis.clone();
        Rc::new(move || {
            let visible =
                budget_values_use_percentage(&monthly_budget.text(), &yearly_budget.text());
            income_basis_label.set_visible(visible);
            income_basis.set_visible(visible);
        })
    };

    connect_budget_value_visibility_update(monthly_budget, &update_visibility);
    connect_budget_value_visibility_update(yearly_budget, &update_visibility);
    update_visibility();
}

fn connect_budget_value_visibility_update(entry: &gtk::Entry, update: &Rc<dyn Fn()>) {
    let update = Rc::clone(update);
    entry.connect_changed(move |_| update());
}

pub(in crate::app) fn budget_values_use_percentage(
    monthly_budget: &str,
    yearly_budget: &str,
) -> bool {
    budget_value_uses_percentage(monthly_budget) || budget_value_uses_percentage(yearly_budget)
}

fn budget_value_uses_percentage(value: &str) -> bool {
    value.contains('%')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_values_use_percentage_when_either_value_contains_percent() {
        assert!(budget_values_use_percentage("10% of income", ""));
        assert!(budget_values_use_percentage(
            "500",
            "12.5% of yearly income"
        ));
        assert!(!budget_values_use_percentage("500", "6000"));
        assert!(!budget_values_use_percentage("", ""));
    }

    #[test]
    fn parent_option_uses_mode_labels_and_filters_invalid_targets() {
        let simple = parent_option("HOME", "Housing", true, "RENT", false).unwrap();
        let advanced = parent_option("HOME", "Housing", true, "RENT", true).unwrap();

        assert_eq!(simple.label, "Housing");
        assert_eq!(advanced.label, "HOME · Housing");
        assert!(parent_option("RENT", "Rent", true, "RENT", false).is_none());
        assert!(parent_option("TRANSFER", "Transfer", false, "RENT", false).is_none());
        assert!(parent_option("", "Empty", true, "RENT", false).is_none());
    }
}
