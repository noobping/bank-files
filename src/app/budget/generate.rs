use super::*;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub(in crate::app) fn generate_budgets_from_transactions_with_status(
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
    let scope = current_transaction_load_scope(&snapshot, ui.as_ref());
    let state_for_generate = Rc::clone(state);
    let ui_for_generate = Rc::clone(ui);
    show_config_status(
        ui.as_ref(),
        dialog_status.as_ref(),
        "Generating budgets from transactions...",
    );
    begin_background_operation(ui.as_ref());

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            let budgets = generated_budget_suggestions(&snapshot);
            if budgets.is_empty() {
                return anyhow::Ok(GeneratedBudgetsOutcome::None);
            }

            let count = budgets.len();
            data::write_editable_budgets(&budgets)?;
            let new_data = data::load_app_data_with_config_cleanup(mode, auto_clean_config, scope)?;
            anyhow::Ok(GeneratedBudgetsOutcome::Generated {
                count,
                data: new_data,
            })
        });

        match task.await {
            Ok(Ok(GeneratedBudgetsOutcome::Generated { count, data })) => {
                *state_for_generate.borrow_mut() = data;
                render_views(
                    &state_for_generate.borrow(),
                    &ui_for_generate,
                    &state_for_generate,
                );
                let message = trf(
                    "Replaced default budgets with {count} budgets from transactions.",
                    &[("count", count.to_string())],
                );
                show_config_status_text(ui_for_generate.as_ref(), dialog_status.as_ref(), &message);
            }
            Ok(Ok(GeneratedBudgetsOutcome::None)) => {
                show_config_status(
                    ui_for_generate.as_ref(),
                    dialog_status.as_ref(),
                    "No budgets could be generated from the imported transactions yet.",
                );
            }
            Ok(Err(error)) => {
                let message = trf(
                    "Could not generate budgets: {error}",
                    &[("error", format!("{error:#}"))],
                );
                show_config_status_text(ui_for_generate.as_ref(), dialog_status.as_ref(), &message);
            }
            Err(_) => show_config_status(
                ui_for_generate.as_ref(),
                dialog_status.as_ref(),
                "Budget generation canceled: the background task stopped unexpectedly.",
            ),
        }
        finish_background_operation(ui_for_generate.as_ref());
        finish_config_operation(&ui_for_generate);
    });
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

enum GeneratedBudgetsOutcome {
    Generated { count: usize, data: AppData },
    None,
}

fn generated_budget_suggestions(data: &AppData) -> Vec<EditableBudget> {
    let months = data
        .transactions
        .iter()
        .map(|transaction| transaction.month_key().to_string())
        .collect::<BTreeSet<_>>();
    let period = months.into_iter().rev().take(12).collect::<BTreeSet<_>>();
    if period.is_empty() {
        return Vec::new();
    }

    let mut groups = BTreeMap::<String, GeneratedBudgetStats>::new();
    for transaction in &data.transactions {
        if transaction.amount >= Decimal::ZERO
            || !period.contains(&transaction.month_key().to_string())
        {
            continue;
        }
        let code = transaction.budget_code.trim();
        if code.is_empty() || code.eq_ignore_ascii_case("INC-OTHER") {
            continue;
        }

        let group = groups.entry(code.to_string()).or_default();
        let expense = -transaction.amount;
        group.total += expense;
        let category = transaction.category.trim();
        if !category.is_empty() {
            *group.categories.entry(category.to_string()).or_default() += expense;
        }
    }

    let month_count = Decimal::from(period.len() as u64);
    groups
        .into_iter()
        .filter_map(|(code, stats)| {
            if stats.total <= Decimal::ZERO {
                return None;
            }
            let monthly_budget = (stats.total / month_count).round_dp(2);
            if monthly_budget <= Decimal::ZERO {
                return None;
            }
            Some(EditableBudget {
                code,
                category: stats.category(),
                monthly_budget: monthly_budget.to_string(),
                yearly_budget: String::new(),
                direction: "expense".to_string(),
                income_basis: "real".to_string(),
                notes: trf(
                    "Generated from transactions over {count} imported months on {date}.",
                    &[
                        ("count", period.len().to_string()),
                        ("date", chrono::Local::now().date_naive().to_string()),
                    ],
                ),
            })
        })
        .collect()
}

#[derive(Default)]
struct GeneratedBudgetStats {
    total: Decimal,
    categories: HashMap<String, Decimal>,
}

impl GeneratedBudgetStats {
    fn category(&self) -> String {
        self.categories
            .iter()
            .max_by(|left, right| left.1.cmp(right.1).then_with(|| right.0.cmp(left.0)))
            .map(|(category, _)| category.clone())
            .unwrap_or_else(|| "Generated budget".to_string())
    }
}
