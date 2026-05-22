use super::*;

#[derive(Clone, Copy)]
pub(in crate::app) struct FieldAliasSpec {
    pub(in crate::app) canonical: &'static str,
    pub(in crate::app) label: &'static str,
}

pub(in crate::app) const FIELD_ALIAS_SPECS: [FieldAliasSpec; 11] = [
    FieldAliasSpec {
        canonical: "date",
        label: "Date",
    },
    FieldAliasSpec {
        canonical: "amount",
        label: "Amount",
    },
    FieldAliasSpec {
        canonical: "debit",
        label: "Debit",
    },
    FieldAliasSpec {
        canonical: "credit",
        label: "Credit",
    },
    FieldAliasSpec {
        canonical: "description",
        label: "Description",
    },
    FieldAliasSpec {
        canonical: "counterparty",
        label: "Counterparty",
    },
    FieldAliasSpec {
        canonical: "tags",
        label: "Tags",
    },
    FieldAliasSpec {
        canonical: "account",
        label: "Account",
    },
    FieldAliasSpec {
        canonical: "transaction_id",
        label: "Transaction ID",
    },
    FieldAliasSpec {
        canonical: "currency",
        label: "Currency",
    },
    FieldAliasSpec {
        canonical: "direction",
        label: "Direction",
    },
];

pub(in crate::app) fn field_alias_options() -> Vec<(&'static str, &'static str)> {
    FIELD_ALIAS_SPECS
        .iter()
        .map(|spec| (spec.canonical, spec.label))
        .collect()
}

pub(in crate::app) fn field_alias_combo(active: &str) -> gtk::ComboBoxText {
    ui::combo_from_options(&field_alias_options(), active)
}
