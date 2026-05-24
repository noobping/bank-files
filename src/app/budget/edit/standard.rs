use super::*;

pub(in crate::app) fn budget_edit_button(
    code: &str,
    category: &str,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) -> gtk::Button {
    let button = ui::icon_button("document-edit-symbolic", "Edit budget");
    button.add_css_class("flat");
    button.set_halign(gtk::Align::End);
    button.set_valign(gtk::Align::Center);
    button.set_sensitive(!code.trim().is_empty());
    register_exclusive_config_widget(ui_handles, &button);

    let code = code.trim().to_string();
    let category = category.trim().to_string();
    let ui_for_edit = Rc::clone(ui_handles);
    let state_for_edit = Rc::clone(state);
    button.connect_clicked(move |_| {
        if config_operation_is_active(&ui_for_edit, "Another edit or save is already running.") {
            return;
        }
        show_budget_edit_dialog(&code, &category, &state_for_edit, &ui_for_edit);
    });

    button
}

pub(in crate::app) fn show_budget_edit_dialog(
    code: &str,
    fallback_category: &str,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let (initial, can_delete_budget) = editable_budget_for(code, fallback_category);
    if planned_income::is_budget_code(&initial.code) {
        show_planned_income_budget_edit_dialog(initial, can_delete_budget, state, ui_handles);
        return;
    }

    let header = ui::cancelable_dialog_header("Edit Budget", code);

    let delete_button = ui::icon_button("user-trash-symbolic", "Delete budget");
    delete_button.add_css_class("destructive-action");
    delete_button.set_sensitive(can_delete_budget);
    let save_button = ui::primary_text_icon_button("document-save-symbolic", "Save", "Save budget");
    register_exclusive_config_widget(ui_handles, &save_button);
    register_exclusive_config_widget(ui_handles, &delete_button);
    header.pack_start(&delete_button);
    header.pack_end(&save_button);

    let page = ui::page_box();
    page.append(&ui::section_title(
        "Edit Budget",
        "Use monthly budgets, yearly budgets, or both. Percentage budgets can use real or planned income.",
    ));

    let grid = ui::form_grid();
    let advanced_features = ui_handles.advanced_features.get();
    if advanced_features {
        ui::add_labeled(&grid, 0, "Budget code", &ui::wrapped_label(&initial.code));
    }
    let category = ui::text_combo(&initial.category, app_category_values(&state.borrow()));
    let monthly_budget = ui::entry(&initial.monthly_budget, "500 or 10% of income");
    let yearly_budget = ui::entry(&initial.yearly_budget, "5000 or 10% of yearly income");
    let direction = ui::combo_from_options(
        &[
            ("expense", "Expenses"),
            ("income", "Income"),
            ("transfer", "Transfers"),
        ],
        ui::budget_direction_id(&initial.direction),
    );
    let income_basis = ui::combo_from_options(
        &[("real", "Real income"), ("planned", "Planned income")],
        ui::budget_income_basis_id(&initial.income_basis),
    );
    if !budget_direction_editable(advanced_features, can_delete_budget) {
        direction.set_sensitive(false);
    }
    let notes = ui::entry(&initial.notes, "Note");
    let first_row = if advanced_features { 1 } else { 0 };
    ui::add_labeled(&grid, first_row, "Category", &category);
    ui::add_labeled(&grid, first_row + 1, "Monthly budget", &monthly_budget);
    ui::add_labeled(&grid, first_row + 2, "Yearly budget", &yearly_budget);
    ui::add_labeled(&grid, first_row + 3, "Direction", &direction);
    let income_basis_label =
        ui::add_labeled(&grid, first_row + 4, "Percentage basis", &income_basis);
    ui::add_labeled(&grid, first_row + 5, "Note", &notes);
    bind_percentage_basis_visibility(
        &monthly_budget,
        &yearly_budget,
        &income_basis_label,
        &income_basis,
    );
    page.append(&grid);

    let status = ui::wrapped_label(&tr("Changes are saved to your budget configuration."));
    status.add_css_class("dim-label");
    page.append(&status);
    let content = ui::action_dialog_scroll(&page);
    let view = ui::dialog_toolbar_view(&header, &content);

    let dialog = ui::content_dialog(tr("Edit Budget"), &view)
        .content_width(620)
        .default_widget(&save_button)
        .build();

    let state_for_save = Rc::clone(state);
    let ui_for_save = Rc::clone(ui_handles);
    let dialog_for_save = dialog.clone();
    let code_for_save = initial.code.clone();
    let category_for_save = initial.category.clone();
    let direction_for_save = initial.direction.clone();
    let status_for_save = status.clone();
    let delete_button_for_save = delete_button.clone();
    save_button.connect_clicked(move |button| {
        let category_text = ui::combo_text(&category);
        if category_text.is_empty() {
            status_for_save.set_text(&tr("Enter a category first."));
            category.grab_focus();
            return;
        }

        let budget = EditableBudget {
            code: code_for_save.clone(),
            category: category_text,
            monthly_budget: monthly_budget.text().trim().to_string(),
            yearly_budget: yearly_budget.text().trim().to_string(),
            direction: ui::combo_active_id(&direction),
            income_basis: ui::combo_active_id(&income_basis),
            notes: notes.text().trim().to_string(),
        };

        let direction_changes = if can_delete_budget {
            let from =
                BudgetDirection::parse(&direction_for_save, &code_for_save, &category_for_save);
            let to = BudgetDirection::parse(&budget.direction, &budget.code, &budget.category);
            budget_direction_change(&budget.code, from, to)
                .into_iter()
                .collect()
        } else {
            Vec::new()
        };

        let save_ui = BudgetSaveUi {
            button: button.clone(),
            delete_button: Some(delete_button_for_save.clone()),
            delete_button_sensitive: can_delete_budget,
            status: status_for_save.clone(),
            dialog: dialog_for_save.clone(),
        };
        let state_for_save = Rc::clone(&state_for_save);
        let ui_for_save = Rc::clone(&ui_for_save);
        let dialog_for_confirm = save_ui.dialog.clone();
        confirm_budget_direction_changes(&dialog_for_confirm, direction_changes, move || {
            save_budget_with_reload(budget, save_ui, state_for_save, ui_for_save);
        });
    });

    connect_budget_delete_action(BudgetDeleteAction {
        delete_button: &delete_button,
        save_button: &save_button,
        status: &status,
        dialog: &dialog,
        code: initial.code.clone(),
        state: Rc::clone(state),
        ui_handles: Rc::clone(ui_handles),
    });

    dialog.present(Some(&ui_handles.window));
}
