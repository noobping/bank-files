use super::super::*;
use super::transaction_builder::normalize_fake_transaction;
use super::FAKE_TRANSACTION_SOURCE;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub(in crate::app) struct FakeTransaction {
    pub(in crate::app) id: u64,
    pub(in crate::app) transaction: Transaction,
}

#[derive(Clone)]
pub(in crate::app) struct FakeTransactionStore {
    next_id: Rc<Cell<u64>>,
    transactions: Rc<RefCell<Vec<FakeTransaction>>>,
}

pub(super) enum FakeTransactionUpdateOutcome {
    Render(&'static str),
    Skip,
}

impl FakeTransactionStore {
    pub(in crate::app) fn new() -> Self {
        Self {
            next_id: Rc::new(Cell::new(1)),
            transactions: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub(in crate::app) fn add(&self, transaction: Transaction) -> u64 {
        let id = self.next_id.get();
        self.next_id.set(id.saturating_add(1));
        self.transactions.borrow_mut().push(FakeTransaction {
            id,
            transaction: normalize_fake_transaction(id, transaction),
        });
        id
    }

    pub(in crate::app) fn update(&self, id: u64, transaction: Transaction) -> bool {
        let mut transactions = self.transactions.borrow_mut();
        let Some(fake) = transactions.iter_mut().find(|fake| fake.id == id) else {
            return false;
        };
        fake.transaction = normalize_fake_transaction(id, transaction);
        true
    }

    pub(in crate::app) fn remove(&self, id: u64) -> bool {
        let mut transactions = self.transactions.borrow_mut();
        let Some(index) = transactions.iter().position(|fake| fake.id == id) else {
            return false;
        };
        transactions.remove(index);
        true
    }

    pub(in crate::app) fn clear(&self) -> usize {
        let mut transactions = self.transactions.borrow_mut();
        let count = transactions.len();
        transactions.clear();
        count
    }

    pub(in crate::app) fn count(&self) -> usize {
        self.transactions.borrow().len()
    }

    pub(in crate::app) fn list(&self) -> Vec<FakeTransaction> {
        self.transactions.borrow().clone()
    }

    pub(super) fn get(&self, id: u64) -> Option<FakeTransaction> {
        self.transactions
            .borrow()
            .iter()
            .find(|fake| fake.id == id)
            .cloned()
    }
}

pub(in crate::app) fn data_with_fake_transactions(
    mut data: AppData,
    fake_transactions: Vec<FakeTransaction>,
) -> AppData {
    if fake_transactions.is_empty() {
        return data;
    }

    data.transactions
        .extend(fake_transactions.into_iter().map(|fake| fake.transaction));
    sort_transactions(&mut data.transactions);
    extend_available_periods(&mut data);
    data
}

pub(in crate::app) fn transaction_is_fake(transaction: &Transaction) -> bool {
    transaction.source_file == FAKE_TRANSACTION_SOURCE
        && transaction.transaction_id.starts_with("FAKE-")
}

pub(in crate::app) fn real_transactions(transactions: &[Transaction]) -> Vec<Transaction> {
    transactions
        .iter()
        .filter(|transaction| !transaction_is_fake(transaction))
        .cloned()
        .collect()
}

fn sort_transactions(transactions: &mut [Transaction]) {
    transactions.sort_by(|a, b| {
        b.date
            .cmp(&a.date)
            .then_with(|| a.description.cmp(&b.description))
    });
}

fn extend_available_periods(data: &mut AppData) {
    let mut months = data
        .available_months
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    months.extend(data.transactions.iter().map(Transaction::month_key));
    data.available_months = months.into_iter().collect();
    data.available_years = data
        .available_months
        .iter()
        .map(|month| month.year)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if data.default_month.is_none() {
        data.default_month = data.available_months.last().copied();
    }
}
