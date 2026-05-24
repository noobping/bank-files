use super::page_actions::{page_action_csv, page_action_text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) struct StaticPageSnapshot {
    pub(super) key: String,
    pub(super) title: String,
    pub(super) subtitle: String,
    pub(super) columns: Vec<String>,
    pub(super) rows: Vec<Vec<String>>,
}

impl StaticPageSnapshot {
    pub(in crate::app) fn new(
        key: &str,
        title: &str,
        subtitle: &str,
        columns: &[&str],
        rows: Vec<Vec<String>>,
    ) -> Self {
        Self {
            key: key.to_string(),
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            columns: columns.iter().map(|column| (*column).to_string()).collect(),
            rows,
        }
    }

    #[cfg(test)]
    pub(in crate::app) fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) struct PageActionSnapshot {
    pub(super) key: String,
    pub(super) title: String,
    pub(super) subtitle: String,
    pub(super) columns: Vec<String>,
    pub(super) rows: Vec<Vec<String>>,
    pub(super) text: String,
    pub(super) csv: String,
}

impl PageActionSnapshot {
    pub(in crate::app) fn from_rows(
        key: &str,
        title: &str,
        subtitle: &str,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    ) -> anyhow::Result<Self> {
        let text = page_action_text(title, subtitle, &columns, &rows);
        let csv = page_action_csv(&columns, &rows)?;
        Ok(Self {
            key: key.to_string(),
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            columns,
            rows,
            text,
            csv,
        })
    }

    pub(in crate::app) fn from_csv(
        key: &str,
        title: &str,
        subtitle: &str,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
        csv: String,
    ) -> Self {
        let text = page_action_text(title, subtitle, &columns, &rows);
        Self {
            key: key.to_string(),
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            columns,
            rows,
            text,
            csv,
        }
    }

    pub(super) fn from_static(snapshot: StaticPageSnapshot) -> anyhow::Result<Self> {
        Self::from_rows(
            &snapshot.key,
            &snapshot.title,
            &snapshot.subtitle,
            snapshot.columns,
            snapshot.rows,
        )
    }
}
