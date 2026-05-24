use super::data::TransactionPatternInfo;
use super::dialog::show_transaction_pattern_rule_dialog;
use super::*;

pub(super) fn append_transaction_pattern_rows(
    container: &gtk::Box,
    patterns: &[(analytics::TransactionPattern, TransactionPatternInfo)],
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    for (pattern, info) in patterns {
        let filter = TransactionFilter::Pattern(pattern.clone());
        let state_for_pattern = Rc::clone(state);
        let ui_for_pattern = Rc::clone(ui_handles);
        let card = transaction_pattern_card(pattern, &info.badges, move || {
            show_transactions_filter(&state_for_pattern, &ui_for_pattern, filter.clone());
        });
        container.append(&transaction_pattern_edit_row(
            card,
            pattern,
            info.hidden,
            state,
            ui_handles,
        ));
    }
}

pub(super) fn append_transaction_patterns_more_button(
    section: &gtk::Box,
    rows_box: &gtk::Box,
    patterns: Vec<(analytics::TransactionPattern, TransactionPatternInfo)>,
    hidden_button: Option<&gtk::Button>,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    let more_button = ui::plain_text_icon_button(
        "view-more-symbolic",
        "More",
        "Show all transaction patterns",
    );
    let rows_box = rows_box.clone();
    let state_for_more = Rc::clone(state);
    let ui_for_more = Rc::clone(ui_handles);
    let hidden_button = hidden_button.cloned();
    more_button.connect_clicked(move |button| {
        ui::clear_box(&rows_box);
        append_transaction_pattern_rows(&rows_box, &patterns, &state_for_more, &ui_for_more);
        if let Some(hidden_button) = &hidden_button {
            hidden_button.set_visible(true);
        }
        button.set_visible(false);
    });
    section.append(&more_button);
}

pub(super) fn transaction_patterns_hidden_button(
    rows_box: &gtk::Box,
    patterns: Vec<(analytics::TransactionPattern, TransactionPatternInfo)>,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> gtk::Button {
    let button = ui::plain_text_icon_button(
        "view-reveal-symbolic",
        "Show Hidden",
        "Show hidden transaction patterns",
    );
    let rows_box = rows_box.clone();
    let state_for_hidden = Rc::clone(state);
    let ui_for_hidden = Rc::clone(ui_handles);
    button.connect_clicked(move |button| {
        ui::clear_box(&rows_box);
        append_transaction_pattern_rows(&rows_box, &patterns, &state_for_hidden, &ui_for_hidden);
        button.set_visible(false);
    });
    button
}

fn transaction_pattern_edit_row(
    card: gtk::Box,
    pattern: &analytics::TransactionPattern,
    hidden: bool,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> gtk::Overlay {
    card.set_hexpand(true);
    if let Some(content) = card
        .first_child()
        .and_then(|child| child.downcast::<gtk::Box>().ok())
    {
        content.set_margin_end(content.margin_end() + 84);
    }

    let overlay = gtk::Overlay::new();
    overlay.set_hexpand(true);
    overlay.set_child(Some(&card));

    let actions = ui::linked_button_group();
    actions.set_halign(gtk::Align::End);
    actions.set_valign(gtk::Align::Center);
    actions.set_margin_end(12);

    let visibility_button = if hidden {
        ui::icon_button("edit-undo-symbolic", "Reopen detected pattern")
    } else {
        ui::icon_button("window-close-symbolic", "Hide detected pattern")
    };
    visibility_button.add_css_class("flat");
    register_config_widget(ui_handles, &visibility_button);

    actions.append(&visibility_button);
    let edit_button = if ui_handles.advanced_features.get() {
        let edit_button = ui::icon_button("document-edit-symbolic", "Create rule from pattern");
        edit_button.add_css_class("flat");
        register_config_widget(ui_handles, &edit_button);
        actions.append(&edit_button);
        Some(edit_button)
    } else {
        None
    };

    let pattern_for_visibility = pattern.clone();
    let state_for_visibility = Rc::clone(state);
    let ui_for_visibility = Rc::clone(ui_handles);
    visibility_button.connect_clicked(move |_| {
        toggle_transaction_pattern_visibility(
            &pattern_for_visibility,
            hidden,
            &state_for_visibility,
            &ui_for_visibility,
        );
    });

    if let Some(edit_button) = edit_button {
        let pattern = pattern.clone();
        let state_for_edit = Rc::clone(state);
        let ui_for_edit = Rc::clone(ui_handles);
        edit_button.connect_clicked(move |_| {
            show_transaction_pattern_rule_dialog(&pattern, &state_for_edit, &ui_for_edit);
        });
    }

    overlay.add_overlay(&actions);
    overlay
}

fn toggle_transaction_pattern_visibility(
    pattern: &analytics::TransactionPattern,
    hidden: bool,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) {
    if config_operation_is_active(ui_handles, "Another edit or save is already running.") {
        return;
    }

    let key = analytics::transaction_pattern_key(pattern);
    let result = if hidden {
        crate::data::reopen_transaction_pattern(&key)
    } else {
        crate::data::ignore_transaction_pattern(&key, &pattern.label)
    };

    match result {
        Ok(_) => {
            show_status(
                ui_handles,
                if hidden {
                    "Pattern reopened."
                } else {
                    "Pattern hidden. Use Show Hidden to reopen it."
                },
            );
            render_views(&state.borrow(), ui_handles, state);
        }
        Err(error) => show_status(
            ui_handles,
            &trf(
                "Could not update hidden pattern: {error}",
                &[("error", format!("{error:#}"))],
            ),
        ),
    }
}
