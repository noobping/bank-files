use super::*;

pub(super) fn append_orphaned_config_section(
    search: Option<&SearchFilter>,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> bool {
    let orphaned_rules = match data::orphaned_rules() {
        Ok(rules) => rules,
        Err(error) => {
            if search
                .map(|filter| filter.matches("orphaned configuration missing budget codes rules"))
                .unwrap_or(true)
            {
                ui_handles.debug.append(&ui::section_title(
                    "Orphaned Configuration",
                    "Rules that point to missing budget codes can create ghost categories or budget codes.",
                ));
                ui_handles.debug.append(&ui::text_card(&trf(
                    "Could not inspect configuration: {error}",
                    &[("error", format!("{error:#}"))],
                )));
                return true;
            }
            return false;
        }
    };
    let section_matches = search
        .map(|filter| {
            filter.matches(
                "orphaned configuration missing budget codes ghost categories rules cleanup",
            )
        })
        .unwrap_or(!orphaned_rules.is_empty());
    let visible_rules = orphaned_rules
        .iter()
        .filter(|rule| {
            search
                .map(|filter| orphaned_rule_matches(rule, filter))
                .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();
    if !section_matches && visible_rules.is_empty() {
        return false;
    }

    let remove_button = ui::plain_text_icon_button(
        "user-trash-symbolic",
        "Remove Orphaned Rules",
        "Remove rules that point to missing budget codes",
    );
    remove_button.add_css_class("destructive-action");
    remove_button.set_sensitive(!orphaned_rules.is_empty());
    register_config_widget(ui_handles, &remove_button);
    let state_for_remove = Rc::clone(state);
    let ui_for_remove = Rc::clone(ui_handles);
    remove_button.connect_clicked(move |button| {
        remove_orphaned_config_rules(&state_for_remove, &ui_for_remove, button);
    });

    ui_handles.debug.append(&ui::section_title_with_action(
        "Orphaned Configuration",
        "Rules that point to missing budget codes can create ghost categories or budget codes.",
        &remove_button,
    ));

    let rows_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    if visible_rules.is_empty() {
        rows_box.append(&ui::text_card(&tr("No orphaned configuration found.")));
    } else {
        let show_all = ui_handles.show_all.get() || search.is_some();
        let preview_limit = if show_all {
            usize::MAX
        } else {
            CATEGORY_PREVIEW_LIMIT
        };
        append_orphaned_rule_rows(&rows_box, visible_rules.iter().take(preview_limit));
        if !show_all && visible_rules.len() > preview_limit {
            let more_row = ui::more_list_row("More", "Show all orphaned rules");
            let more_container = more_row.container.clone();
            let rows_box_for_more = rows_box.clone();
            more_row.row.connect_activated(move |_| {
                ui::clear_box(&rows_box_for_more);
                append_orphaned_rule_rows(&rows_box_for_more, visible_rules.iter());
                more_container.set_visible(false);
            });
            rows_box.append(&more_row.container);
        }
    }
    ui_handles.debug.append(&rows_box);
    true
}

fn append_orphaned_rule_rows<'a>(
    container: &gtk::Box,
    rules: impl IntoIterator<Item = &'a data::OrphanedRule>,
) {
    for orphan in rules {
        container.append(&ui::text_card(&orphaned_rule_text(orphan)));
    }
}

fn orphaned_rule_matches(orphan: &data::OrphanedRule, filter: &SearchFilter) -> bool {
    let rule = &orphan.rule;
    filter.matches(&format!(
        "{} {} {} {} {} {} orphan missing budget code",
        orphan.budget_code, rule.category, rule.search, rule.field, rule.direction, rule.notes,
    ))
}

fn orphaned_rule_text(orphan: &data::OrphanedRule) -> String {
    trf(
        "Rule “{search}” uses missing budget code “{code}” in category “{category}”.",
        &[
            ("search", truncate(&orphan.rule.search, 80)),
            ("code", orphan.budget_code.clone()),
            ("category", orphan.rule.category.clone()),
        ],
    )
}
fn remove_orphaned_config_rules(
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    button: &gtk::Button,
) {
    if !try_begin_config_operation(ui_handles, "Another edit or save is already running.") {
        return;
    }

    let button = button.clone();
    let borrowed = state.borrow();
    let mode = borrowed.dedupe_mode;
    let remember_mode = ui_handles.remember_mode.get();
    let sources = current_sources_for_reload(&borrowed, remember_mode);
    let scope = current_transaction_load_scope(&borrowed, ui_handles.as_ref());
    drop(borrowed);
    let auto_clean_config = ui_handles.preferences.auto_clean_config();
    let state_for_remove = Rc::clone(state);
    let ui_for_remove = Rc::clone(ui_handles);
    button.set_sensitive(false);
    show_status(ui_handles, "Removing orphaned rules...");
    begin_background_operation(ui_handles.as_ref());

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            let removed = data::remove_orphaned_rules()?;
            let new_data = data::load_app_data_with_sources(
                mode,
                auto_clean_config,
                scope,
                remember_mode,
                &sources,
            )?
            .0;
            anyhow::Ok((removed, new_data))
        });

        match task.await {
            Ok(Ok((removed, new_data))) => {
                *state_for_remove.borrow_mut() = new_data;
                render_views(
                    &state_for_remove.borrow(),
                    &ui_for_remove,
                    &state_for_remove,
                );
                show_status(
                    &ui_for_remove,
                    &trf(
                        "{count} orphaned rule(s) removed.",
                        &[("count", removed.to_string())],
                    ),
                );
            }
            Ok(Err(error)) => show_status(
                &ui_for_remove,
                &trf(
                    "Could not remove orphaned rules: {error}",
                    &[("error", format!("{error:#}"))],
                ),
            ),
            Err(_) => show_status(
                &ui_for_remove,
                "Removing orphaned rules canceled: the background task stopped unexpectedly.",
            ),
        }
        finish_background_operation(ui_for_remove.as_ref());
        finish_config_operation(&ui_for_remove);
    });
}
