use super::*;

pub(in crate::app) fn budget_progress_detail(
    budget: &analytics::BudgetUsage,
) -> (String, ui::ProgressState) {
    if budget.budget <= Decimal::ZERO && budget.actual > Decimal::ZERO {
        return (
            trf(
                "{amount} without a budget",
                &[("amount", money(budget.actual))],
            ),
            ui::ProgressState::Error,
        );
    }
    if budget.remaining < Decimal::ZERO {
        (
            trf(
                "{amount} over budget",
                &[("amount", money(-budget.remaining))],
            ),
            ui::ProgressState::Error,
        )
    } else {
        (
            trf("{amount} available", &[("amount", money(budget.remaining))]),
            ui::ProgressState::Normal,
        )
    }
}

pub(in crate::app) fn annual_budget_progress_detail(
    budget: &analytics::AnnualBudgetUsage,
) -> (String, ui::ProgressState) {
    if budget.budget <= Decimal::ZERO && budget.actual > Decimal::ZERO {
        return (
            trf(
                "{amount} spent without an annual budget",
                &[("amount", money(budget.actual))],
            ),
            ui::ProgressState::Error,
        );
    }
    if budget.remaining < Decimal::ZERO {
        (
            trf(
                "{amount} over annual budget",
                &[("amount", money(-budget.remaining))],
            ),
            ui::ProgressState::Error,
        )
    } else {
        (
            trf("{amount} available", &[("amount", money(budget.remaining))]),
            ui::ProgressState::Normal,
        )
    }
}

pub(in crate::app) fn annual_budget_previous_state(
    budget: &analytics::AnnualBudgetUsage,
) -> ui::ProgressState {
    let Some(previous_actual) = budget.previous_actual else {
        return ui::ProgressState::Normal;
    };
    if budget.budget <= Decimal::ZERO && previous_actual > Decimal::ZERO {
        return ui::ProgressState::Error;
    }
    if budget.budget > Decimal::ZERO && previous_actual > budget.budget {
        ui::ProgressState::Error
    } else {
        ui::ProgressState::Normal
    }
}

pub(in crate::app) fn budget_display_title(
    code: &str,
    category: &str,
    advanced_features: bool,
) -> String {
    let category = category.trim();
    if advanced_features {
        let code = code.trim();
        if code.is_empty() {
            category.to_string()
        } else if category.is_empty() {
            code.to_string()
        } else {
            format!("{} · {}", code, category)
        }
    } else {
        category.to_string()
    }
}

pub(in crate::app) fn category_transaction_detail(
    count: impl ToString,
    budget_code: &str,
    advanced_features: bool,
) -> String {
    if advanced_features {
        trf(
            "{count} transactions · budget code {code}",
            &[
                ("count", count.to_string()),
                ("code", budget_code.to_string()),
            ],
        )
    } else {
        trf("{count} transactions", &[("count", count.to_string())])
    }
}

pub(in crate::app) fn planned_budget_label(budget: Decimal, basis: &str) -> String {
    if budget <= Decimal::ZERO {
        tr("no budget")
    } else if matches!(basis, "fixed budget" | "unconfigured budget") {
        money(budget)
    } else {
        trf(
            "{amount} ({basis})",
            &[
                ("amount", money(budget)),
                ("basis", budget_basis_label(basis)),
            ],
        )
    }
}

fn budget_basis_label(basis: &str) -> String {
    if let Some(percent) = basis.strip_suffix("% of real income") {
        return trf(
            "{percent}% of real income",
            &[("percent", percent.trim().to_string())],
        );
    }
    if let Some(percent) = basis.strip_suffix("% of planned income") {
        return trf(
            "{percent}% of planned income",
            &[("percent", percent.trim().to_string())],
        );
    }
    if let Some(percent) = basis.strip_suffix("% of income") {
        return trf(
            "{percent}% of income",
            &[("percent", percent.trim().to_string())],
        );
    }
    if let Some((basis, context)) = basis.trim().split_once(" (") {
        if let Some(context) = context.strip_suffix(')') {
            return trf(
                "{basis} - {context}",
                &[("basis", tr(basis.trim())), ("context", tr(context.trim()))],
            );
        }
    }
    tr(basis)
}

pub(in crate::app) fn fraction(value: Decimal, max: Decimal) -> f64 {
    if max <= Decimal::ZERO {
        if value > Decimal::ZERO {
            1.0
        } else {
            0.0
        }
    } else {
        (value / max).to_f64().unwrap_or(0.0).clamp(0.0, 1.0)
    }
}

pub(in crate::app) fn file_size(path: &PathBuf) -> u64 {
    std::fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0)
}

pub(in crate::app) fn format_size(size: u64) -> String {
    if size >= 1_048_576 {
        format!("{:.1} MB", size as f64 / 1_048_576.0)
    } else if size >= 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else {
        format!("{size} bytes")
    }
}
