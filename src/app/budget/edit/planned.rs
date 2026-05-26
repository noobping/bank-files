use super::*;

pub(in crate::app::budget::edit) fn show_planned_income_budget_edit_dialog(
    initial: EditableBudget,
    can_delete_budget: bool,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let advanced_features = ui_handles.advanced_features.get();
    let budget_code = initial.code.clone();
    let header = ui::cancelable_dialog_header(
        "Edit Planned Income",
        if advanced_features {
            budget_code.as_str()
        } else {
            "Planned income"
        },
    );

    let delete_button = ui::icon_button("user-trash-symbolic", "Delete planned income budget");
    delete_button.add_css_class("destructive-action");
    delete_button.set_sensitive(can_delete_budget);
    let save_button = ui::primary_text_icon_button(
        "document-save-symbolic",
        "Save",
        "Save planned income budget",
    );
    register_exclusive_config_widget(ui_handles, &save_button);
    register_exclusive_config_widget(ui_handles, &delete_button);
    header.pack_start(&delete_button);
    header.pack_end(&save_button);

    let page = ui::page_box();
    page.append(&ui::section_title(
        "Planned Income",
        if advanced_features {
            "Set fixed monthly and yearly planned income for this budget code."
        } else {
            "Set fixed monthly and yearly planned income."
        },
    ));
    let reminder = ui::wrapped_label(&tr(planned_income::NET_INCOME_REMINDER));
    reminder.add_css_class("dim-label");
    page.append(&reminder);

    let grid = ui::form_grid();
    if advanced_features {
        ui::add_labeled(&grid, 0, "Budget code", &ui::wrapped_label(&budget_code));
    }
    let category = ui::text_combo(&initial.category, app_category_values(&state.borrow()));
    let monthly_budget = ui::entry(
        &planned_income::fixed_budget_amount_text(&initial.monthly_budget),
        "500",
    );
    let yearly_budget = ui::entry(
        &planned_income::fixed_budget_amount_text(&initial.yearly_budget),
        "5000",
    );
    planned_income::connect_fixed_budget_entry(&monthly_budget);
    planned_income::connect_fixed_budget_entry(&yearly_budget);
    let notes = ui::entry(&initial.notes, "Note");
    let first_row = if advanced_features { 1 } else { 0 };
    ui::add_labeled(&grid, first_row, "Category", &category);
    ui::add_labeled(
        &grid,
        first_row + 1,
        "Monthly planned income",
        &monthly_budget,
    );
    ui::add_labeled(
        &grid,
        first_row + 2,
        "Yearly planned income",
        &yearly_budget,
    );
    ui::add_labeled(&grid, first_row + 3, "Note", &notes);
    page.append(&grid);

    let status = ui::wrapped_label(&tr("Changes are saved to your budget configuration."));
    status.add_css_class("dim-label");
    page.append(&status);
    let content = ui::action_dialog_scroll(&page);
    let view = ui::dialog_toolbar_view(&header, &content);

    let dialog = ui::content_dialog(tr("Edit Planned Income"), &view)
        .content_width(620)
        .default_widget(&save_button)
        .build();

    let state_for_save = Rc::clone(state);
    let ui_for_save = Rc::clone(ui_handles);
    let dialog_for_save = dialog.clone();
    let status_for_save = status.clone();
    let delete_button_for_save = delete_button.clone();
    let budget_code_for_save = budget_code.clone();
    save_button.connect_clicked(move |button| {
        let category_text = ui::combo_text(&category);
        if category_text.is_empty() {
            status_for_save.set_text(&tr("Enter a category first."));
            category.grab_focus();
            return;
        }

        let mut budget = planned_income::editable_budget(
            category_text,
            monthly_budget.text().trim().to_string(),
            yearly_budget.text().trim().to_string(),
            notes.text().trim().to_string(),
        );
        budget.code = budget_code_for_save.clone();
        budget.special = crate::model::BudgetSpecialKind::PlannedIncome
            .as_config()
            .to_string();
        save_budget_with_reload(
            budget,
            BudgetSaveUi {
                button: button.clone(),
                delete_button: Some(delete_button_for_save.clone()),
                delete_button_sensitive: can_delete_budget,
                status: status_for_save.clone(),
                dialog: dialog_for_save.clone(),
            },
            Rc::clone(&state_for_save),
            Rc::clone(&ui_for_save),
        );
    });

    connect_budget_delete_action(BudgetDeleteAction {
        delete_button: &delete_button,
        save_button: &save_button,
        status: &status,
        dialog: &dialog,
        code: budget_code,
        state: Rc::clone(state),
        ui_handles: Rc::clone(ui_handles),
    });

    dialog.present(Some(&ui_handles.window));
}
