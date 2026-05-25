use super::helpers::add_bool_toggle_action;
use super::*;

mod features;
mod modes;

use features::register_feature_preference_actions;
use modes::register_mode_preference_actions;

pub(super) fn register_preference_actions(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    register_mode_preference_actions(app, state, ui);
    register_feature_preference_actions(app, state, ui);
}
