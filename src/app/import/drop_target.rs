use super::open::open_uris_in_background;
use super::*;

pub(in crate::app) fn connect_drop_target(
    root: &gtk::Box,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let target = gtk::DropTarget::new(
        gtk::gdk::FileList::static_type(),
        gtk::gdk::DragAction::COPY,
    );
    let state_for_drop = Rc::clone(state);
    let ui_for_drop = Rc::clone(ui);
    target.connect_drop(move |_, value, _, _| {
        let Ok(file_list) = value.get::<gtk::gdk::FileList>() else {
            show_status(&ui_for_drop, "Drop contains no readable files.");
            return false;
        };
        let uris = file_list
            .files()
            .iter()
            .map(|file| file.uri().to_string())
            .collect::<Vec<_>>();
        if uris.is_empty() {
            show_status(&ui_for_drop, "Drop contains no files.");
            return true;
        }

        show_status(&ui_for_drop, "Dropped bank files. Opening files...");

        let state_for_import = Rc::clone(&state_for_drop);
        let ui_for_import = Rc::clone(&ui_for_drop);
        gtk::glib::MainContext::default().spawn_local(async move {
            open_uris_in_background(uris, state_for_import, ui_for_import).await;
        });
        true
    });

    root.add_controller(target);
}

pub(in crate::app) fn import_uris_into_session(
    uris: Vec<String>,
    state: Rc<RefCell<AppData>>,
    ui: Rc<UiHandles>,
) {
    show_status(&ui, "Bank files received. Opening files...");
    gtk::glib::MainContext::default().spawn_local(async move {
        open_uris_in_background(uris, state, ui).await;
    });
}
