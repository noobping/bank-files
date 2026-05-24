use super::*;

#[derive(Clone)]
pub(in crate::app) struct ActiveSession {
    pub(in crate::app) state: Rc<RefCell<AppData>>,
    pub(in crate::app) ui: Rc<UiHandles>,
}

thread_local! {
    pub(in crate::app) static ACTIVE_SESSION: RefCell<Option<ActiveSession>> = const { RefCell::new(None) };
}
