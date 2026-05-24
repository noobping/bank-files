use super::*;

pub(in crate::app::transactions::common) fn category_combo(
    data: &AppData,
    selected: &str,
) -> gtk::ComboBoxText {
    ui::text_combo(selected, app_category_values(data))
}

pub(in crate::app::transactions::common) fn budget_code_combo(
    data: &AppData,
    selected: &str,
) -> gtk::ComboBoxText {
    ui::text_combo(selected, app_budget_code_values(data))
}
