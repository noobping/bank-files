use super::form::{
    connect_transaction_budget_move_form_save_sensitivity, TransactionBudgetMoveFormSensitivity,
};
use super::model::*;
use super::save::{connect_transaction_budget_move_save, TransactionBudgetMoveSave};
use super::*;
use crate::app::transactions::common::form_options::{budget_code_combo, category_combo};
use crate::app::transactions::common::rule_helpers::editable_rule_for_transaction;

pub(in crate::app::transactions::common) fn show_transaction_budget_code_dialog(
    tx: &Transaction,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let initial = {
        let data = state.borrow();
        editable_rule_for_transaction(tx, None, &data.budgets)
    };
    let advanced_features = ui_handles.advanced_features.get();

    let dialog_title = if advanced_features {
        "Move Budget Code"
    } else {
        "Move Category"
    };
    let dialog_subtitle = if advanced_features {
        "Move this transaction to a category, budget code, or direction."
    } else {
        "Move this transaction to a category with the same direction."
    };
    let match_name = truncate(&initial.search, 60);
    let header_title = transaction_budget_move_dialog_title(&match_name, dialog_title);
    let submit_tooltip = if advanced_features {
        "Save budget code move"
    } else {
        "Save category move"
    };
    let shell = build_action_dialog_shell(
        &header_title,
        dialog_subtitle,
        "Save",
        "document-save-symbolic",
        submit_tooltip,
        "Search categories",
    );
    register_config_widget(ui_handles, &shell.submit_button);
    shell.submit_button.set_sensitive(false);

    let match_summary = transaction_budget_move_match_summary(&initial);
    let budget_targets =
        transaction_budget_move_targets(tx, &state.borrow().budgets, advanced_features);

    let list_page = adw::PreferencesPage::new();
    let choices_group = adw::PreferencesGroup::new();
    let selected_target: Rc<RefCell<Option<TransactionBudgetTarget>>> = Rc::new(RefCell::new(None));
    let mut row_widgets = Vec::new();
    let mut search_rows = Vec::<SearchableActionRow>::new();

    for target in &budget_targets {
        let title = transaction_budget_target_title(target, advanced_features);
        let subtitle = transaction_budget_target_subtitle(target, advanced_features);
        let row = adw::ActionRow::builder()
            .title(title.as_str())
            .subtitle(subtitle.as_str())
            .build();
        row.set_title_lines(1);
        row.set_subtitle_lines(2);
        row.set_activatable(true);
        row.set_focusable(true);

        let check = gtk::Image::from_icon_name("object-select-symbolic");
        check.add_css_class("accent");
        check.set_visible(false);
        row.add_suffix(&check);

        choices_group.add(&row);
        search_rows.push(searchable_action_row(
            &row,
            &title,
            &subtitle,
            &transaction_budget_target_search_keywords(target, advanced_features, &match_summary),
        ));
        row_widgets.push(TransactionBudgetTargetRow {
            row,
            check,
            target: target.clone(),
        });
    }

    let empty_search = ui::wrapped_label(&tr("No matching categories."));
    empty_search.add_css_class("dim-label");
    empty_search.set_visible(false);
    choices_group.add(&empty_search);
    list_page.add(&choices_group);

    if transaction_budget_more_options_visible(advanced_features) {
        let advanced_group = adw::PreferencesGroup::new();
        let more_row = adw::ActionRow::builder()
            .title(tr("More Options"))
            .subtitle(tr(
                "Edit the category, budget code, direction, and confirmation details.",
            ))
            .build();
        more_row.set_activatable(true);
        more_row.add_prefix(&gtk::Image::from_icon_name("emblem-system-symbolic"));
        advanced_group.add(&more_row);
        list_page.add(&advanced_group);

        let shell_for_more = shell.page_handle();
        more_row.connect_activated(move |_| {
            shell_for_more.set_form_page();
        });
    }

    let status = ui::wrapped_label("");
    status.add_css_class("dim-label");
    status.set_visible(false);
    choices_group.add(&status);

    let rows = Rc::new(row_widgets);
    for index in 0..rows.len() {
        let row = rows[index].row.clone();
        let rows_for_click = Rc::clone(&rows);
        let selected_for_click = Rc::clone(&selected_target);
        let save_for_click = shell.submit_button.clone();
        let tx_for_click = tx.clone();
        let click = gtk::GestureClick::new();
        click.set_button(0);
        click.connect_pressed(move |_, n_press, _, _| {
            select_transaction_budget_target_row(
                rows_for_click.as_ref(),
                &selected_for_click,
                &save_for_click,
                &tx_for_click,
                advanced_features,
                index,
            );
            if n_press >= 2 && save_for_click.is_sensitive() {
                save_for_click.emit_clicked();
            }
        });
        row.add_controller(click);

        let rows_for_key = Rc::clone(&rows);
        let selected_for_key = Rc::clone(&selected_target);
        let save_for_key = shell.submit_button.clone();
        let tx_for_key = tx.clone();
        let key = gtk::EventControllerKey::new();
        key.connect_key_pressed(move |_, key, _, _| {
            let activates = matches!(
                key,
                gtk::gdk::Key::Return | gtk::gdk::Key::KP_Enter | gtk::gdk::Key::space
            );
            if !activates {
                return gtk::glib::Propagation::Proceed;
            }
            select_transaction_budget_target_row(
                rows_for_key.as_ref(),
                &selected_for_key,
                &save_for_key,
                &tx_for_key,
                advanced_features,
                index,
            );
            if save_for_key.is_sensitive() {
                save_for_key.emit_clicked();
            }
            gtk::glib::Propagation::Stop
        });
        row.add_controller(key);
    }

    let list_max_height = transaction_budget_move_list_max_height(&ui_handles.window);
    let list_min_height = transaction_budget_move_list_min_height(rows.len()).min(list_max_height);
    shell.add_list_page(&ui::action_dialog_scroll_with_limits(
        &list_page,
        list_min_height,
        list_max_height,
    ));
    if let Some(selected_index) = rows
        .iter()
        .position(|row| transaction_budget_target_is_current(tx, &row.target, advanced_features))
    {
        select_transaction_budget_target_row(
            rows.as_ref(),
            &selected_target,
            &shell.submit_button,
            tx,
            advanced_features,
            selected_index,
        );
    }
    connect_action_search(
        &shell.search_entry,
        search_rows,
        Some(empty_search.clone().upcast::<gtk::Widget>()),
    );

    let mut form_category: Option<gtk::ComboBoxText> = None;
    let mut form_budget_code: Option<gtk::ComboBoxText> = None;
    let mut form_direction: Option<gtk::ComboBoxText> = None;
    let mut form_status: Option<gtk::Label> = None;

    if advanced_features {
        let form_page = ui::page_box();
        let form = ui::form_box();
        ui::add_labeled_stacked(&form, "Rule match", &ui::wrapped_label(&match_summary));

        let category = category_combo(&state.borrow(), &initial.category);
        let budget_code = budget_code_combo(&state.borrow(), &initial.budget_code);
        let direction = ui::combo_from_options(
            &[
                ("expense", "Expenses"),
                ("income", "Income"),
                ("transfer", "Transfers"),
            ],
            &initial.direction,
        );
        connect_budget_fields_autofill(
            &category,
            &budget_code,
            &direction,
            app_budget_autofill_entries(&state.borrow()),
            &ui_handles.advanced_autofill,
        );
        connect_transaction_budget_move_form_save_sensitivity(
            TransactionBudgetMoveFormSensitivity {
                stack: &shell.stack,
                save_button: &shell.submit_button,
                tx,
                selected_target: &selected_target,
                initial: &initial,
                category: &category,
                budget_code: &budget_code,
                direction: &direction,
                advanced_features,
            },
        );
        ui::focus_button_after_combo_selections(
            &shell.submit_button,
            &[&category, &budget_code, &direction],
        );
        ui::add_labeled_stacked(&form, "Category", &category);
        ui::add_labeled_stacked(&form, "Budget code", &budget_code);
        ui::add_labeled_stacked(&form, "Direction", &direction);
        form_page.append(&form);

        let advanced_status = ui::wrapped_label(&tr(
            "Save adds a categorization rule to the processing queue. The original bank CSV is not changed.",
        ));
        advanced_status.add_css_class("dim-label");
        form_page.append(&advanced_status);
        shell.add_form_page(&ui::action_dialog_scroll(&form_page));

        form_category = Some(category);
        form_budget_code = Some(budget_code);
        form_direction = Some(direction);
        form_status = Some(advanced_status);
    }

    shell.set_list_page();

    let dialog = ui::content_dialog(tr(dialog_title), &shell.root)
        .content_width(680)
        .default_widget(&shell.submit_button)
        .build();
    ui::bind_search_bar(&shell.root, &dialog, &shell.search_bar, &shell.search_entry);

    let shell_for_back = shell.page_handle();
    shell.back_button.connect_clicked(move |_| {
        shell_for_back.set_list_page();
    });

    connect_transaction_budget_move_save(
        &shell.submit_button,
        TransactionBudgetMoveSave {
            state: Rc::clone(state),
            ui_handles: Rc::clone(ui_handles),
            dialog: dialog.clone(),
            tx: tx.clone(),
            initial: initial.clone(),
            selected_target: Rc::clone(&selected_target),
            stack: shell.stack.clone(),
            list_status: status.clone(),
            form_status,
            form_category,
            form_budget_code,
            form_direction,
            advanced_features,
        },
    );

    dialog.present(Some(&ui_handles.window));
}
