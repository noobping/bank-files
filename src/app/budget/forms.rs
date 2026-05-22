use super::*;

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
}
