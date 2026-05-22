use super::*;

pub(in crate::app) fn generate_configuration_from_transactions_with_status(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    dialog_status: Option<gtk::Label>,
) {
    let busy_message = "Another edit or save is already running.";
    if !try_begin_config_operation(ui, busy_message) {
        if let Some(label) = dialog_status.as_ref() {
            label.set_text(&tr(busy_message));
        }
        return;
    }

    let snapshot = state.borrow().clone();
    let mode = snapshot.dedupe_mode;
    let auto_clean_config = ui.preferences.auto_clean_config();
    let restore_scope = current_transaction_load_scope(&snapshot, ui.as_ref());
    let state_for_generate = Rc::clone(state);
    let ui_for_generate = Rc::clone(ui);
    show_config_status(
        ui.as_ref(),
        dialog_status.as_ref(),
        "Generating configuration from transactions...",
    );
    begin_background_operation(ui.as_ref());

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            let generation_data = generation_app_data(snapshot, mode, auto_clean_config)?;
            let generated = data::generate_automatic_configuration(&generation_data)?;
            if generated.summary.is_empty() {
                return anyhow::Ok(GeneratedConfigurationOutcome::None);
            }

            let summary = generated.summary.clone();
            data::write_generated_configuration(&generated)?;
            let data = data::load_app_data_with_config_cleanup(
                mode,
                auto_clean_config,
                restore_scope,
            )?;
            anyhow::Ok(GeneratedConfigurationOutcome::Generated { summary, data })
        });

        match task.await {
            Ok(Ok(GeneratedConfigurationOutcome::Generated { summary, data })) => {
                *state_for_generate.borrow_mut() = data;
                render_views(
                    &state_for_generate.borrow(),
                    &ui_for_generate,
                    &state_for_generate,
                );
                let message = trf(
                    "Generated configuration: {budgets} budget(s), {rules} rule(s), {fields} field mapping(s), {hidden} hidden pattern(s).",
                    &[
                        ("budgets", summary.budgets.to_string()),
                        ("rules", summary.rules.to_string()),
                        ("fields", summary.field_mappings.to_string()),
                        ("hidden", summary.ignored_patterns.to_string()),
                    ],
                );
                show_config_status_text(ui_for_generate.as_ref(), dialog_status.as_ref(), &message);
            }
            Ok(Ok(GeneratedConfigurationOutcome::None)) => {
                show_config_status(
                    ui_for_generate.as_ref(),
                    dialog_status.as_ref(),
                    "No configuration could be generated from imported transactions yet.",
                );
            }
            Ok(Err(error)) => {
                let message = trf(
                    "Could not generate configuration: {error}",
                    &[("error", format!("{error:#}"))],
                );
                show_config_status_text(ui_for_generate.as_ref(), dialog_status.as_ref(), &message);
            }
            Err(_) => show_config_status(
                ui_for_generate.as_ref(),
                dialog_status.as_ref(),
                "Configuration generation canceled: the background task stopped unexpectedly.",
            ),
        }
        finish_background_operation(ui_for_generate.as_ref());
        finish_config_operation(&ui_for_generate);
    });
}

fn generation_app_data(
    snapshot: AppData,
    mode: DedupeMode,
    auto_clean_config: bool,
) -> anyhow::Result<AppData> {
    if matches!(snapshot.loaded_scope, TransactionLoadScope::All) {
        Ok(snapshot)
    } else {
        data::load_app_data_with_config_cleanup(mode, auto_clean_config, TransactionLoadScope::All)
    }
}

fn show_config_status(ui: &UiHandles, dialog_status: Option<&gtk::Label>, message: &str) {
    let message = tr(message);
    show_config_status_text(ui, dialog_status, &message);
}

fn show_config_status_text(ui: &UiHandles, dialog_status: Option<&gtk::Label>, message: &str) {
    if let Some(label) = dialog_status {
        label.set_text(message);
    }
    show_status(ui, message);
}

enum GeneratedConfigurationOutcome {
    Generated {
        summary: data::GeneratedConfigurationSummary,
        data: AppData,
    },
    None,
}
