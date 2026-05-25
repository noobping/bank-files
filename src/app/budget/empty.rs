use super::*;

pub(in crate::app) fn append_empty_budget_action(
    container: &gtk::Box,
    data: &AppData,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    if !data.budgets.is_empty() {
        return;
    }

    let button = ui::primary_text_icon_button(
        "list-add-symbolic",
        empty_budget_action_label(ui_handles.as_ref()),
        empty_budget_action_tooltip(ui_handles.as_ref()),
    );
    button.set_halign(gtk::Align::Center);
    button.set_margin_top(6);
    button.set_margin_bottom(12);
    register_exclusive_config_widget(ui_handles, &button);

    let state_for_add = Rc::clone(state);
    let ui_for_add = Rc::clone(ui_handles);
    button.connect_clicked(move |_| {
        if config_operation_is_active(&ui_for_add, "Another edit or save is already running.") {
            return;
        }
        show_new_budget_dialog(&state_for_add, &ui_for_add);
    });

    container.append(&button);
}

fn empty_budget_action_label(ui_handles: &UiHandles) -> &'static str {
    if ui_handles.advanced_features.get() {
        "New Budget"
    } else {
        "New Category"
    }
}

fn empty_budget_action_tooltip(ui_handles: &UiHandles) -> &'static str {
    if ui_handles.advanced_features.get() {
        "Create a new budget"
    } else {
        "Create a new category with monthly or yearly amounts"
    }
}
