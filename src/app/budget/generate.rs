use super::*;
#[cfg(not(feature = "flatpak"))]
use gtk::gio::prelude::NetworkMonitorExt;

pub(in crate::app) fn generate_configuration_from_transactions_with_status(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    dialog_status: Option<StatusHandle>,
) {
    let smart_insights_enabled =
        smart_pattern_detection_enabled(ui.advanced_features.get(), ui.show_predictions.get());
    if !smart_insights_enabled {
        show_config_status(
            ui.as_ref(),
            dialog_status.as_ref(),
            "Configuration generation needs Smart Insights. Enable Smart Insights to generate configuration from transactions.",
        );
        show_verbose_status(
            ui.as_ref(),
            "configuration generation skipped; smart insights disabled",
        );
        return;
    }

    let busy_message = "Another edit or save is already running.";
    if !try_begin_config_operation(ui, busy_message) {
        if let Some(status) = dialog_status.as_ref() {
            status.set_text(&tr(busy_message));
        }
        return;
    }

    let snapshot = state.borrow().clone();
    let mode = snapshot.dedupe_mode;
    let remember_mode = ui.remember_mode.get();
    let sources = current_sources_for_reload(&snapshot, remember_mode);
    let auto_clean_config = ui.preferences.auto_clean_config();
    let restore_scope = current_transaction_load_scope(&snapshot, ui.as_ref());
    show_verbose_status(
        ui.as_ref(),
        format!(
            "configuration generation requested; loaded_scope={:?}; transactions={}; smart_insights={smart_insights_enabled}",
            snapshot.loaded_scope,
            snapshot.transactions.len(),
        ),
    );
    let state_for_generate = Rc::clone(state);
    let ui_for_generate = Rc::clone(ui);
    show_config_status(
        ui.as_ref(),
        dialog_status.as_ref(),
        "Generating configuration from transactions...",
    );
    show_config_status(
        ui.as_ref(),
        dialog_status.as_ref(),
        "Configuration generation uses complete imported calendar years for budget amounts and ignores incomplete years.",
    );
    show_smart_enrichment_status(ui.as_ref(), dialog_status.as_ref());
    let analysis_message = match (
        matches!(snapshot.loaded_scope, TransactionLoadScope::All),
        smart_insights_enabled,
    ) {
        (true, true) => {
            "Analysing loaded transactions, yearly comparisons, recurring patterns, transfers, and field mappings..."
        }
        (true, false) => {
            "Analysing loaded transactions, yearly comparisons, repeated categories, and field mappings..."
        }
        (false, true) => {
            "Loading all imported transactions before analysing yearly comparisons, patterns, and transfers..."
        }
        (false, false) => {
            "Loading all imported transactions before analysing yearly comparisons and repeated categories..."
        }
    };
    show_config_status(ui.as_ref(), dialog_status.as_ref(), analysis_message);
    set_config_status_loading(dialog_status.as_ref(), true);
    begin_background_operation(ui.as_ref());

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            let generation_data = generation_app_data(
                snapshot,
                mode,
                auto_clean_config,
                remember_mode,
                &sources,
                smart_insights_enabled,
            )?;
            let generated =
                data::generate_automatic_configuration(&generation_data, smart_insights_enabled)?;
            let ai_outcome = crate::local_ai::enhance_generated_configuration(
                &generation_data,
                generated,
                smart_insights_enabled,
            );
            let generated = ai_outcome.configuration;
            if generated.summary.is_empty() {
                return anyhow::Ok(GeneratedConfigurationOutcome::None {
                    ai_status: ai_outcome.status,
                });
            }

            let summary = generated.summary.clone();
            data::write_generated_configuration(&generated)?;
            let data = data::load_app_data_with_sources(
                mode,
                auto_clean_config,
                restore_scope,
                remember_mode,
                &sources,
                smart_insights_enabled,
            )?
            .0;
            anyhow::Ok(GeneratedConfigurationOutcome::Generated {
                summary,
                data: Box::new(data),
                ai_status: ai_outcome.status,
            })
        });

        match task.await {
            Ok(Ok(GeneratedConfigurationOutcome::Generated {
                summary,
                data,
                ai_status,
            })) => {
                *state_for_generate.borrow_mut() = *data;
                show_verbose_status(
                    ui_for_generate.as_ref(),
                    format!(
                        "configuration generation finished; years={}; months={}; budgets={}; rules={}; fields={}; hidden={}",
                        summary.complete_years,
                        summary.budget_months,
                        summary.budgets,
                        summary.rules,
                        summary.field_mappings,
                        summary.ignored_patterns,
                    ),
                );
                render_views(
                    &state_for_generate.borrow(),
                    &ui_for_generate,
                    &state_for_generate,
                );
                if let Some(status) = ai_status {
                    show_config_status_text(ui_for_generate.as_ref(), dialog_status.as_ref(), &status);
                }
                let message = trf(
                    "Generated configuration from {years} complete year(s), covering {months} month(s): {budgets} budget(s), {rules} rule(s), {fields} field mapping(s), {hidden} hidden pattern(s).",
                    &[
                        ("years", summary.complete_years.to_string()),
                        ("months", summary.budget_months.to_string()),
                        ("budgets", summary.budgets.to_string()),
                        ("rules", summary.rules.to_string()),
                        ("fields", summary.field_mappings.to_string()),
                        ("hidden", summary.ignored_patterns.to_string()),
                    ],
                );
                show_config_status_text(ui_for_generate.as_ref(), dialog_status.as_ref(), &message);
            }
            Ok(Ok(GeneratedConfigurationOutcome::None { ai_status })) => {
                show_verbose_status(ui_for_generate.as_ref(), "configuration generation finished with no changes");
                if let Some(status) = ai_status {
                    show_config_status_text(ui_for_generate.as_ref(), dialog_status.as_ref(), &status);
                }
                show_config_status(
                    ui_for_generate.as_ref(),
                    dialog_status.as_ref(),
                    "No configuration could be generated yet. Import a complete calendar year to generate budget amounts.",
                );
            }
            Ok(Err(error)) => {
                show_verbose_status(
                    ui_for_generate.as_ref(),
                    format!("configuration generation failed; error={error:#}"),
                );
                let message = trf(
                    "Could not generate configuration: {error}",
                    &[("error", format!("{error:#}"))],
                );
                show_config_status_text(ui_for_generate.as_ref(), dialog_status.as_ref(), &message);
            }
            Err(_) => {
                show_verbose_status(ui_for_generate.as_ref(), "configuration generation task canceled");
                show_config_status(
                    ui_for_generate.as_ref(),
                dialog_status.as_ref(),
                    "Configuration generation canceled: the background task stopped unexpectedly.",
                )
            }
        }
        set_config_status_loading(dialog_status.as_ref(), false);
        finish_background_operation(ui_for_generate.as_ref());
        finish_config_operation(&ui_for_generate);
    });
}

