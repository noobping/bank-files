use super::super::*;
use super::card::collapsible_form_card;
use super::state::{
    attach_details_grid, connect_combo_summary, connect_delete_button, connect_entry_summary,
    set_option_combo, set_summary,
};
use super::summaries::alias_summary;
use super::values::alias_value;

pub(in crate::app) fn append_alias_form(
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<AliasForm>>>,
    alias: EditableAlias,
) {
    let card = collapsible_form_card("Field name", "", "Delete field name");

    let grid = form_grid();
    let canonical = field_alias_combo(&alias.canonical);
    let alias_entry = entry(&alias.alias, "Column name from bank CSV");
    add_labeled(&grid, 0, "Fixed field", &canonical);
    add_labeled(&grid, 1, "Bank column", &alias_entry);
    attach_details_grid(&card, &grid);

    let update_summary: Rc<dyn Fn()> = {
        let title = card.title.clone();
        let subtitle = card.subtitle.clone();
        let canonical = canonical.clone();
        let alias_entry = alias_entry.clone();
        Rc::new(move || set_summary(&title, &subtitle, alias_summary(&canonical, &alias_entry)))
    };
    connect_combo_summary(&canonical, &update_summary);
    connect_entry_summary(&alias_entry, &update_summary);
    update_summary();

    let original_alias = alias_value(&canonical, &alias_entry);
    let update_revert_state: Rc<dyn Fn()> = {
        let revert_button = card.revert_button.clone();
        let original_alias = original_alias.clone();
        let canonical = canonical.clone();
        let alias_entry = alias_entry.clone();
        Rc::new(move || {
            revert_button.set_sensitive(alias_value(&canonical, &alias_entry) != original_alias);
        })
    };
    connect_combo_summary(&canonical, &update_revert_state);
    connect_entry_summary(&alias_entry, &update_revert_state);
    update_revert_state();

    let update_for_revert = Rc::clone(&update_summary);
    let update_revert_for_revert = Rc::clone(&update_revert_state);
    let canonical_for_revert = canonical.clone();
    let alias_for_revert = alias_entry.clone();
    card.revert_button.connect_clicked(move |_| {
        set_option_combo(&canonical_for_revert, &original_alias.canonical);
        alias_for_revert.set_text(&original_alias.alias);
        update_for_revert();
        update_revert_for_revert();
    });

    let deleted = Rc::new(Cell::new(false));
    connect_delete_button(&card.delete_button, &deleted, &card.form_box);

    container.append(&card.form_box);
    forms.borrow_mut().push(AliasForm {
        form_box: card.form_box,
        deleted,
        canonical,
        alias: alias_entry,
    });
}
