use super::*;

pub(in crate::app) fn register_page_copy_feedback_button(ui: &UiHandles, button: &gtk::Button) {
    ui.page_copy_buttons.borrow_mut().push(button.clone());
}

pub(in crate::app) fn show_page_copy_feedback(ui: &UiHandles) {
    show_copy_feedback_for_buttons(&ui.page_copy_buttons, &ui.page_copy_feedback_generation);
}

fn show_copy_feedback_for_buttons(
    buttons: &Rc<RefCell<Vec<gtk::Button>>>,
    generation: &Rc<Cell<u64>>,
) {
    let current = generation.get().wrapping_add(1);
    generation.set(current);
    set_copy_feedback_icons(buttons, COPIED_ICON);

    let buttons = Rc::clone(buttons);
    let generation = Rc::clone(generation);
    gtk::glib::timeout_add_seconds_local(COPY_FEEDBACK_SECONDS, move || {
        if generation.get() == current {
            set_copy_feedback_icons(&buttons, COPY_ICON);
        }
        gtk::glib::ControlFlow::Break
    });
}

pub(super) fn show_copy_feedback(button: &gtk::Button, generation: &Rc<Cell<u64>>) {
    show_icon_feedback(button, generation, COPIED_ICON, COPY_ICON);
}

fn show_icon_feedback(
    button: &gtk::Button,
    generation: &Rc<Cell<u64>>,
    feedback_icon: &str,
    restore_icon: &str,
) {
    let current = generation.get().wrapping_add(1);
    generation.set(current);
    ui::set_button_icon(button, feedback_icon);

    let button = button.clone();
    let generation = Rc::clone(generation);
    let restore_icon = restore_icon.to_string();
    gtk::glib::timeout_add_seconds_local(COPY_FEEDBACK_SECONDS, move || {
        if generation.get() == current {
            ui::set_button_icon(&button, &restore_icon);
        }
        gtk::glib::ControlFlow::Break
    });
}

fn set_copy_feedback_icons(buttons: &Rc<RefCell<Vec<gtk::Button>>>, icon_name: &str) {
    let mut buttons = buttons.borrow_mut();
    buttons.retain(|button| button.root().is_some());
    for button in buttons.iter() {
        ui::set_button_icon(button, icon_name);
    }
}
