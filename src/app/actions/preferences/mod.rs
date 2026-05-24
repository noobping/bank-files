use super::helpers::add_bool_toggle_action;
#[cfg(feature = "smart-insights")]
use super::helpers::set_simple_action_enabled;
use super::*;

mod features;
mod modes;
mod smart;

use features::register_feature_preference_actions;
use modes::register_mode_preference_actions;
use smart::register_smart_preference_actions;

pub(super) fn register_preference_actions(
    app: &adw::Application,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    register_mode_preference_actions(app, state, ui);
    register_feature_preference_actions(app, state, ui);
    register_smart_preference_actions(app, state, ui);
}
