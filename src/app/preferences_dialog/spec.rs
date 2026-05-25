use super::*;

pub(super) struct PreferenceSpec<'a> {
    pub(super) title: &'a str,
    pub(super) subtitle: &'a str,
    pub(super) action_name: &'a str,
    pub(super) active: bool,
    pub(super) visibility_target: Option<gtk::Widget>,
    pub(super) visibility_gate: Option<Rc<Cell<bool>>>,
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
            visibility_target: None,
            visibility_gate: None,
        }
    }
}
