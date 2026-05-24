use super::*;

pub fn builder_from_resource(file_name: &str) -> gtk::Builder {
    let builder = gtk::Builder::new();
    let path = format!("{}/ui/{file_name}", crate::app_info::RESOURCE_ID);
    if let Err(err) = builder.add_from_resource(&path) {
        panic!("Failed to load GTK UI resource {path}: {err}");
    }
    translate_builder_objects(&builder);
    builder
}

pub fn builder_object<T: IsA<gtk::glib::Object>>(
    builder: &gtk::Builder,
    id: &str,
    resource: &str,
) -> T {
    let Some(object) = builder.object::<T>(id) else {
        panic!("{resource} should define {id}");
    };
    object
}

fn translate_builder_objects(builder: &gtk::Builder) {
    for object in builder.objects() {
        translate_builder_object(&object);
    }
}

fn translate_builder_object(object: &gtk::glib::Object) {
    if let Some(widget) = object.downcast_ref::<gtk::Widget>() {
        translate_widget_tooltip(widget);
    }
    if let Some(label) = object.downcast_ref::<gtk::Label>() {
        translate_label(label);
    }
    if let Some(button) = object.downcast_ref::<gtk::Button>() {
        translate_button(button);
    }
    if let Some(entry) = object.downcast_ref::<gtk::Entry>() {
        translate_entry(entry);
    }
    if let Some(entry) = object.downcast_ref::<gtk::SearchEntry>() {
        translate_search_entry(entry);
    }
    if let Some(content) = object.downcast_ref::<adw::ButtonContent>() {
        translate_button_content(content);
    }
    if let Some(row) = object.downcast_ref::<adw::ActionRow>() {
        translate_action_row(row);
    }
    if let Some(title) = object.downcast_ref::<adw::WindowTitle>() {
        translate_window_title(title);
    }
    if let Some(page) = object.downcast_ref::<adw::ViewStackPage>() {
        translate_view_stack_page(page);
    }
    if let Some(menu) = object.downcast_ref::<gtk::gio::Menu>() {
        translate_menu(menu);
    }
}

fn translate_widget_tooltip(widget: &gtk::Widget) {
    if let Some(text) = widget.tooltip_text() {
        widget.set_tooltip_text(Some(&translate_text(&text)));
    }
}

fn translate_label(label: &gtk::Label) {
    let text = label.label();
    if !text.is_empty() {
        label.set_label(&translate_text(&text));
    }
}

fn translate_button(button: &gtk::Button) {
    if let Some(text) = button.label() {
        button.set_label(&translate_text(&text));
    }
}

fn translate_entry(entry: &gtk::Entry) {
    if let Some(text) = entry.placeholder_text() {
        entry.set_placeholder_text(Some(&translate_text(&text)));
    }
}

fn translate_search_entry(entry: &gtk::SearchEntry) {
    if let Some(text) = entry.placeholder_text() {
        entry.set_placeholder_text(Some(&translate_text(&text)));
    }
}

fn translate_button_content(content: &adw::ButtonContent) {
    let label = content.label();
    if !label.is_empty() {
        content.set_label(&translate_text(&label));
    }
}

fn translate_action_row(row: &adw::ActionRow) {
    let title = row.title();
    if !title.is_empty() {
        row.set_title(&translate_text(&title));
    }
    if let Some(subtitle) = row.subtitle() {
        if !subtitle.is_empty() {
            row.set_subtitle(&translate_text(&subtitle));
        }
    }
}

fn translate_window_title(title: &adw::WindowTitle) {
    let text = title.title();
    if !text.is_empty() {
        title.set_title(&translate_text(&text));
    }
    let subtitle = title.subtitle();
    if !subtitle.is_empty() {
        title.set_subtitle(&translate_text(&subtitle));
    }
}

fn translate_view_stack_page(page: &adw::ViewStackPage) {
    if let Some(title) = page.title() {
        if !title.is_empty() {
            page.set_title(Some(&translate_text(&title)));
        }
    }
}

fn translate_menu(menu: &gtk::gio::Menu) {
    let translated_items = (0..menu.n_items())
        .map(|index| translated_menu_item(menu, index))
        .collect::<Vec<_>>();
    menu.remove_all();
    for item in translated_items {
        menu.append_item(&item);
    }
}

fn translated_menu_model(model: &gtk::gio::MenuModel) -> gtk::gio::Menu {
    let menu = gtk::gio::Menu::new();
    for index in 0..model.n_items() {
        menu.append_item(&translated_menu_item(model, index));
    }
    menu
}

fn translated_menu_item(model: &impl IsA<gtk::gio::MenuModel>, index: i32) -> gtk::gio::MenuItem {
    let item = gtk::gio::MenuItem::new(None, None);

    let attributes = model.iterate_item_attributes(index);
    while let Some((name, value)) = attributes.next() {
        if name.as_str() == "label" {
            if let Some(label) = value.str() {
                item.set_label(Some(&translate_text(label)));
                continue;
            }
        }
        item.set_attribute_value(name.as_str(), Some(&value));
    }

    let links = model.iterate_item_links(index);
    while let Some((name, link)) = links.next() {
        let translated = translated_menu_model(&link);
        item.set_link(name.as_str(), Some(&translated));
    }

    item
}

fn translate_text(text: &str) -> String {
    gettext(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_window_resource_builds_expected_objects() {
        gtk::init().expect("initialize GTK");
        crate::resources::register().expect("register embedded resources");
        let builder = builder_from_resource("main-window.ui");

        let _: gtk::Box = builder_object(&builder, "main_root", "main-window.ui");
        let _: adw::ViewStack = builder_object(&builder, "main_stack", "main-window.ui");
        let _: gtk::SearchEntry = builder_object(&builder, "main_search_entry", "main-window.ui");
    }

    #[test]
    fn translated_menu_keeps_actions_and_translates_labels() {
        let menu = gtk::gio::Menu::new();
        menu.append(Some("Save"), Some("app.save"));

        translate_menu(&menu);

        let label = menu
            .item_attribute_value(0, "label", None)
            .and_then(|value| value.get::<String>());
        let action = menu
            .item_attribute_value(0, "action", None)
            .and_then(|value| value.get::<String>());

        let expected_label = gettext("Save");
        assert_eq!(label.as_deref(), Some(expected_label.as_str()));
        assert_eq!(action.as_deref(), Some("app.save"));
    }

    #[test]
    fn strings_without_translation_are_left_as_is() {
        assert_eq!(translate_text("not in the catalog"), "not in the catalog");
    }
}
