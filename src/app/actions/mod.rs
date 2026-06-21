use super::*;
use adw::glib::variant::{StaticVariantType, ToVariant};

mod data;
mod dialogs;
mod helpers;
#[cfg(all(target_os = "linux", feature = "setup", not(feature = "flatpak")))]
mod local_setup;
mod navigation;
mod page;
mod preferences;

use data::register_data_actions;
use dialogs::register_dialog_actions;
use helpers::add_view_action;
#[cfg(all(target_os = "linux", feature = "setup", not(feature = "flatpak")))]
use local_setup::register_local_setup_action;
use navigation::register_navigation_actions;
use page::register_page_actions;
use preferences::register_preference_actions;

pub(in crate::app) fn connect_actions(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    import_button: gtk::Button,
    menu_button: gtk::MenuButton,
) {
    #[cfg(not(all(target_os = "linux", feature = "setup", not(feature = "flatpak"))))]
    let _ = menu_button;

    install_action_accelerators(app);
    register_data_actions(app, window, state, ui, import_button);
    register_navigation_actions(app, state, ui);
    register_dialog_actions(app, window, state, ui);
    register_view_actions(app, ui);
    register_page_actions(app, state, ui);
    register_preference_actions(app, state, ui);

    #[cfg(all(target_os = "linux", feature = "setup", not(feature = "flatpak")))]
    register_local_setup_action(app, state, ui, menu_button);

    updater::register_app_actions(app);
    refresh_write_actions(ui.as_ref());
    refresh_menu(ui, &state.borrow());
    connect_search(state, ui);
}

fn register_view_actions(app: &adw::Application, ui: &Rc<UiHandles>) {
    add_view_action(app, ui, "view-overview", "overview");
    add_view_action(app, ui, "view-budget", "categories");
    add_view_action(app, ui, "view-transactions", "transactions");
    add_view_action(app, ui, "view-diagnostics", "debug");
}
