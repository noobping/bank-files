use super::*;

const LIST_PAGE: &str = "list";
const FORM_PAGE: &str = "form";

pub(in crate::app) struct ActionDialogShell {
    pub(in crate::app) root: gtk::Box,
    pub(in crate::app) back_button: gtk::Button,
    pub(in crate::app) submit_button: gtk::Button,
    pub(in crate::app) close_button: gtk::Button,
    pub(in crate::app) search_bar: gtk::SearchBar,
    pub(in crate::app) search_entry: gtk::SearchEntry,
    pub(in crate::app) stack: gtk::Stack,
    start_stack: gtk::Stack,
}

impl ActionDialogShell {
    pub(in crate::app) fn set_list_page(&self) {
        self.stack.set_visible_child_name(LIST_PAGE);
        self.start_stack.set_visible_child_name("search");
    }

    pub(in crate::app) fn set_form_only(&self) {
        self.search_bar.set_search_mode(false);
        self.start_stack.set_visible_child_name("empty");
    }

    pub(in crate::app) fn add_list_page(&self, child: &impl IsA<gtk::Widget>) {
        self.stack.add_named(child, Some(LIST_PAGE));
    }

    pub(in crate::app) fn add_form_page(&self, child: &impl IsA<gtk::Widget>) {
        self.stack.add_named(child, Some(FORM_PAGE));
    }

    pub(in crate::app) fn page_handle(&self) -> ActionDialogPageHandle {
        ActionDialogPageHandle {
            stack: self.stack.clone(),
            start_stack: self.start_stack.clone(),
            search_bar: self.search_bar.clone(),
        }
    }
}

#[derive(Clone)]
pub(in crate::app) struct ActionDialogPageHandle {
    stack: gtk::Stack,
    start_stack: gtk::Stack,
    search_bar: gtk::SearchBar,
}

impl ActionDialogPageHandle {
    pub(in crate::app) fn set_list_page(&self) {
        self.stack.set_visible_child_name(LIST_PAGE);
        self.start_stack.set_visible_child_name("search");
    }

    pub(in crate::app) fn set_form_page(&self) {
        self.search_bar.set_search_mode(false);
        self.stack.set_visible_child_name(FORM_PAGE);
        self.start_stack.set_visible_child_name("back");
    }
}

pub(in crate::app) fn build_action_dialog_shell(
    title: &str,
    subtitle: &str,
    submit_label: &str,
    submit_icon_name: &str,
    submit_tooltip: &str,
    search_placeholder: &str,
) -> ActionDialogShell {
    let builder = ui::builder_from_resource("action-dialog.ui");
    let root = builder
        .object::<gtk::Box>("action_root")
        .expect("action-dialog.ui should define action_root");
    let header = builder
        .object::<adw::HeaderBar>("action_header")
        .expect("action-dialog.ui should define action_header");
    header.set_title_widget(Some(&adw::WindowTitle::new(&tr(title), &tr(subtitle))));

    let start_stack = builder
        .object::<gtk::Stack>("action_start_stack")
        .expect("action-dialog.ui should define action_start_stack");
    let search_button = builder
        .object::<gtk::Button>("action_search_button")
        .expect("action-dialog.ui should define action_search_button");
    search_button.set_tooltip_text(Some(&tr("Search")));
    let back_button = builder
        .object::<gtk::Button>("action_back_button")
        .expect("action-dialog.ui should define action_back_button");
    back_button.set_tooltip_text(Some(&tr("Back")));

    let submit_button = builder
        .object::<gtk::Button>("action_submit_button")
        .expect("action-dialog.ui should define action_submit_button");
    let submit_content = builder
        .object::<adw::ButtonContent>("action_submit_content")
        .expect("action-dialog.ui should define action_submit_content");
    submit_content.set_label(&tr(submit_label));
    submit_content.set_icon_name(submit_icon_name);
    submit_button.set_tooltip_text(Some(&tr(submit_tooltip)));

    let close_button = builder
        .object::<gtk::Button>("action_close_button")
        .expect("action-dialog.ui should define action_close_button");
    close_button.set_tooltip_text(Some(&tr("Close")));

    let search_bar = builder
        .object::<gtk::SearchBar>("action_search_bar")
        .expect("action-dialog.ui should define action_search_bar");
    let search_entry = builder
        .object::<gtk::SearchEntry>("action_search_entry")
        .expect("action-dialog.ui should define action_search_entry");
    search_entry.set_placeholder_text(Some(&tr(search_placeholder)));
    search_bar.connect_entry(&search_entry);
    ui::connect_search_button(&search_button, &search_bar, &search_entry);

    let stack = builder
        .object::<gtk::Stack>("action_stack")
        .expect("action-dialog.ui should define action_stack");

    ActionDialogShell {
        root,
        back_button,
        submit_button,
        close_button,
        search_bar,
        search_entry,
        stack,
        start_stack,
    }
}

#[derive(Clone)]
pub(in crate::app) struct SearchableActionRow {
    pub(in crate::app) widget: gtk::Widget,
    keywords: String,
}

pub(in crate::app) fn searchable_action_row(
    row: &impl IsA<gtk::Widget>,
    title: &str,
    subtitle: &str,
    extra_keywords: &[String],
) -> SearchableActionRow {
    SearchableActionRow {
        widget: row.clone().upcast::<gtk::Widget>(),
        keywords: action_search_keywords(title, subtitle, extra_keywords),
    }
}

pub(in crate::app) fn connect_action_search(
    search_entry: &gtk::SearchEntry,
    rows: Vec<SearchableActionRow>,
    empty_widget: Option<gtk::Widget>,
) {
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().trim().to_lowercase();
        let show_all = query.is_empty();
        let mut visible_count = 0usize;

        for row in &rows {
            let visible = show_all || row.keywords.contains(&query);
            row.widget.set_visible(visible);
            if visible {
                visible_count += 1;
            }
        }

        if let Some(empty_widget) = &empty_widget {
            empty_widget.set_visible(visible_count == 0);
        }
    });
}

fn action_search_keywords(title: &str, subtitle: &str, extra_keywords: &[String]) -> String {
    let mut values = vec![
        title.to_string(),
        subtitle.to_string(),
        tr(title),
        tr(subtitle),
    ];
    values.extend(extra_keywords.iter().cloned());
    values.join(" ").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_search_keywords_include_extra_values() {
        let keywords = action_search_keywords(
            "Groceries",
            "Expenses",
            &["FOOD".to_string(), "Rule match".to_string()],
        );
        assert!(keywords.contains("groceries"));
        assert!(keywords.contains("expenses"));
        assert!(keywords.contains("food"));
        assert!(keywords.contains("rule match"));
    }
}
