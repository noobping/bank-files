use super::super::*;
use crate::model::{FieldMap, ImportReport};
use std::collections::HashSet;

pub(super) fn generated_field_aliases(
    reports: &[ImportReport],
) -> Result<(Vec<EditableAlias>, usize)> {
    let mut aliases = parse_editable_aliases(default_aliases())?;
    let mut keys = aliases
        .iter()
        .map(alias_key)
        .collect::<HashSet<(String, String)>>();
    let initial_count = aliases.len();

    for report in reports.iter().filter(|report| report.rows_imported > 0) {
        for (canonical, alias) in field_map_aliases(&report.guessed_fields) {
            let alias = alias.trim();
            if alias.is_empty() {
                continue;
            }
            let candidate = EditableAlias {
                canonical: canonical.to_string(),
                alias: alias.to_string(),
            };
            if keys.insert(alias_key(&candidate)) {
                aliases.push(candidate);
            }
        }
    }

    aliases.sort_by(|left, right| {
        left.canonical
            .cmp(&right.canonical)
            .then_with(|| left.alias.cmp(&right.alias))
    });
    let generated_count = aliases.len().saturating_sub(initial_count);
    Ok((aliases, generated_count))
}

fn field_map_aliases(fields: &FieldMap) -> Vec<(&'static str, &str)> {
    [
        ("date", fields.date.as_deref()),
        ("amount", fields.amount.as_deref()),
        ("debit", fields.debit.as_deref()),
        ("credit", fields.credit.as_deref()),
        ("description", fields.description.as_deref()),
        ("counterparty", fields.counterparty.as_deref()),
        ("tags", fields.tags.as_deref()),
        ("account", fields.account.as_deref()),
        ("transaction_id", fields.transaction_id.as_deref()),
        ("currency", fields.currency.as_deref()),
        ("direction", fields.direction.as_deref()),
    ]
    .into_iter()
    .filter_map(|(canonical, alias)| alias.map(|alias| (canonical, alias)))
    .collect()
}

fn alias_key(alias: &EditableAlias) -> (String, String) {
    (normalize_key(&alias.canonical), normalize_key(&alias.alias))
}
