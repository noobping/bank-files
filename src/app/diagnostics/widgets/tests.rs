use super::file_actions::csv_file_action_available;

#[test]
fn csv_file_actions_are_disabled_while_loading() {
    assert!(csv_file_action_available(0));
    assert!(!csv_file_action_available(1));
    assert!(!csv_file_action_available(3));
}
