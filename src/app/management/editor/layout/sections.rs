use super::super::*;

pub(super) struct BudgetActionSection {
    pub(super) container: adw::WrapBox,
    pub(super) add_budget_button: gtk::Button,
    pub(super) move_budget_code_button: gtk::Button,
    pub(super) use_real_income_button: gtk::Button,
    pub(super) use_planned_income_button: gtk::Button,
    pub(super) use_monthly_values_button: gtk::Button,
    pub(super) use_yearly_values_button: gtk::Button,
}

pub(super) fn build_budget_action_section(advanced_features: bool) -> BudgetActionSection {
    let add_budget_button = ui::plain_text_icon_button(
        "list-add-symbolic",
        if advanced_features {
            "New Budget"
        } else {
            "New Category"
        },
        if advanced_features {
            "Create a new budget"
        } else {
            "Create a new category with monthly or yearly amounts"
        },
    );
    let move_budget_code_button = ui::plain_text_icon_button(
        "send-to-symbolic",
        "Move Code",
        "Move rules from one budget code to another",
    );
    let use_real_income_button = ui::plain_text_icon_button(
        "view-refresh-symbolic",
        "Real Income",
        "Set every budget percentage basis to real income",
    );
    let use_planned_income_button = ui::plain_text_icon_button(
        "view-refresh-symbolic",
        "Planned Income",
        "Set every budget percentage basis to planned income",
    );
    let use_monthly_values_button = ui::plain_text_icon_button(
        "go-previous-symbolic",
        "Monthly",
        "Convert budget values to monthly values",
    );
    let use_yearly_values_button = ui::plain_text_icon_button(
        "go-next-symbolic",
        "Yearly",
        "Convert budget values to yearly values",
    );

    let container = adw::WrapBox::builder()
        .orientation(gtk::Orientation::Horizontal)
        .child_spacing(8)
        .child_spacing_unit(adw::LengthUnit::Px)
        .line_spacing(6)
        .line_spacing_unit(adw::LengthUnit::Px)
        .natural_line_length(720)
        .natural_line_length_unit(adw::LengthUnit::Px)
        .wrap_policy(adw::WrapPolicy::Natural)
        .hexpand(true)
        .halign(gtk::Align::Fill)
        .build();

    let budget_create_actions = ui::linked_button_group();
    budget_create_actions.append(&add_budget_button);
    if advanced_features {
        budget_create_actions.append(&move_budget_code_button);
    }
    container.append(&budget_create_actions);

    if advanced_features {
        let budget_income_actions = ui::linked_button_group();
        budget_income_actions.append(&use_real_income_button);
        budget_income_actions.append(&use_planned_income_button);
        let budget_value_actions = ui::linked_button_group();
        budget_value_actions.append(&use_monthly_values_button);
        budget_value_actions.append(&use_yearly_values_button);
        container.append(&budget_income_actions);
        container.append(&budget_value_actions);
    }

    BudgetActionSection {
        container,
        add_budget_button,
        move_budget_code_button,
        use_real_income_button,
        use_planned_income_button,
        use_monthly_values_button,
        use_yearly_values_button,
    }
}
