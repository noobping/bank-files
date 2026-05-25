use super::super::*;
use super::model::{ApplyCounts, QueuedOperationKind, RuleCombineSummary};
use super::widgets::refresh_operation_queue_ui;

pub(super) fn clear_done(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
    let removed = ui.operation_queue.clear_applied();
    refresh_operation_queue_ui(state, ui);
    if removed > 0 {
        ui.operation_queue_widgets.dialog.close();
        show_status(ui, "Completed queued operations cleared.");
    }
}

pub(super) fn apply_one(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>, id: u64) {
    apply_operations(state, ui, vec![id]);
}

pub(super) fn apply_all(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
    apply_operations(state, ui, ui.operation_queue.actionable_ids());
}

fn apply_operations(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>, ids: Vec<u64>) {
    let ids = ids
        .into_iter()
        .filter(|id| ui.operation_queue.operation_kind(*id).is_some())
        .collect::<Vec<_>>();
    if ids.is_empty() {
        show_status(ui, "No queued operations to apply.");
        return;
    }
    if ui.operation_queue.is_processing() {
        show_status(ui, "The processing queue is already running.");
        return;
    }
    if !try_begin_config_operation(ui, "Another edit or save is already running.") {
        return;
    }

    show_verbose_status(
        ui.as_ref(),
        format!("queue apply started; operations={}", ids.len()),
    );
    ui.operation_queue.set_processing(true);
    ui.status_icon.set_icon_name(Some("view-refresh-symbolic"));
    refresh_operation_queue_ui(state, ui);

    let state_for_apply = Rc::clone(state);
    let ui_for_apply = Rc::clone(ui);
    gtk::glib::MainContext::default().spawn_local(async move {
        let total = ids.len();
        let mut counts = ApplyCounts::default();
        for (index, id) in ids.into_iter().enumerate() {
            let Some(kind) = ui_for_apply.operation_queue.operation_kind(id) else {
                continue;
            };
            if !ui_for_apply.operation_queue.mark_applying(id) {
                continue;
            }
            refresh_operation_queue_ui(&state_for_apply, &ui_for_apply);
            show_status(
                &ui_for_apply,
                &trf(
                    "Applying queued operation {current} of {total}...",
                    &[
                        ("current", (index + 1).to_string()),
                        ("total", total.to_string()),
                    ],
                ),
            );

            let task = gtk::gio::spawn_blocking(move || apply_queued_operation(kind));
            match task.await {
                Ok(Ok(())) => {
                    counts.applied += 1;
                    ui_for_apply.operation_queue.mark_applied(id);
                    show_verbose_status(
                        ui_for_apply.as_ref(),
                        format!("queue operation applied; id={id}"),
                    );
                }
                Ok(Err(error)) => {
                    counts.failed += 1;
                    show_verbose_status(
                        ui_for_apply.as_ref(),
                        format!("queue operation failed; id={id}; error={error:#}"),
                    );
                    ui_for_apply
                        .operation_queue
                        .mark_failed(id, format!("{error:#}"));
                }
                Err(_) => {
                    counts.failed += 1;
                    show_verbose_status(
                        ui_for_apply.as_ref(),
                        format!("queue operation canceled; id={id}"),
                    );
                    ui_for_apply
                        .operation_queue
                        .mark_failed(id, tr("The background task stopped unexpectedly."));
                }
            }
            refresh_operation_queue_ui(&state_for_apply, &ui_for_apply);
        }

        ui_for_apply.operation_queue.set_processing(false);
        ui_for_apply
            .status_icon
            .set_icon_name(Some("dialog-information-symbolic"));
        refresh_operation_queue_ui(&state_for_apply, &ui_for_apply);

        if counts.applied > 0 {
            reload_after_queue_apply(state_for_apply, ui_for_apply, counts).await;
        } else {
            show_apply_summary(&ui_for_apply, counts, None);
            finish_config_operation(&ui_for_apply);
        }
    });
}

fn apply_queued_operation(kind: QueuedOperationKind) -> anyhow::Result<()> {
    match kind {
        QueuedOperationKind::Rule {
            rule,
            ensure_budget,
            ..
        } => {
            super::super::config_ops::apply_rule_config_change(rule, ensure_budget)?;
            Ok(())
        }
    }
}

