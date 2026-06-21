use super::*;

#[test]
fn compact_print_table_removes_columns_with_only_empty_cells() {
    let columns = vec![
        PrintColumn {
            title: "Date".into(),
            width: 1.0,
            align: PrintAlign::Left,
        },
        PrintColumn {
            title: "Budget".into(),
            width: 1.0,
            align: PrintAlign::Left,
        },
        PrintColumn {
            title: "Description".into(),
            width: 1.0,
            align: PrintAlign::Left,
        },
    ];
    let rows = vec![
        vec![cell("2025-04-01"), cell(""), cell("Coffee")],
        vec![cell("2025-04-02"), cell("   "), cell("Groceries")],
    ];

    let (columns, rows) = compact_print_table(&columns, &rows);

    assert_eq!(
        columns
            .iter()
            .map(|column| column.title.as_str())
            .collect::<Vec<_>>(),
        ["Date", "Description"]
    );
    assert_eq!(rows[0][1].text, "Coffee");
    assert_eq!(rows[1][1].text, "Groceries");
}

#[test]
fn paginated_tables_keep_rows_inside_printable_area() {
    let report = PrintReport {
        title: "Test".into(),
        subtitle: "Print layout".into(),
        generated: "Generated".into(),
        metrics: Vec::new(),
        sections: vec![PrintSection::Table {
            title: "Rows".into(),
            subtitle: String::new(),
            columns: vec![
                PrintColumn {
                    title: "Description".into(),
                    width: 2.0,
                    align: PrintAlign::Left,
                },
                PrintColumn {
                    title: "Amount".into(),
                    width: 1.0,
                    align: PrintAlign::Right,
                },
            ],
            rows: (0..40)
                .map(|index| {
                    vec![
                        PrintCell {
                            text: format!("Row {index}"),
                            tone: PrintTone::Normal,
                        },
                        PrintCell {
                            text: "1.00".into(),
                            tone: PrintTone::Normal,
                        },
                    ]
                })
                .collect(),
        }],
    };

    let width = 420.0;
    let height = 320.0;
    let bottom = print_content_bottom(height);
    let pages = paginate_print_report(&report, width, height);

    assert!(pages.len() > 1);
    for page in pages {
        let used_height = page
            .elements
            .iter()
            .map(|element| print_element_height(element, width))
            .sum::<f64>();
        assert!(print_content_top() + used_height <= bottom);
    }
}
