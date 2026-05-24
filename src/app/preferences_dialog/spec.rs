use super::group::{preference_row_enabled, preference_row_visible};
use super::*;

pub(super) struct PreferenceSpec<'a> {
    pub(super) title: &'a str,
    pub(super) subtitle: &'a str,
    pub(super) action_name: &'a str,
    pub(super) active: bool,
    pub(super) requires_smart_insights: bool,
    pub(super) visibility_target: Option<gtk::Widget>,
    pub(super) visibility_gate: Option<Rc<Cell<bool>>>,
    pub(super) enabled_controller_gate: Option<Rc<Cell<bool>>>,
    pub(super) enabled_by_gate: Option<Rc<Cell<bool>>>,
}

impl<'a> PreferenceSpec<'a> {
    pub(super) fn new(
        title: &'a str,
        subtitle: &'a str,
        action_name: &'a str,
        active: bool,
    ) -> Self {
        Self {
            title,
            subtitle,
            action_name,
            active,
            requires_smart_insights: false,
            visibility_target: None,
            visibility_gate: None,
            enabled_controller_gate: None,
            enabled_by_gate: None,
        }
    }

    #[cfg(feature = "smart-insights")]
    pub(super) fn enabled_by(mut self, gate: Rc<Cell<bool>>) -> Self {
        self.enabled_by_gate = Some(gate);
        self
    }

    #[cfg(feature = "smart-insights")]
    pub(super) fn toggles_enabled(mut self, gate: Rc<Cell<bool>>) -> Self {
        self.enabled_controller_gate = Some(gate);
        self
    }

    pub(super) fn toggles_visibility(
        mut self,
        target: &impl IsA<gtk::Widget>,
        gate: Rc<Cell<bool>>,
    ) -> Self {
        self.visibility_target = Some(target.clone().upcast::<gtk::Widget>());
        self.visibility_gate = Some(gate);
        self
    }

    #[cfg(feature = "smart-insights")]
    pub(super) fn requires_smart_insights(mut self) -> Self {
        self.requires_smart_insights = true;
        self
    }

    pub(super) fn visible(
        &self,
        writable: bool,
        advanced_features: bool,
        smart_insights_enabled: bool,
    ) -> bool {
        preference_row_visible(
            writable,
            advanced_features,
            smart_insights_enabled,
            self.requires_smart_insights,
        )
    }

    pub(super) fn sensitive(&self, writable: bool, smart_insights_enabled: bool) -> bool {
        preference_row_enabled(
            writable,
            smart_insights_enabled,
            self.requires_smart_insights,
        )
    }
}
