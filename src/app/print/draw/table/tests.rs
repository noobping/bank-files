use super::*;

#[test]
fn print_wrapping_splits_long_words() {
    assert_eq!(
        wrap_text_for_print("abcdefghij", 4),
        vec!["abcd".to_string(), "efgh".to_string(), "ij".to_string()]
    );
}

#[test]
fn table_row_height_grows_for_wrapped_cells() {
    let columns = vec![PrintColumn {
        title: "Message".to_string(),
        width: 1.0,
        align: PrintAlign::Left,
    }];
    let cells = vec![PrintCell {
        text: "This is a longer message that needs more than one printed line".to_string(),
        tone: PrintTone::Normal,
    }];

    assert!(print_table_row_height(&columns, &cells, 120.0) > PRINT_TABLE_ROW_HEIGHT);
}