async fn reload_after_queue_apply(
    state: Rc<RefCell<AppData>>,
    ui: Rc<UiHandles>,
    counts: ApplyCounts,
) {
    let remember_mode = ui.remember_mode.get();
    let (mode, sources, scope) = {
        let borrowed = state.borrow();
        (
            borrowed.dedupe_mode,
            current_sources_for_reload(&borrowed, remember_mode),
            current_transaction_load_scope(&borrowed, ui.as_ref()),
        )
    };
    show_verbose_status(
        ui.as_ref(),
        format!(
            "queue reload started; scope={scope:?}; remember={remember_mode:?}; sources={}",
            sources.len()
        ),
    );
    show_status(&ui, "Grouping and combining queued rules...");
    begin_background_operation(ui.as_ref());
    let task = gtk::gio::spawn_blocking(move || {
        let combine_summary = combine_queued_rules()?;
        let new_data = data::load_app_data_with_sources(mode, scope, remember_mode, &sources)?.0;
        anyhow::Ok((new_data, combine_summary))
    });

    match task.await {
        Ok(Ok((new_data, combine_summary))) => {
            *state.borrow_mut() = new_data;
            show_verbose_status(
                ui.as_ref(),
                format!(
                    "queue reload finished; transactions={}; reports={}",
                    state.borrow().transactions.len(),
                    state.borrow().reports.len(),
                ),
            );
            render_views(&state.borrow(), &ui, &state);
            show_apply_summary(&ui, counts, combine_summary);
        }
        Ok(Err(error)) => show_status(
            &ui,
            &trf(
                "Queued operations applied, but reload failed: {error}",
                &[("error", format!("{error:#}"))],
            ),
        ),
        Err(_) => show_status(
            &ui,
            "Queued operations applied, but reload canceled: the background task stopped unexpectedly.",
        ),
    }

    finish_background_operation(ui.as_ref());
    finish_config_operation(&ui);
    refresh_operation_queue_ui(&state, &ui);
}

fn combine_queued_rules() -> anyhow::Result<Option<RuleCombineSummary>> {
    let rules = data::load_editable_rules()?;
    let (rules, summary) = group_and_combine_queued_rules(&rules);
    if summary.is_some() {
        data::write_editable_rules(&rules)?;
    }
    Ok(summary)
}

pub(super) fn group_and_combine_queued_rules(
    rules: &[EditableRule],
) -> (Vec<EditableRule>, Option<RuleCombineSummary>) {
    let grouped = data::group_editable_rules_for_combining(rules);
    let report = data::combine_editable_rules(&grouped.rules);
    if report.before_count == report.after_count {
        return (report.rules, None);
    }

    let summary = RuleCombineSummary {
        before_count: report.before_count,
        after_count: report.after_count,
    };
    (report.rules, Some(summary))
}

fn show_apply_summary(
    ui: &Rc<UiHandles>,
    counts: ApplyCounts,
    combine_summary: Option<RuleCombineSummary>,
) {
    let message = apply_summary_message(counts, combine_summary);
    show_status(ui, &message);
}

pub(super) fn apply_summary_message(
    counts: ApplyCounts,
    combine_summary: Option<RuleCombineSummary>,
) -> String {
    let message = match (counts.applied, counts.failed) {
        (0, 0) => tr("No queued operations were applied."),
        (applied, 0) => trf(
            "Applied {count} queued operation(s).",
            &[("count", applied.to_string())],
        ),
        (0, failed) => trf(
            "{count} queued operation(s) failed.",
            &[("count", failed.to_string())],
        ),
        (applied, failed) => trf(
            "Applied {applied} queued operation(s); {failed} failed.",
            &[
                ("applied", applied.to_string()),
                ("failed", failed.to_string()),
            ],
        ),
    };

    if let Some(summary) = combine_summary {
        trf(
            "{summary} Grouped and combined rules from {before_count} to {after_count}.",
            &[
                ("summary", message),
                ("before_count", summary.before_count.to_string()),
                ("after_count", summary.after_count.to_string()),
            ],
        )
    } else {
        message
    }
}
