use super::super::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::app) enum OperationSource {
    CreateRule,
    ChangeBudgetCode,
    MarkTransfer,
    UndoTransfer,
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
    RuleRemoval {
        rule_match: TransactionRuleMatch,
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
    pub(super) action_registrations: Rc<RefCell<Vec<OperationQueueActionRegistration>>>,
}

#[derive(Clone)]
pub(super) struct OperationQueueActionRegistration {
    pub(super) owner: gtk::Widget,
    pub(super) target: OperationQueueActionTarget,
    pub(super) kind: QueuedOperationKind,
    pub(super) base_enabled: bool,
    pub(super) base_visible: bool,
    pub(super) base_tooltip: Option<String>,
    pub(super) was_rooted: Rc<Cell<bool>>,
}

#[derive(Clone)]
pub(super) enum OperationQueueActionTarget {
    Widget(gtk::Widget),
    MenuAction(gtk::gio::SimpleAction),
}

#[derive(Clone)]
pub(in crate::app) struct OperationQueueWidgets {
    pub(in crate::app) button: gtk::Button,
    pub(in crate::app) badge: gtk::Label,
    pub(in crate::app) summary_row: gtk::Box,
    pub(in crate::app) summary: gtk::Label,
    pub(in crate::app) apply_all_button: gtk::Button,
    pub(in crate::app) clear_done_button: gtk::Button,
    pub(in crate::app) search_entry: gtk::SearchEntry,
    pub(in crate::app) list: gtk::ListBox,
    pub(in crate::app) dialog: adw::Dialog,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub(super) struct ApplyCounts {
    pub(super) applied: usize,
    pub(super) failed: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) struct RuleCombineSummary {
    pub(super) before_count: usize,
    pub(super) after_count: usize,
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
        self.enqueue_kind(QueuedOperationKind::Rule {
            rule,
            ensure_budget,
            source,
        })
    }

    pub(in crate::app) fn enqueue_rule_removal(
        &self,
        rule_match: TransactionRuleMatch,
        source: OperationSource,
    ) -> EnqueueOperationResult {
        self.enqueue_kind(QueuedOperationKind::RuleRemoval { rule_match, source })
    }

    fn enqueue_kind(&self, kind: QueuedOperationKind) -> EnqueueOperationResult {
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

    pub(super) fn contains_kind(&self, kind: &QueuedOperationKind) -> bool {
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

    pub(super) fn is_processing(&self) -> bool {
        self.processing.get()
    }

    pub(super) fn set_processing(&self, processing: bool) {
        self.processing.set(processing);
    }

    pub(super) fn operation_kind(&self, id: u64) -> Option<QueuedOperationKind> {
        self.operations
            .borrow()
            .iter()
            .find(|operation| operation.id == id && operation.status.is_actionable())
            .map(|operation| operation.kind.clone())
    }

    pub(super) fn mark_applying(&self, id: u64) -> bool {
        self.set_status_if(id, QueuedOperationStatus::Applying, |status| {
            status.is_actionable()
        })
    }

    pub(super) fn mark_applied(&self, id: u64) {
        self.set_status(id, QueuedOperationStatus::Applied);
    }

    pub(super) fn mark_failed(&self, id: u64, message: String) {
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
    pub(super) fn id(self) -> u64 {
        match self {
            Self::Queued(id) | Self::AlreadyQueued(id) => id,
        }
    }
}

impl QueuedOperationStatus {
    pub(super) fn is_actionable(&self) -> bool {
        matches!(self, Self::Pending | Self::Failed(_))
    }

    pub(super) fn can_remove(&self) -> bool {
        !matches!(self, Self::Applying)
    }
}
