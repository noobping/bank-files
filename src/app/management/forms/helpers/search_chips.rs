use super::*;

const RESOURCE: &str = "rule-search-chips.ui";

type SearchChips = Rc<RefCell<Vec<String>>>;

pub(in crate::app) struct RuleSearchChipsEditor {
    pub(in crate::app) container: gtk::Box,
    pub(in crate::app) entry: adw::EntryRow,
}

impl RuleSearchChipsEditor {
    pub(in crate::app) fn focus_entry(&self) {
        self.entry.grab_focus();
    }
}

pub(in crate::app) fn rule_search_chips_editor(
    rule: &EditableRule,
    search: &gtk::TextView,
    is_regex: &gtk::Switch,
) -> RuleSearchChipsEditor {
    let builder = ui::builder_from_resource(RESOURCE);
    let container = ui::builder_object::<gtk::Box>(&builder, "rule_search_chips_editor", RESOURCE);
    let wrap = ui::builder_object::<adw::WrapBox>(&builder, "rule_search_chips_wrap", RESOURCE);
    let entry = ui::builder_object::<adw::EntryRow>(&builder, "rule_search_chips_entry", RESOURCE);
    let terms = Rc::new(RefCell::new(initial_search_chips(rule)));
    let sync_chips = sync_rule_search_chips(Rc::clone(&terms), search.clone(), is_regex.clone());
    render_search_chips(&wrap, &terms, &sync_chips);

    entry.connect_changed(|entry| {
        entry.set_show_apply_button(!entry.text().trim().is_empty());
    });

    let wrap_for_apply = wrap.clone();
    let terms_for_apply = Rc::clone(&terms);
    let sync_for_apply = Rc::clone(&sync_chips);
    entry.connect_apply(move |entry| {
        add_entry_search_chip(entry, &wrap_for_apply, &terms_for_apply, &sync_for_apply);
    });

    RuleSearchChipsEditor { container, entry }
}

fn initial_search_chips(rule: &EditableRule) -> Vec<String> {
    data::editable_rule_literal_terms(rule).unwrap_or_else(|| fallback_search_chip(&rule.search))
}

fn fallback_search_chip(search: &str) -> Vec<String> {
    let search = search.trim();
    if search.is_empty() {
        Vec::new()
    } else {
        vec![search.to_string()]
    }
}

fn add_entry_search_chip(
    entry: &adw::EntryRow,
    wrap: &adw::WrapBox,
    terms: &SearchChips,
    sync_chips: &Rc<dyn Fn()>,
) {
    let term = normalize_search_chip(&entry.text());
    if term.is_empty()
        || terms
            .borrow()
            .iter()
            .any(|existing| same_search_chip(existing, &term))
    {
        reset_search_chip_entry(entry);
        return;
    }

    terms.borrow_mut().push(term);
    reset_search_chip_entry(entry);
    render_search_chips(wrap, terms, sync_chips);
    sync_chips();
}

fn reset_search_chip_entry(entry: &adw::EntryRow) {
    entry.set_text("");
    entry.set_show_apply_button(false);
    entry.grab_focus();
}

fn render_search_chips(wrap: &adw::WrapBox, terms: &SearchChips, sync_chips: &Rc<dyn Fn()>) {
    wrap.remove_all();
    for (index, term) in terms.borrow().iter().enumerate() {
        let chip = search_chip(term);
        let wrap_for_remove = wrap.clone();
        let terms_for_remove = Rc::clone(terms);
        let sync_for_remove = Rc::clone(sync_chips);
        chip.remove_button.connect_clicked(move |_| {
            remove_search_chip(index, &wrap_for_remove, &terms_for_remove, &sync_for_remove);
        });
        wrap.append(&chip.container);
    }
}

fn remove_search_chip(
    index: usize,
    wrap: &adw::WrapBox,
    terms: &SearchChips,
    sync_chips: &Rc<dyn Fn()>,
) {
    if index < terms.borrow().len() {
        terms.borrow_mut().remove(index);
        render_search_chips(wrap, terms, sync_chips);
        sync_chips();
    }
}

struct SearchChip {
    container: gtk::Box,
    remove_button: gtk::Button,
}

fn search_chip(term: &str) -> SearchChip {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    container.add_css_class("card");
    container.set_valign(gtk::Align::Center);

    let label = gtk::Label::new(Some(term));
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    label.set_margin_start(8);
    label.set_xalign(0.0);
    container.append(&label);

    let remove_button = gtk::Button::builder()
        .icon_name("window-close-symbolic")
        .tooltip_text(tr("Remove"))
        .valign(gtk::Align::Center)
        .margin_end(4)
        .build();
    remove_button.add_css_class("flat");
    container.append(&remove_button);

    SearchChip {
        container,
        remove_button,
    }
}

fn sync_rule_search_chips(
    terms: SearchChips,
    search: gtk::TextView,
    is_regex: gtk::Switch,
) -> Rc<dyn Fn()> {
    Rc::new(move || {
        let terms = terms.borrow();
        let (search_text, regex) = data::rule_search_from_literal_terms(&terms);
        set_rule_search_text(&search, &search_text);
        is_regex.set_active(regex);
    })
}

fn normalize_search_chip(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn same_search_chip(left: &str, right: &str) -> bool {
    left.to_lowercase() == right.to_lowercase()
}
