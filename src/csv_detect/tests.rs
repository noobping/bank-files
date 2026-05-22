use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MonthKey, TransactionLoadScope};
    use std::fs;

    #[test]
    fn imports_bundled_two_year_demo() {
        let path = PathBuf::from("data/example/demo_transactions.csv");
        assert!(path.exists(), "demo CSV missing");

        let outcome = import_files(
            &[path],
            &FieldAliases::defaults(),
            TransactionLoadScope::All,
        )
        .unwrap();

        assert!(outcome.warnings.is_empty(), "{:?}", outcome.warnings);
        assert_eq!(outcome.transactions.len(), 148);
        assert_eq!(outcome.reports[0].rows_seen, 148);
        assert_eq!(outcome.reports[0].rows_imported, 148);
        assert_eq!(outcome.reports[0].rows_skipped, 0);
        assert_eq!(
            outcome.reports[0].headers,
            [
                "Date",
                "Description",
                "Counterparty",
                "Amount",
                "Account",
                "Transaction ID"
            ]
        );

        let months = crate::analytics::monthly_totals_without_transfers(
            &outcome.transactions,
            &[],
            usize::MAX,
        );
        assert_eq!(months.len(), 24);
        assert_eq!(months.first().unwrap().month, MonthKey::new(2024, 1));
        assert_eq!(months.last().unwrap().month, MonthKey::new(2025, 12));
        assert!(months
            .iter()
            .any(|month| month.totals.income > Decimal::ZERO));
        assert!(months
            .iter()
            .any(|month| month.totals.expenses > Decimal::ZERO));
    }

    #[test]
    fn imports_bundled_language_demos() {
        for path in [
            "data/example/demo_transactions.en.csv",
            "data/example/demo_transactions.nl.csv",
            "data/example/demo_transactions.de.csv",
        ] {
            let path = PathBuf::from(path);
            assert!(path.exists(), "demo CSV missing: {}", path.display());

            let outcome = import_files(
                &[path],
                &FieldAliases::defaults(),
                TransactionLoadScope::All,
            )
            .unwrap();

            assert!(outcome.warnings.is_empty(), "{:?}", outcome.warnings);
            assert_eq!(outcome.transactions.len(), 12);
            assert_eq!(outcome.reports[0].rows_seen, 12);
            assert_eq!(outcome.reports[0].rows_imported, 12);
            assert_eq!(outcome.reports[0].rows_skipped, 0);
        }
    }

    #[test]
    fn parallel_chunk_ranges_cover_items_once_in_order() {
        let ranges = import::parallel_chunk_ranges(10, 3);

        assert_eq!(ranges, vec![0..4, 4..7, 7..10]);
        assert_eq!(
            import::parallel_chunk_ranges(0, 3),
            Vec::<std::ops::Range<usize>>::new()
        );
        assert_eq!(import::parallel_chunk_ranges(3, 10), vec![0..1, 1..2, 2..3]);
    }

    #[test]
    fn parallel_import_preserves_file_order() {
        let first = scoped_test_csv(
            "parallel_import_preserves_file_order_first",
            "Date;Description;Amount
2025-01-01;First file;10,00
",
        );
        let second = scoped_test_csv(
            "parallel_import_preserves_file_order_second",
            "Date;Description;Amount
2025-01-02;Second file;20,00
",
        );
        let files = vec![first.clone(), second.clone()];

        let outcome =
            import_files(&files, &FieldAliases::defaults(), TransactionLoadScope::All).unwrap();
        let _ = fs::remove_file(first);
        let _ = fs::remove_file(second);

        assert!(outcome.warnings.is_empty(), "{:?}", outcome.warnings);
        assert_eq!(outcome.reports.len(), 2);
        assert_eq!(outcome.reports[0].source, files[0]);
        assert_eq!(outcome.reports[1].source, files[1]);
        assert_eq!(outcome.transactions[0].description, "First file");
        assert_eq!(outcome.transactions[1].description, "Second file");
    }

    #[test]
    fn scoped_year_import_loads_only_selected_year_but_keeps_metadata() {
        let path = PathBuf::from("data/example/demo_transactions.csv");
        let outcome = import_files(
            &[path],
            &FieldAliases::defaults(),
            TransactionLoadScope::Year(Some(2025)),
        )
        .unwrap();

        assert_eq!(outcome.loaded_scope, TransactionLoadScope::Year(Some(2025)));
        assert_eq!(outcome.available_months.len(), 24);
        assert!(outcome.transactions.iter().all(|tx| tx.year() == 2025));
        assert!(outcome.transactions.iter().all(|tx| tx.year() != 2024));
    }

    #[test]
    fn scoped_year_with_previous_imports_selected_and_previous_year() {
        let path = PathBuf::from("data/example/demo_transactions.csv");
        let outcome = import_files(
            &[path],
            &FieldAliases::defaults(),
            TransactionLoadScope::YearWithPrevious(Some(2025)),
        )
        .unwrap();

        assert_eq!(
            outcome.loaded_scope,
            TransactionLoadScope::YearWithPrevious(Some(2025))
        );
        assert!(outcome.transactions.iter().any(|tx| tx.year() == 2024));
        assert!(outcome.transactions.iter().any(|tx| tx.year() == 2025));
        assert!(outcome
            .transactions
            .iter()
            .all(|tx| matches!(tx.year(), 2024 | 2025)));
    }

    #[test]
    fn scoped_month_import_loads_only_selected_month() {
        let path = PathBuf::from("data/example/demo_transactions.csv");
        let month = MonthKey::new(2025, 12);
        let outcome = import_files(
            &[path],
            &FieldAliases::defaults(),
            TransactionLoadScope::Month(Some(month)),
        )
        .unwrap();

        assert_eq!(
            outcome.loaded_scope,
            TransactionLoadScope::Month(Some(month))
        );
        assert!(outcome
            .transactions
            .iter()
            .all(|tx| tx.month_key() == month));
    }

    #[test]
    fn scoped_month_with_previous_imports_two_months() {
        let path = PathBuf::from("data/example/demo_transactions.csv");
        let month = MonthKey::new(2025, 12);
        let outcome = import_files(
            &[path],
            &FieldAliases::defaults(),
            TransactionLoadScope::MonthWithPrevious(Some(month)),
        )
        .unwrap();

        let previous = month.previous();
        assert_eq!(
            outcome.loaded_scope,
            TransactionLoadScope::MonthWithPrevious(Some(month))
        );
        assert!(outcome
            .transactions
            .iter()
            .any(|tx| tx.month_key() == previous));
        assert!(outcome
            .transactions
            .iter()
            .any(|tx| tx.month_key() == month));
        assert!(outcome
            .transactions
            .iter()
            .all(|tx| tx.month_key() == previous || tx.month_key() == month));
    }

    #[test]
    fn sorted_scope_stops_transaction_import_after_window() {
        let path = PathBuf::from("data/example/demo_transactions.csv");
        let outcome = import_files(
            &[path],
            &FieldAliases::defaults(),
            TransactionLoadScope::Month(Some(MonthKey::new(2025, 1))),
        )
        .unwrap();

        assert!(outcome.reports[0].rows_seen < 148);
        assert!(outcome
            .transactions
            .iter()
            .all(|tx| tx.month_key() == MonthKey::new(2025, 1)));
    }

    #[test]
    fn unsorted_scope_keeps_scanning_for_matching_rows() {
        let path = scoped_test_csv(
            "unsorted_scope_keeps_scanning_for_matching_rows",
            "Date;Description;Amount\n\
2025-03-01;March income;100,00\n\
2024-01-01;Old expense;-10,00\n\
2025-01-01;January expense;-20,00\n\
2026-01-01;Future income;200,00\n",
        );

        let outcome = import_files(
            std::slice::from_ref(&path),
            &FieldAliases::defaults(),
            TransactionLoadScope::Year(Some(2025)),
        )
        .unwrap();
        let _ = fs::remove_file(path);

        assert_eq!(outcome.reports[0].rows_seen, 4);
        assert_eq!(outcome.transactions.len(), 2);
        assert!(outcome.transactions.iter().all(|tx| tx.year() == 2025));
    }

    fn scoped_test_csv(name: &str, content: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("bank-files-{name}-{}.csv", std::process::id()));
        fs::write(&path, content).unwrap();
        path
    }
}
