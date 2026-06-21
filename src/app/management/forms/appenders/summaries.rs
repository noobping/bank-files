use super::super::*;
use super::state::{combo_display_text, entry_summary, entry_summary_fixed_budget};

pub(super) struct RuleSummaryWidgets<'a> {
    pub(super) active: &'a gtk::Switch,
    pub(super) priority: &'a gtk::SpinButton,
    pub(super) field: &'a gtk::ComboBoxText,
    pub(super) search: &'a gtk::TextView,
    pub(super) is_regex: &'a gtk::Switch,
    pub(super) category: &'a gtk::ComboBoxText,
    pub(super) budget_code: &'a gtk::ComboBoxText,
    pub(super) direction: &'a gtk::ComboBoxText,
    pub(super) amount_min: &'a gtk::Entry,
    pub(super) amount_max: &'a gtk::Entry,
    pub(super) advanced_features: bool,
}

pub(super) fn rule_summary(widgets: RuleSummaryWidgets<'_>) -> (String, String) {
    if !widgets.advanced_features {
        return simple_rule_summary(widgets);
    }

    let title = format!(
        "{} · {}",
        entry_summary_text(&ui::combo_text(widgets.category), "Uncategorized"),
        entry_summary_text(&ui::combo_text(widgets.budget_code), "No budget code")
    );
    let match_kind = tr(if widgets.is_regex.is_active() {
        "regex"
    } else {
        "text"
    });
    let state = tr(if widgets.active.is_active() {
        "active"
    } else {
        "inactive"
    });
    let mut parts = vec![
        format!(
            "{}: {}",
            combo_display_text(widgets.field),
            rule_search_text(widgets.search)
        ),
        combo_display_text(widgets.direction),
        format!(
            "{state} · {} {} · {match_kind}",
            tr("priority"),
            widgets.priority.value_as_int()
        ),
    ];
    let min = widgets.amount_min.text().trim().to_string();
    let max = widgets.amount_max.text().trim().to_string();
    if !min.is_empty() || !max.is_empty() {
        parts.push(format!("{} {min}..{max}", tr("amount")));
    }
    (title, parts.join(" · "))
}

fn simple_rule_summary(widgets: RuleSummaryWidgets<'_>) -> (String, String) {
    let mut parts = vec![rule_search_text(widgets.search)];
    if !widgets.active.is_active() {
        parts.push(tr("inactive"));
    }
    (
        entry_summary_text(&ui::combo_text(widgets.category), "Uncategorized"),
        parts
            .into_iter()
            .filter(|part| !part.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" · "),
    )
}

pub(super) struct BudgetSummaryWidgets<'a> {
    pub(super) code: &'a gtk::ComboBoxText,
    pub(super) parent_code: &'a gtk::ComboBoxText,
    pub(super) category: &'a gtk::ComboBoxText,
    pub(super) monthly_budget: &'a gtk::Entry,
    pub(super) yearly_budget: &'a gtk::Entry,
    pub(super) direction: &'a gtk::ComboBoxText,
    pub(super) income_basis: &'a gtk::ComboBoxText,
    pub(super) special: &'a str,
    pub(super) show_code: bool,
}

pub(super) fn budget_summary(widgets: BudgetSummaryWidgets<'_>) -> (String, String) {
    let code_text = ui::combo_text(widgets.code);
    let category_text = ui::combo_text(widgets.category);
    let special = crate::model::budget_special_kind_for_config(widgets.special, &code_text);
    let planned_income = special.is_planned_income() || planned_income::is_budget_code(&code_text);
    let title = if widgets.show_code {
        format!(
            "{} · {}",
            entry_summary_text(&code_text, "No code"),
            entry_summary_text(&category_text, "Uncategorized")
        )
    } else {
        entry_summary_text(&category_text, "Uncategorized")
    };
    let mut parts = vec![
        combo_display_text(widgets.direction),
        format!(
            "{} {}",
            tr("monthly"),
            if planned_income {
                entry_summary_fixed_budget(widgets.monthly_budget, "-")
            } else {
                entry_summary(widgets.monthly_budget, "-")
            }
        ),
        format!(
            "{} {}",
            tr("yearly"),
            if planned_income {
                entry_summary_fixed_budget(widgets.yearly_budget, "-")
            } else {
                entry_summary(widgets.yearly_budget, "-")
            }
        ),
    ];
    if !planned_income
        && budget_values_use_percentage(
            &widgets.monthly_budget.text(),
            &widgets.yearly_budget.text(),
        )
    {
        parts.push(combo_display_text(widgets.income_basis));
    }
    let parent_code = combo_active_id(widgets.parent_code);
    if !parent_code.trim().is_empty() {
        parts.push(format!(
            "{} {}",
            tr("parent"),
            combo_display_text(widgets.parent_code)
        ));
    }
    (title, parts.join(" · "))
}

pub(super) fn alias_summary(canonical: &gtk::ComboBoxText, alias: &gtk::Entry) -> (String, String) {
    let alias = entry_summary(alias, "No bank column");
    let canonical = combo_display_text(canonical);
    (
        format!("{alias} · {canonical}"),
        format!("{alias} -> {canonical}"),
    )
}

fn entry_summary_text(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.trim().to_string()
    }
}
