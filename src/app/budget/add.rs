use super::*;

pub(in crate::app) fn budget_add_action(
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) -> gtk::Button {
    let button = ui::plain_text_icon_button(
        "list-add-symbolic",
        budget_add_action_label(ui_handles.as_ref()),
        budget_add_action_tooltip(ui_handles.as_ref()),
    );
    register_exclusive_config_widget(ui_handles, &button);

    let state_for_add = Rc::clone(state);
    let ui_for_add = Rc::clone(ui_handles);
    button.connect_clicked(move |_| {
        if config_operation_is_active(&ui_for_add, "Another edit or save is already running.") {
            return;
        }
        show_new_budget_dialog(&state_for_add, &ui_for_add);
    });

    button
}

fn budget_add_action_label(ui_handles: &UiHandles) -> &'static str {
    if ui_handles.advanced_features.get() {
        "New Budget"
    } else {
        "New Category"
    }
}

fn budget_add_action_tooltip(ui_handles: &UiHandles) -> &'static str {
    if ui_handles.advanced_features.get() {
        "Create a new budget"
    } else {
        "Create a new category with monthly or yearly amounts"
    }
}
