#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::app) enum FinancialMetric {
    Income,
    Expenses,
    Balance,
}

pub(in crate::app) fn financial_metric_order() -> [FinancialMetric; 3] {
    [
        FinancialMetric::Income,
        FinancialMetric::Expenses,
        FinancialMetric::Balance,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn financial_metrics_keep_balance_last() {
        assert_eq!(
            financial_metric_order(),
            [
                FinancialMetric::Income,
                FinancialMetric::Expenses,
                FinancialMetric::Balance,
            ]
        );
    }
}
