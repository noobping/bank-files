use super::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::app) enum OperationSource {
    CreateRule,
    ChangeBudgetCode,
    MarkTransfer,
    MarkInvalid,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(in crate::app) enum QueuedOperationStatus {
    Pending,
    Applying,
    Applied,
    Failed(String),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::app) enum EnqueueOperationResult {
    Queued(u64),
    AlreadyQueued(u64),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(in crate::app) enum QueuedOperationKind {
    Rule {
        rule: EditableRule,
        ensure_budget: bool,
        source: OperationSource,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(in crate::app) struct QueuedOperation {
    pub(in crate::app) id: u64,
    pub(in crate::app) kind: QueuedOperationKind,
    pub(in crate::app) status: QueuedOperationStatus,
}

#[derive(Clone)]
pub(in crate::app) struct OperationQueue {
    next_id: Rc<Cell<u64>>,
    operations: Rc<RefCell<Vec<QueuedOperation>>>,
    processing: Rc<Cell<bool>>,
    action_registrations: Rc<RefCell<Vec<OperationQueueActionRegistration>>>,
}

#[derive(Clone)]
struct OperationQueueActionRegistration {
    owner: gtk::Widget,
    target: OperationQueueActionTarget,
    kind: QueuedOperationKind,
    base_enabled: bool,
    base_visible: bool,
    base_tooltip: Option<String>,
    was_rooted: Rc<Cell<bool>>,
}

#[derive(Clone)]
enum OperationQueueActionTarget {
    Widget(gtk::Widget),
    MenuAction(gtk::gio::SimpleAction),
}

#[derive(Clone)]
pub(in crate::app) struct OperationQueueWidgets {
    pub(in crate::app) button: gtk::Button,
    pub(in crate::app) badge: gtk::Label,
    pub(in crate::app) summary: gtk::Label,
    pub(in crate::app) apply_all_button: gtk::Button,
    pub(in crate::app) clear_done_button: gtk::Button,
    pub(in crate::app) search_entry: gtk::SearchEntry,
    pub(in crate::app) list: gtk::ListBox,
    pub(in crate::app) dialog: adw::Dialog,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
struct ApplyCounts {
    applied: usize,
    failed: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct RuleCombineSummary {
    before_count: usize,
    after_count: usize,
}

impl OperationQueue {
    pub(in crate::app) fn new() -> Self {
        Self {
            next_id: Rc::new(Cell::new(1)),
            operations: Rc::new(RefCell::new(Vec::new())),
            processing: Rc::new(Cell::new(false)),
            action_registrations: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub(in crate::app) fn enqueue_rule(
        &self,
        rule: EditableRule,
        ensure_budget: bool,
        source: OperationSource,
    ) -> EnqueueOperationResult {
        let kind = QueuedOperationKind::Rule {
            rule,
            ensure_budget,
            source,
        };
        if let Some(id) = self.existing_operation_id(&kind) {
            return EnqueueOperationResult::AlreadyQueued(id);
        }

        let id = self.next_id.get();
        self.next_id.set(id.saturating_add(1));
        self.operations.borrow_mut().push(QueuedOperation {
            id,
            kind,
            status: QueuedOperationStatus::Pending,
        });
        EnqueueOperationResult::Queued(id)
    }

    fn existing_operation_id(&self, kind: &QueuedOperationKind) -> Option<u64> {
        self.operations
            .borrow()
            .iter()
            .find(|operation| operation.kind == *kind)
            .map(|operation| operation.id)
    }

    fn contains_kind(&self, kind: &QueuedOperationKind) -> bool {
        self.existing_operation_id(kind).is_some()
    }

    pub(in crate::app) fn operations(&self) -> Vec<QueuedOperation> {
        self.operations.borrow().clone()
    }

    pub(in crate::app) fn actionable_count(&self) -> usize {
        self.operations
            .borrow()
            .iter()
            .filter(|operation| operation.status.is_actionable())
            .count()
    }

    pub(in crate::app) fn actionable_ids(&self) -> Vec<u64> {
        self.operations
            .borrow()
            .iter()
            .filter(|operation| operation.status.is_actionable())
            .map(|operation| operation.id)
            .collect()
    }

    pub(in crate::app) fn applied_count(&self) -> usize {
        self.operations
            .borrow()
            .iter()
            .filter(|operation| matches!(operation.status, QueuedOperationStatus::Applied))
            .count()
    }

    pub(in crate::app) fn clear_applied(&self) -> usize {
        let mut operations = self.operations.borrow_mut();
        let before = operations.len();
        operations.retain(|operation| !matches!(operation.status, QueuedOperationStatus::Applied));
        before.saturating_sub(operations.len())
    }

    fn is_processing(&self) -> bool {
        self.processing.get()
    }

    fn set_processing(&self, processing: bool) {
        self.processing.set(processing);
    }

    fn operation_kind(&self, id: u64) -> Option<QueuedOperationKind> {
        self.operations
            .borrow()
            .iter()
            .find(|operation| operation.id == id && operation.status.is_actionable())
            .map(|operation| operation.kind.clone())
    }

    fn mark_applying(&self, id: u64) -> bool {
        self.set_status_if(id, QueuedOperationStatus::Applying, |status| {
            status.is_actionable()
        })
    }

    fn mark_applied(&self, id: u64) {
        self.set_status(id, QueuedOperationStatus::Applied);
    }

    fn mark_failed(&self, id: u64, message: String) {
        self.set_status(id, QueuedOperationStatus::Failed(message));
    }

    fn set_status(&self, id: u64, status: QueuedOperationStatus) -> bool {
        self.set_status_if(id, status, |_| true)
    }

    fn set_status_if(
        &self,
        id: u64,
        status: QueuedOperationStatus,
        predicate: impl FnOnce(&QueuedOperationStatus) -> bool,
    ) -> bool {
        let mut operations = self.operations.borrow_mut();
        let Some(operation) = operations.iter_mut().find(|operation| operation.id == id) else {
            return false;
        };
        if !predicate(&operation.status) {
            return false;
        }
        operation.status = status;
        true
    }

    pub(in crate::app) fn remove(&self, id: u64) -> bool {
        let mut operations = self.operations.borrow_mut();
        let Some(index) = operations
            .iter()
            .position(|operation| operation.id == id && operation.status.can_remove())
        else {
            return false;
        };
        operations.remove(index);
        true
    }
}

impl EnqueueOperationResult {
    pub(in crate::app) fn queued(self) -> bool {
        matches!(self, Self::Queued(_))
    }

    #[cfg(test)]
    fn id(self) -> u64 {
        match self {
            Self::Queued(id) | Self::AlreadyQueued(id) => id,
        }
    }
}

impl QueuedOperationStatus {
    fn is_actionable(&self) -> bool {
        matches!(self, Self::Pending | Self::Failed(_))
    }

    fn can_remove(&self) -> bool {
        !matches!(self, Self::Applying)
    }
}

pub(in crate::app) fn build_operation_queue_widgets() -> OperationQueueWidgets {
    let badge = gtk::Label::new(None);
    badge.add_css_class("caption");
    badge.set_visible(false);

    let icon = gtk::Image::from_icon_name("view-list-symbolic");
    let icon_shell = gtk::Overlay::new();
    icon_shell.set_child(Some(&icon));

    let button_content = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    badge.set_halign(gtk::Align::Center);
    badge.set_valign(gtk::Align::Center);
    button_content.append(&badge);
    button_content.append(&icon_shell);

    let button = gtk::Button::builder()
        .tooltip_text(tr("Processing queue"))
        .build();
    button.add_css_class("flat");
    button.set_focus_on_click(false);
    button.set_child(Some(&button_content));

    let shell = build_settings_dialog_shell("Processing Queue", "Search queued operations");
    let root = shell.root;
    let header = shell.header;
    let search_bar = shell.search_bar;
    let search_entry = shell.search_entry;

    let apply_all_button = ui::primary_text_icon_button(
        "object-select-symbolic",
        "Apply all",
        "Apply all pending queued operations",
    );
    apply_all_button.add_css_class("suggested-action");
    let clear_done_button = ui::icon_button("edit-clear-symbolic", "Clear completed operations");
    clear_done_button.add_css_class("flat");
    header.pack_end(&clear_done_button);
    header.pack_end(&apply_all_button);

    let content = ui::page_box();
    let summary = gtk::Label::new(None);
    summary.add_css_class("dim-label");
    summary.set_selectable(false);
    summary.set_xalign(0.0);
    summary.set_width_chars(1);
    summary.set_wrap(true);
    summary.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    content.append(&summary);

    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    list.set_selection_mode(gtk::SelectionMode::None);
    list.set_hexpand(true);
    content.append(&list);
    root.append(&ui::action_dialog_scroll_with_min(&content, 360));

    let dialog = adw::Dialog::builder()
        .title(tr("Processing Queue"))
        .content_width(620)
        .content_height(560)
        .child(&root)
        .build();
    ui::connect_search_shortcut(&dialog, &search_bar, &search_entry);
    search_bar.set_key_capture_widget(Some(&dialog));

    OperationQueueWidgets {
        button,
        badge,
        summary,
        apply_all_button,
        clear_done_button,
        search_entry,
        list,
        dialog,
    }
}

pub(in crate::app) fn update_operation_queue_action_widgets(ui: &UiHandles) {
    ui.operation_queue.update_registered_actions(ui);
}

pub(in crate::app) fn register_operation_queue_widget<W: IsA<gtk::Widget>>(
    ui: &Rc<UiHandles>,
    widget: &W,
    kind: QueuedOperationKind,
) {
    let widget = widget.clone().upcast::<gtk::Widget>();
    let registration = OperationQueueActionRegistration {
        owner: widget.clone(),
        target: OperationQueueActionTarget::Widget(widget.clone()),
        kind,
        base_enabled: widget.is_sensitive(),
        base_visible: widget.is_visible(),
        base_tooltip: widget.tooltip_text().map(|text| text.to_string()),
        was_rooted: Rc::new(Cell::new(widget.root().is_some())),
    };
    apply_operation_queue_action_state(ui.as_ref(), &registration);
    ui.operation_queue
        .action_registrations
        .borrow_mut()
        .push(registration);
}

pub(in crate::app) fn register_operation_queue_menu_action<W: IsA<gtk::Widget>>(
    ui: &Rc<UiHandles>,
    owner: &W,
    action: &gtk::gio::SimpleAction,
    kind: QueuedOperationKind,
) {
    let owner = owner.clone().upcast::<gtk::Widget>();
    let registration = OperationQueueActionRegistration {
        owner: owner.clone(),
        target: OperationQueueActionTarget::MenuAction(action.clone()),
        kind,
        base_enabled: action.is_enabled(),
        base_visible: true,
        base_tooltip: None,
        was_rooted: Rc::new(Cell::new(owner.root().is_some())),
    };
    apply_operation_queue_action_state(ui.as_ref(), &registration);
    ui.operation_queue
        .action_registrations
        .borrow_mut()
        .push(registration);
}

fn operation_queue_action_registration_is_live(
    registration: &OperationQueueActionRegistration,
) -> bool {
    let rooted = registration.owner.root().is_some();
    if rooted {
        registration.was_rooted.set(true);
    }
    rooted || !registration.was_rooted.get()
}

fn operation_queue_action_enabled(base_enabled: bool, queued: bool, config_enabled: bool) -> bool {
    base_enabled && config_enabled && !queued
}

fn apply_operation_queue_action_state(
    ui: &UiHandles,
    registration: &OperationQueueActionRegistration,
) {
    let queued = ui.operation_queue.contains_kind(&registration.kind);
    let config_idle = !ui.management_dialog_active.get() && ui.loading_count.get() == 0;
    match &registration.target {
        OperationQueueActionTarget::Widget(widget) => match config_write_availability(ui) {
            ActionAvailability::Available => {
                let enabled =
                    operation_queue_action_enabled(registration.base_enabled, queued, config_idle);
                widget.set_visible(registration.base_visible);
                widget.set_sensitive(enabled);
                if queued {
                    widget.set_tooltip_text(Some(&tr(
                        "This operation is already in the processing queue.",
                    )));
                } else {
                    widget.set_tooltip_text(registration.base_tooltip.as_deref());
                }
            }
            ActionAvailability::Hidden => {
                widget.set_visible(false);
                widget.set_sensitive(false);
                widget.set_tooltip_text(None);
            }
            ActionAvailability::Disabled(reason) => {
                widget.set_visible(registration.base_visible);
                widget.set_sensitive(false);
                widget.set_tooltip_text(Some(&tr(&reason)));
            }
        },
        OperationQueueActionTarget::MenuAction(action) => {
            let config_enabled =
                matches!(config_write_availability(ui), ActionAvailability::Available)
                    && config_idle;
            action.set_enabled(operation_queue_action_enabled(
                registration.base_enabled,
                queued,
                config_enabled,
            ));
        }
    }
}

impl OperationQueue {
    fn update_registered_actions(&self, ui: &UiHandles) {
        let mut registrations = self.action_registrations.borrow_mut();
        registrations.retain(operation_queue_action_registration_is_live);
        for registration in registrations.iter() {
            apply_operation_queue_action_state(ui, registration);
        }
    }
}

pub(in crate::app) fn connect_operation_queue(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
    let state_for_dialog = Rc::clone(state);
    let ui_for_dialog = Rc::clone(ui);
    let window_for_dialog = ui.window.clone();
    let dialog_for_button = ui.operation_queue_widgets.dialog.clone();
    ui.operation_queue_widgets.button.connect_clicked(move |_| {
        refresh_operation_queue_ui(&state_for_dialog, &ui_for_dialog);
        dialog_for_button.present(Some(&window_for_dialog));
    });

    let state_for_apply_all = Rc::clone(state);
    let ui_for_apply_all = Rc::clone(ui);
    ui.operation_queue_widgets
        .apply_all_button
        .connect_clicked(move |_| apply_all(&state_for_apply_all, &ui_for_apply_all));

    let state_for_clear_done = Rc::clone(state);
    let ui_for_clear_done = Rc::clone(ui);
    ui.operation_queue_widgets
        .clear_done_button
        .connect_clicked(move |_| clear_done(&state_for_clear_done, &ui_for_clear_done));

    let state_for_search = Rc::clone(state);
    let ui_for_search = Rc::clone(ui);
    ui.operation_queue_widgets
        .search_entry
        .connect_search_changed(move |_| {
            refresh_operation_queue_ui(&state_for_search, &ui_for_search)
        });

    refresh_operation_queue_ui(state, ui);
}

pub(in crate::app) fn enqueue_rule_operation(
    ui: &Rc<UiHandles>,
    rule: EditableRule,
    ensure_budget: bool,
    source: OperationSource,
) -> EnqueueOperationResult {
    let result = ui.operation_queue.enqueue_rule(rule, ensure_budget, source);
    refresh_operation_queue_ui_for_active_session(ui);
    if result.queued() {
        show_status(ui, "Operation added to queue.");
    } else {
        show_status(ui, "Operation is already in the processing queue.");
    }
    result
}

fn refresh_operation_queue_ui_for_active_session(ui: &Rc<UiHandles>) {
    ACTIVE_SESSION.with(|active| {
        if let Some(session) = active.borrow().clone() {
            if Rc::ptr_eq(&session.ui, ui) {
                refresh_operation_queue_ui(&session.state, &session.ui);
            }
        }
    });
}

pub(in crate::app) fn refresh_active_operation_queue_ui() {
    ACTIVE_SESSION.with(|active| {
        if let Some(session) = active.borrow().clone() {
            refresh_operation_queue_ui(&session.state, &session.ui);
        }
    });
}

fn refresh_operation_queue_ui(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
    update_operation_queue_action_widgets(ui.as_ref());
    let widgets = &ui.operation_queue_widgets;
    let actionable = ui.operation_queue.actionable_count();
    let applied = ui.operation_queue.applied_count();
    widgets.badge.set_visible(actionable > 0);
    widgets.badge.set_text(&actionable.to_string());
    set_operation_queue_button_style(&widgets.button, actionable);
    widgets.button.set_tooltip_text(Some(&trf(
        "Processing queue: {count} pending",
        &[("count", actionable.to_string())],
    )));
    let idle = operation_queue_actions_are_idle(
        ui.operation_queue.is_processing(),
        ui.loading_count.get(),
    );
    match config_write_availability(ui.as_ref()) {
        ActionAvailability::Available => {
            widgets.apply_all_button.set_visible(true);
            widgets
                .apply_all_button
                .set_sensitive(actionable > 0 && idle);
            widgets
                .apply_all_button
                .set_tooltip_text(Some(&tr("Apply all pending queued operations")));
        }
        availability => apply_action_availability(&widgets.apply_all_button, &availability),
    }
    widgets.clear_done_button.set_sensitive(applied > 0 && idle);
    widgets.clear_done_button.set_visible(applied > 0);
    widgets
        .summary
        .set_text(&queue_summary(&ui.operation_queue));

    ui::clear_list_box(&widgets.list);
    let operations = ui.operation_queue.operations();
    if operations.is_empty() {
        widgets
            .list
            .append(&queue_text_row(&tr("No queued operations.")));
        return;
    }

    let query = widgets.search_entry.text().trim().to_lowercase();
    let mut visible_count = 0;
    for operation in operations {
        if operation_matches_query(&operation, &query) {
            visible_count += 1;
            widgets.list.append(&operation_row(state, ui, operation));
        }
    }

    if visible_count == 0 {
        widgets
            .list
            .append(&queue_text_row(&tr("No queued operations found.")));
    }
}

fn set_operation_queue_button_style(button: &gtk::Button, actionable: usize) {
    if operation_queue_button_is_suggested(actionable) {
        button.remove_css_class("flat");
        button.add_css_class("suggested-action");
    } else {
        button.remove_css_class("suggested-action");
        button.add_css_class("flat");
    }
}

fn operation_queue_button_is_suggested(actionable: usize) -> bool {
    actionable > 0
}

fn queue_text_row(text: &str) -> adw::ActionRow {
    ui::text_list_row(text)
}

fn operation_matches_query(operation: &QueuedOperation, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let keywords = operation_keywords(operation);
    query.split_whitespace().all(|term| keywords.contains(term))
}

fn operation_keywords(operation: &QueuedOperation) -> String {
    [
        operation_title(&operation.kind),
        operation_status_text(&operation.status),
        operation_subtitle(&operation.kind),
        operation_details(&operation.kind, &operation.status),
    ]
    .join(" ")
    .to_lowercase()
}

fn queue_summary(queue: &OperationQueue) -> String {
    let operations = queue.operations();
    let actionable = operations
        .iter()
        .filter(|operation| operation.status.is_actionable())
        .count();
    let failed = operations
        .iter()
        .filter(|operation| matches!(operation.status, QueuedOperationStatus::Failed(_)))
        .count();
    if operations.is_empty() {
        tr("No pending operations.")
    } else if failed > 0 {
        trf(
            "{count} pending, {failed} failed.",
            &[
                ("count", actionable.to_string()),
                ("failed", failed.to_string()),
            ],
        )
    } else {
        trf(
            "{count} pending operation(s).",
            &[("count", actionable.to_string())],
        )
    }
}

fn operation_row(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    operation: QueuedOperation,
) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::builder()
        .activatable(true)
        .selectable(false)
        .build();
    row.set_tooltip_text(Some(&tr("Show operation details")));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.set_hexpand(true);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(10);
    content.set_margin_end(10);

    let summary = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    summary.set_hexpand(true);

    let details_revealer = gtk::Revealer::new();
    details_revealer.set_transition_type(gtk::RevealerTransitionType::SlideDown);
    details_revealer.set_reveal_child(false);

    let expand_icon = gtk::Image::from_icon_name(operation_details_icon_name(false));
    expand_icon.add_css_class("dim-label");
    expand_icon.set_valign(gtk::Align::Start);
    summary.append(&expand_icon);

    let labels = gtk::Box::new(gtk::Orientation::Vertical, 2);
    labels.set_hexpand(true);
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    header.set_hexpand(true);
    let title = gtk::Label::new(Some(&operation_title(&operation.kind)));
    title.set_selectable(false);
    title.set_xalign(0.0);
    title.set_width_chars(1);
    title.set_max_width_chars(20);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title.set_hexpand(true);
    header.append(&title);

    if matches!(operation.status, QueuedOperationStatus::Applying) {
        let spinner = ui::loading_spinner();
        spinner.set_size_request(16, 16);
        header.append(&spinner);
    }
    let status = gtk::Label::new(Some(&operation_status_text(&operation.status)));
    status.add_css_class("dim-label");
    status.set_selectable(false);
    status.set_xalign(1.0);
    status.set_width_chars(1);
    status.set_max_width_chars(10);
    status.set_ellipsize(gtk::pango::EllipsizeMode::End);
    header.append(&status);
    labels.append(&header);

    let subtitle = gtk::Label::new(Some(&operation_subtitle(&operation.kind)));
    subtitle.set_selectable(false);
    subtitle.add_css_class("dim-label");
    subtitle.set_xalign(0.0);
    subtitle.set_width_chars(1);
    subtitle.set_max_width_chars(34);
    subtitle.set_ellipsize(gtk::pango::EllipsizeMode::End);
    labels.append(&subtitle);

    if let QueuedOperationStatus::Failed(message) = &operation.status {
        let error = gtk::Label::new(Some(message));
        error.set_selectable(false);
        error.add_css_class("error");
        error.set_xalign(0.0);
        error.set_width_chars(1);
        error.set_max_width_chars(34);
        error.set_ellipsize(gtk::pango::EllipsizeMode::End);
        labels.append(&error);
    }
    summary.append(&labels);

    let actions = ui::linked_button_group();
    actions.set_halign(gtk::Align::End);
    actions.set_valign(gtk::Align::Start);
    let apply_button = ui::icon_button("object-select-symbolic", "Apply this operation");
    match config_write_availability(ui.as_ref()) {
        ActionAvailability::Available => {
            apply_button.set_visible(true);
            apply_button.set_sensitive(operation_apply_button_sensitive(
                &operation.status,
                ui.operation_queue.is_processing(),
                ui.loading_count.get(),
            ));
        }
        availability => apply_action_availability(&apply_button, &availability),
    }
    let remove_button = ui::icon_button("user-trash-symbolic", "Remove this operation");
    remove_button.set_sensitive(operation.status.can_remove());

    let id = operation.id;
    let state_for_apply = Rc::clone(state);
    let ui_for_apply = Rc::clone(ui);
    apply_button.connect_clicked(move |_| apply_one(&state_for_apply, &ui_for_apply, id));

    let state_for_remove = Rc::clone(state);
    let ui_for_remove = Rc::clone(ui);
    remove_button.connect_clicked(move |_| {
        if ui_for_remove.operation_queue.remove(id) {
            refresh_operation_queue_ui(&state_for_remove, &ui_for_remove);
            show_status(&ui_for_remove, "Queued operation removed.");
        }
    });

    actions.append(&apply_button);
    actions.append(&remove_button);
    summary.append(&actions);
    content.append(&summary);

    let details = ui::wrapped_label(&operation_details(&operation.kind, &operation.status));
    details.add_css_class("dim-label");
    details.set_width_chars(1);
    details.set_max_width_chars(44);
    details.set_margin_top(6);
    details.set_margin_start(32);
    details_revealer.set_child(Some(&details));
    content.append(&details_revealer);

    let details_revealer_for_activate = details_revealer.clone();
    let expand_icon_for_activate = expand_icon.clone();
    row.connect_activate(move |row| {
        toggle_operation_details(
            row,
            &details_revealer_for_activate,
            &expand_icon_for_activate,
        );
    });

    let click = gtk::GestureClick::new();
    click.set_button(0);
    let row_for_click = row.clone();
    let content_for_click = content.clone();
    let actions_for_click = actions.clone();
    let details_revealer_for_click = details_revealer.clone();
    let expand_icon_for_click = expand_icon.clone();
    click.connect_released(move |_, _, x, y| {
        if !point_is_inside_child(&actions_for_click, &content_for_click, x, y) {
            toggle_operation_details(
                &row_for_click,
                &details_revealer_for_click,
                &expand_icon_for_click,
            );
        }
    });
    content.add_controller(click);

    row.set_child(Some(&content));
    row
}

fn operation_title(kind: &QueuedOperationKind) -> String {
    match kind {
        QueuedOperationKind::Rule { source, .. } => tr(match source {
            OperationSource::CreateRule => "Create rule",
            OperationSource::ChangeBudgetCode => "Change budget code",
            OperationSource::MarkTransfer => "Mark transfer",
            OperationSource::MarkInvalid => "Mark invalid detection",
        }),
    }
}

fn operation_queue_actions_are_idle(processing: bool, loading_count: u32) -> bool {
    !processing && loading_count == 0
}

fn operation_apply_button_sensitive(
    status: &QueuedOperationStatus,
    processing: bool,
    loading_count: u32,
) -> bool {
    status.is_actionable() && operation_queue_actions_are_idle(processing, loading_count)
}

fn operation_subtitle(kind: &QueuedOperationKind) -> String {
    match kind {
        QueuedOperationKind::Rule { rule, .. } => trf(
            "Rule {field}: {search} -> {category} / {code} ({direction})",
            &[
                ("field", tr(rule_field_label(&rule.field))),
                ("search", truncate(&rule.search, 48)),
                ("category", truncate(&rule.category, 32)),
                ("code", truncate(&rule.budget_code, 20)),
                ("direction", tr(rule_direction_label(&rule.direction))),
            ],
        ),
    }
}

fn operation_details_icon_name(expanded: bool) -> &'static str {
    if expanded {
        "pan-down-symbolic"
    } else {
        "pan-end-symbolic"
    }
}

fn toggle_operation_details(
    row: &gtk::ListBoxRow,
    details_revealer: &gtk::Revealer,
    expand_icon: &gtk::Image,
) {
    let expanded = !details_revealer.reveals_child();
    details_revealer.set_reveal_child(expanded);
    expand_icon.set_icon_name(Some(operation_details_icon_name(expanded)));
    row.set_tooltip_text(Some(&tr(if expanded {
        "Hide operation details"
    } else {
        "Show operation details"
    })));
}

fn point_is_inside_child(
    child: &impl IsA<gtk::Widget>,
    target: &impl IsA<gtk::Widget>,
    x: f64,
    y: f64,
) -> bool {
    child
        .as_ref()
        .compute_bounds(target)
        .map(|bounds| {
            let left = f64::from(bounds.x());
            let top = f64::from(bounds.y());
            let right = left + f64::from(bounds.width());
            let bottom = top + f64::from(bounds.height());
            x >= left && x <= right && y >= top && y <= bottom
        })
        .unwrap_or(false)
}

fn operation_details(kind: &QueuedOperationKind, status: &QueuedOperationStatus) -> String {
    let mut lines = Vec::new();
    match kind {
        QueuedOperationKind::Rule {
            rule,
            ensure_budget,
            ..
        } => {
            lines.push(operation_detail_line("Action", operation_title(kind)));
            lines.push(operation_detail_line(
                "Status",
                operation_status_text(status),
            ));
            lines.push(operation_detail_line(
                "Field",
                tr(rule_field_label(&rule.field)),
            ));
            lines.push(operation_detail_line("Match", rule.search.trim()));
            lines.push(operation_detail_line(
                "Regular expression",
                tr(if rule.is_regex { "Yes" } else { "No" }),
            ));
            lines.push(operation_detail_line("Category", rule.category.trim()));
            lines.push(operation_detail_line(
                "Budget code",
                rule.budget_code.trim(),
            ));
            lines.push(operation_detail_line(
                "Direction",
                tr(rule_direction_label(&rule.direction)),
            ));
            if !rule.amount_min.trim().is_empty() {
                lines.push(operation_detail_line(
                    "Minimum amount",
                    rule.amount_min.trim(),
                ));
            }
            if !rule.amount_max.trim().is_empty() {
                lines.push(operation_detail_line(
                    "Maximum amount",
                    rule.amount_max.trim(),
                ));
            }
            if !rule.notes.trim().is_empty() {
                lines.push(operation_detail_line("Notes", rule.notes.trim()));
            }
            lines.push(operation_detail_line(
                "Create missing budget",
                tr(if *ensure_budget { "Yes" } else { "No" }),
            ));
        }
    }

    if let QueuedOperationStatus::Failed(message) = status {
        lines.push(operation_detail_line("Error", message));
    }

    lines.join("\n")
}

fn operation_detail_line(label: &str, value: impl AsRef<str>) -> String {
    trf(
        "{label}: {value}",
        &[("label", tr(label)), ("value", value.as_ref().to_string())],
    )
}

fn operation_status_text(status: &QueuedOperationStatus) -> String {
    tr(match status {
        QueuedOperationStatus::Pending => "Pending",
        QueuedOperationStatus::Applying => "Applying",
        QueuedOperationStatus::Applied => "Applied",
        QueuedOperationStatus::Failed(_) => "Failed",
    })
}

fn rule_field_label(field: &str) -> &'static str {
    match field {
        "counterparty" => "Counterparty",
        "description" => "Description",
        "tags" => "Tags",
        "account" => "Account",
        "transaction_id" => "Transaction ID",
        _ => "Everything",
    }
}

fn rule_direction_label(direction: &str) -> &'static str {
    match direction {
        "expense" => "Expenses",
        "income" => "Income",
        "transfer" => "Transfers",
        _ => "All transactions",
    }
}

fn clear_done(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
    let removed = ui.operation_queue.clear_applied();
    refresh_operation_queue_ui(state, ui);
    if removed > 0 {
        ui.operation_queue_widgets.dialog.close();
        show_status(ui, "Completed queued operations cleared.");
    }
}

fn apply_one(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>, id: u64) {
    apply_operations(state, ui, vec![id]);
}

fn apply_all(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
    apply_operations(state, ui, ui.operation_queue.actionable_ids());
}

fn apply_operations(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>, ids: Vec<u64>) {
    let ids = ids
        .into_iter()
        .filter(|id| ui.operation_queue.operation_kind(*id).is_some())
        .collect::<Vec<_>>();
    if ids.is_empty() {
        show_status(ui, "No queued operations to apply.");
        return;
    }
    if ui.operation_queue.is_processing() {
        show_status(ui, "The processing queue is already running.");
        return;
    }
    if !try_begin_config_operation(ui, "Another edit or save is already running.") {
        return;
    }

    show_verbose_status(
        ui.as_ref(),
        format!("queue apply started; operations={}", ids.len()),
    );
    ui.operation_queue.set_processing(true);
    ui.status_icon.set_icon_name(Some("view-refresh-symbolic"));
    refresh_operation_queue_ui(state, ui);

    let state_for_apply = Rc::clone(state);
    let ui_for_apply = Rc::clone(ui);
    gtk::glib::MainContext::default().spawn_local(async move {
        let total = ids.len();
        let mut counts = ApplyCounts::default();
        for (index, id) in ids.into_iter().enumerate() {
            let Some(kind) = ui_for_apply.operation_queue.operation_kind(id) else {
                continue;
            };
            if !ui_for_apply.operation_queue.mark_applying(id) {
                continue;
            }
            refresh_operation_queue_ui(&state_for_apply, &ui_for_apply);
            show_status(
                &ui_for_apply,
                &trf(
                    "Applying queued operation {current} of {total}...",
                    &[
                        ("current", (index + 1).to_string()),
                        ("total", total.to_string()),
                    ],
                ),
            );

            let task = gtk::gio::spawn_blocking(move || apply_queued_operation(kind));
            match task.await {
                Ok(Ok(())) => {
                    counts.applied += 1;
                    ui_for_apply.operation_queue.mark_applied(id);
                    show_verbose_status(
                        ui_for_apply.as_ref(),
                        format!("queue operation applied; id={id}"),
                    );
                }
                Ok(Err(error)) => {
                    counts.failed += 1;
                    show_verbose_status(
                        ui_for_apply.as_ref(),
                        format!("queue operation failed; id={id}; error={error:#}"),
                    );
                    ui_for_apply
                        .operation_queue
                        .mark_failed(id, format!("{error:#}"));
                }
                Err(_) => {
                    counts.failed += 1;
                    show_verbose_status(
                        ui_for_apply.as_ref(),
                        format!("queue operation canceled; id={id}"),
                    );
                    ui_for_apply
                        .operation_queue
                        .mark_failed(id, tr("The background task stopped unexpectedly."));
                }
            }
            refresh_operation_queue_ui(&state_for_apply, &ui_for_apply);
        }

        ui_for_apply.operation_queue.set_processing(false);
        ui_for_apply
            .status_icon
            .set_icon_name(Some("dialog-information-symbolic"));
        refresh_operation_queue_ui(&state_for_apply, &ui_for_apply);

        if counts.applied > 0 {
            reload_after_queue_apply(state_for_apply, ui_for_apply, counts).await;
        } else {
            show_apply_summary(&ui_for_apply, counts, None);
            finish_config_operation(&ui_for_apply);
        }
    });
}

fn apply_queued_operation(kind: QueuedOperationKind) -> anyhow::Result<()> {
    match kind {
        QueuedOperationKind::Rule {
            rule,
            ensure_budget,
            ..
        } => {
            super::config_ops::apply_rule_config_change(rule, ensure_budget)?;
            Ok(())
        }
    }
}

async fn reload_after_queue_apply(
    state: Rc<RefCell<AppData>>,
    ui: Rc<UiHandles>,
    counts: ApplyCounts,
) {
    let remember_mode = ui.remember_mode.get();
    let (mode, sources, scope) = {
        let borrowed = state.borrow();
        (
            borrowed.dedupe_mode,
            current_sources_for_reload(&borrowed, remember_mode),
            current_transaction_load_scope(&borrowed, ui.as_ref()),
        )
    };
    let auto_clean_config = ui.preferences.auto_clean_config();
    let smart_insights_enabled =
        smart_pattern_detection_enabled(ui.advanced_features.get(), ui.show_predictions.get());
    show_verbose_status(
        ui.as_ref(),
        format!(
            "queue reload started; scope={scope:?}; remember={remember_mode:?}; sources={}",
            sources.len()
        ),
    );
    show_status(&ui, "Grouping and combining queued rules...");
    begin_background_operation(ui.as_ref());
    let task = gtk::gio::spawn_blocking(move || {
        let combine_summary = combine_queued_rules()?;
        let new_data = data::load_app_data_with_sources(
            mode,
            auto_clean_config,
            scope,
            remember_mode,
            &sources,
            smart_insights_enabled,
        )?
        .0;
        anyhow::Ok((new_data, combine_summary))
    });

    match task.await {
        Ok(Ok((new_data, combine_summary))) => {
            *state.borrow_mut() = new_data;
            show_verbose_status(
                ui.as_ref(),
                format!(
                    "queue reload finished; transactions={}; reports={}",
                    state.borrow().transactions.len(),
                    state.borrow().reports.len(),
                ),
            );
            render_views(&state.borrow(), &ui, &state);
            show_apply_summary(&ui, counts, combine_summary);
        }
        Ok(Err(error)) => show_status(
            &ui,
            &trf(
                "Queued operations applied, but reload failed: {error}",
                &[("error", format!("{error:#}"))],
            ),
        ),
        Err(_) => show_status(
            &ui,
            "Queued operations applied, but reload canceled: the background task stopped unexpectedly.",
        ),
    }

    finish_background_operation(ui.as_ref());
    finish_config_operation(&ui);
    refresh_operation_queue_ui(&state, &ui);
}

fn combine_queued_rules() -> anyhow::Result<Option<RuleCombineSummary>> {
    let rules = data::load_editable_rules()?;
    let (rules, summary) = group_and_combine_queued_rules(&rules);
    if summary.is_some() {
        data::write_editable_rules(&rules)?;
    }
    Ok(summary)
}

fn group_and_combine_queued_rules(
    rules: &[EditableRule],
) -> (Vec<EditableRule>, Option<RuleCombineSummary>) {
    let grouped = data::group_editable_rules_for_combining(rules);
    let report = data::combine_editable_rules(&grouped.rules);
    if report.before_count == report.after_count {
        return (report.rules, None);
    }

    let summary = RuleCombineSummary {
        before_count: report.before_count,
        after_count: report.after_count,
    };
    (report.rules, Some(summary))
}

fn show_apply_summary(
    ui: &Rc<UiHandles>,
    counts: ApplyCounts,
    combine_summary: Option<RuleCombineSummary>,
) {
    let message = apply_summary_message(counts, combine_summary);
    show_status(ui, &message);
}

fn apply_summary_message(
    counts: ApplyCounts,
    combine_summary: Option<RuleCombineSummary>,
) -> String {
    let message = match (counts.applied, counts.failed) {
        (0, 0) => tr("No queued operations were applied."),
        (applied, 0) => trf(
            "Applied {count} queued operation(s).",
            &[("count", applied.to_string())],
        ),
        (0, failed) => trf(
            "{count} queued operation(s) failed.",
            &[("count", failed.to_string())],
        ),
        (applied, failed) => trf(
            "Applied {applied} queued operation(s); {failed} failed.",
            &[
                ("applied", applied.to_string()),
                ("failed", failed.to_string()),
            ],
        ),
    };

    if let Some(summary) = combine_summary {
        trf(
            "{summary} Grouped and combined rules from {before_count} to {after_count}.",
            &[
                ("summary", message),
                ("before_count", summary.before_count.to_string()),
                ("after_count", summary.after_count.to_string()),
            ],
        )
    } else {
        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(search: &str) -> EditableRule {
        EditableRule {
            search: search.to_string(),
            ..EditableRule::new_default()
        }
    }

    #[test]
    fn enqueue_assigns_stable_ids_and_counts_actionable_items() {
        let queue = OperationQueue::new();

        let first = queue.enqueue_rule(rule("alpha"), true, OperationSource::CreateRule);
        let second = queue.enqueue_rule(rule("beta"), true, OperationSource::ChangeBudgetCode);

        assert_eq!(first, EnqueueOperationResult::Queued(1));
        assert_eq!(second, EnqueueOperationResult::Queued(2));
        assert_eq!(queue.actionable_count(), 2);
    }

    #[test]
    fn duplicate_rule_operations_are_not_enqueued_twice() {
        let queue = OperationQueue::new();
        let first = queue.enqueue_rule(rule("alpha"), true, OperationSource::CreateRule);
        let duplicate = queue.enqueue_rule(rule("alpha"), true, OperationSource::CreateRule);
        let other_source =
            queue.enqueue_rule(rule("alpha"), true, OperationSource::ChangeBudgetCode);

        assert_eq!(first, EnqueueOperationResult::Queued(1));
        assert_eq!(duplicate, EnqueueOperationResult::AlreadyQueued(1));
        assert_eq!(other_source, EnqueueOperationResult::Queued(2));
        assert_eq!(queue.operations().len(), 2);
    }

    #[test]
    fn queued_action_sensitivity_follows_duplicate_state() {
        assert!(operation_queue_action_enabled(true, false, true));
        assert!(!operation_queue_action_enabled(true, true, true));
        assert!(!operation_queue_action_enabled(false, false, true));
        assert!(!operation_queue_action_enabled(true, false, false));
    }

    #[test]
    fn pending_remove_deletes_item() {
        let queue = OperationQueue::new();
        let id = queue
            .enqueue_rule(rule("alpha"), true, OperationSource::CreateRule)
            .id();

        assert!(queue.remove(id));
        assert!(queue.operations().is_empty());
    }

    #[test]
    fn applying_item_cannot_be_removed() {
        let queue = OperationQueue::new();
        let id = queue
            .enqueue_rule(rule("alpha"), true, OperationSource::CreateRule)
            .id();

        assert!(queue.mark_applying(id));
        assert!(!queue.remove(id));
        assert_eq!(queue.operations().len(), 1);
    }

    #[test]
    fn apply_buttons_are_disabled_while_loading_or_processing() {
        assert!(operation_apply_button_sensitive(
            &QueuedOperationStatus::Pending,
            false,
            0,
        ));
        assert!(!operation_apply_button_sensitive(
            &QueuedOperationStatus::Pending,
            true,
            0,
        ));
        assert!(!operation_apply_button_sensitive(
            &QueuedOperationStatus::Pending,
            false,
            1,
        ));
        assert!(!operation_apply_button_sensitive(
            &QueuedOperationStatus::Applied,
            false,
            0,
        ));
    }

    #[test]
    fn applied_and_failed_items_can_be_removed() {
        let queue = OperationQueue::new();
        let applied = queue
            .enqueue_rule(rule("alpha"), true, OperationSource::CreateRule)
            .id();
        let failed = queue
            .enqueue_rule(rule("beta"), true, OperationSource::CreateRule)
            .id();

        queue.mark_applied(applied);
        queue.mark_failed(failed, "nope".to_string());

        assert!(queue.remove(applied));
        assert!(queue.remove(failed));
        assert!(queue.operations().is_empty());
    }

    #[test]
    fn apply_summary_includes_rule_combine_result() {
        let message = apply_summary_message(
            ApplyCounts {
                applied: 2,
                failed: 0,
            },
            Some(RuleCombineSummary {
                before_count: 8,
                after_count: 5,
            }),
        );

        assert!(message.contains('2'));
        assert!(message.contains('8'));
        assert!(message.contains('5'));
    }

    #[test]
    fn queued_rule_combine_groups_compatible_rules_first() {
        let software = EditableRule {
            search: "hosting".to_string(),
            category: "Software".to_string(),
            budget_code: "SOFT".to_string(),
            ..EditableRule::new_default()
        };

        let (rules, summary) =
            group_and_combine_queued_rules(&[rule("alpha"), software, rule("beta")]);

        assert_eq!(
            summary,
            Some(RuleCombineSummary {
                before_count: 3,
                after_count: 2,
            })
        );
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].search, "(?:alpha|beta)");
        assert!(rules[0].is_regex);
        assert_eq!(rules[1].search, "hosting");
    }

    #[test]
    fn operation_search_matches_rule_details() {
        let operation = QueuedOperation {
            id: 1,
            kind: QueuedOperationKind::Rule {
                rule: EditableRule {
                    field: "counterparty".to_string(),
                    search: "Coffee Shop".to_string(),
                    category: "Food".to_string(),
                    budget_code: "FOOD".to_string(),
                    ..EditableRule::new_default()
                },
                ensure_budget: true,
                source: OperationSource::CreateRule,
            },
            status: QueuedOperationStatus::Pending,
        };

        assert!(operation_matches_query(&operation, "coffee food"));
        assert!(operation_matches_query(&operation, "coffee shop"));
        assert!(!operation_matches_query(&operation, "transport"));
    }

    #[test]
    fn queue_button_is_suggested_only_for_pending_operations() {
        assert!(!operation_queue_button_is_suggested(0));
        assert!(operation_queue_button_is_suggested(1));
    }

    #[test]
    fn clear_applied_removes_only_successful_items() {
        let queue = OperationQueue::new();
        let applied = queue
            .enqueue_rule(rule("alpha"), true, OperationSource::CreateRule)
            .id();
        let failed = queue
            .enqueue_rule(rule("beta"), true, OperationSource::CreateRule)
            .id();
        let pending = queue
            .enqueue_rule(rule("gamma"), true, OperationSource::CreateRule)
            .id();

        queue.mark_applied(applied);
        queue.mark_failed(failed, "nope".to_string());

        assert_eq!(queue.applied_count(), 1);
        assert_eq!(queue.clear_applied(), 1);
        assert_eq!(queue.applied_count(), 0);
        assert_eq!(
            queue
                .operations()
                .iter()
                .map(|operation| operation.id)
                .collect::<Vec<_>>(),
            vec![failed, pending]
        );
    }

    #[test]
    fn operation_details_icon_tracks_expansion_state() {
        assert_eq!(operation_details_icon_name(false), "pan-end-symbolic");
        assert_eq!(operation_details_icon_name(true), "pan-down-symbolic");
    }

    #[test]
    fn operation_details_keep_full_rule_text() {
        let long_search =
            "a very long shop counterparty with multiple words that should not be truncated";
        let kind = QueuedOperationKind::Rule {
            rule: EditableRule {
                field: "counterparty".to_string(),
                search: long_search.to_string(),
                is_regex: true,
                category: "Groceries and daily shopping".to_string(),
                budget_code: "GROCERY-LONG".to_string(),
                direction: "expense".to_string(),
                amount_min: "-100".to_string(),
                notes: "Generated from repeated matches".to_string(),
                ..EditableRule::new_default()
            },
            ensure_budget: true,
            source: OperationSource::CreateRule,
        };

        let details = operation_details(&kind, &QueuedOperationStatus::Pending);

        assert!(details.contains(long_search));
        assert!(details.contains("Groceries and daily shopping"));
        assert!(details.contains("GROCERY-LONG"));
        assert!(details.contains("-100"));
        assert!(details.contains("Generated from repeated matches"));
    }

    #[test]
    fn operation_details_include_failure_message() {
        let kind = QueuedOperationKind::Rule {
            rule: rule("alpha"),
            ensure_budget: false,
            source: OperationSource::CreateRule,
        };

        let details = operation_details(
            &kind,
            &QueuedOperationStatus::Failed("full failure text".to_string()),
        );

        assert!(details.contains("alpha"));
        assert!(details.contains("full failure text"));
    }

    #[test]
    fn failed_item_can_be_retried() {
        let queue = OperationQueue::new();
        let id = queue
            .enqueue_rule(rule("alpha"), true, OperationSource::CreateRule)
            .id();
        queue.mark_failed(id, "nope".to_string());

        assert_eq!(queue.actionable_ids(), vec![id]);
        assert!(queue.mark_applying(id));
    }
}