fn generation_app_data(
    snapshot: AppData,
    mode: DedupeMode,
    auto_clean_config: bool,
    remember_mode: RememberMode,
    sources: &[TransactionSource],
    smart_insights_enabled: bool,
) -> anyhow::Result<AppData> {
    if matches!(snapshot.loaded_scope, TransactionLoadScope::All) {
        Ok(snapshot)
    } else {
        data::load_app_data_with_sources(
            mode,
            auto_clean_config,
            TransactionLoadScope::All,
            remember_mode,
            sources,
            smart_insights_enabled,
        )
        .map(|loaded| loaded.0)
    }
}

fn show_config_status(ui: &UiHandles, dialog_status: Option<&StatusHandle>, message: &str) {
    let message = tr(message);
    show_config_status_text(ui, dialog_status, &message);
}

fn show_config_status_text(ui: &UiHandles, dialog_status: Option<&StatusHandle>, message: &str) {
    if let Some(status) = dialog_status {
        status.set_text(message);
    }
    show_status(ui, message);
}

fn set_config_status_loading(dialog_status: Option<&StatusHandle>, loading: bool) {
    if let Some(status) = dialog_status {
        status.set_loading(loading);
    }
}

#[cfg(not(feature = "flatpak"))]
fn show_smart_enrichment_status(ui: &UiHandles, dialog_status: Option<&StatusHandle>) {
    let smart_insights_enabled =
        smart_pattern_detection_enabled(ui.advanced_features.get(), ui.show_predictions.get());
    let message = if !smart_insights_enabled {
        "Smart Insights are disabled. Online merchant enrichment, detected transfers, and extra pattern hints are skipped."
    } else if !ui.online_smart_insights.get() {
        "Online Smart Insights are off by default. Configuration generation uses only local transactions, and no merchant names or transaction fields are sent."
    } else if !online_smart_insights_network_available() {
        "Online Smart Insights are enabled, but no network connection is available. External merchant lookups are skipped, and no transaction data is sent."
    } else {
        "Online Smart Insights are enabled, but no safe lookup provider is configured in this build. External merchant lookups are skipped, and no transaction data is sent."
    };
    show_config_status(ui, dialog_status, message);
}

#[cfg(feature = "flatpak")]
fn show_smart_enrichment_status(ui: &UiHandles, dialog_status: Option<&StatusHandle>) {
    let message =
        if smart_pattern_detection_enabled(ui.advanced_features.get(), ui.show_predictions.get()) {
            "Configuration generation uses local transactions only in this build."
        } else {
            "Smart Insights are disabled. Detected transfers and extra pattern hints are skipped."
        };
    show_config_status(ui, dialog_status, message);
}

#[cfg(not(feature = "flatpak"))]
fn online_smart_insights_network_available() -> bool {
    gtk::gio::NetworkMonitor::default().is_network_available()
}

enum GeneratedConfigurationOutcome {
    Generated {
        summary: data::GeneratedConfigurationSummary,
        data: Box<AppData>,
        ai_status: Option<String>,
    },
    None {
        ai_status: Option<String>,
    },
}
