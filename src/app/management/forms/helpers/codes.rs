use super::*;

pub(super) fn budget_code_key(code: &str) -> String {
    code.trim().to_ascii_lowercase()
}

pub(super) fn renamed_budget_code<'a>(
    code: &str,
    renames: &'a [BudgetCodeRename],
) -> Option<&'a str> {
    let key = budget_code_key(code);
    if key.is_empty() {
        return None;
    }
    renames
        .iter()
        .find(|rename| budget_code_key(&rename.from) == key)
        .map(|rename| rename.to.as_str())
}

pub(in crate::app) fn set_text_combo(combo: &gtk::ComboBoxText, value: &str) {
    let value = value.trim();
    combo.set_active_id(if value.is_empty() { None } else { Some(value) });
    if let Some(entry) = combo
        .child()
        .and_then(|child| child.downcast::<gtk::Entry>().ok())
    {
        entry.set_text(value);
    }
}
