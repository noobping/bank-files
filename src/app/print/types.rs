#[derive(Clone)]
pub(in crate::app) struct PrintReport {
    pub(in crate::app) title: String,
    pub(in crate::app) subtitle: String,
    pub(in crate::app) generated: String,
    pub(in crate::app) metrics: Vec<PrintMetric>,
    pub(in crate::app) sections: Vec<PrintSection>,
}

#[derive(Clone)]
pub(in crate::app) struct PrintMetric {
    pub(in crate::app) label: String,
    pub(in crate::app) value: String,
    pub(in crate::app) detail: String,
    pub(in crate::app) tone: PrintTone,
}

#[derive(Clone)]
pub(in crate::app) enum PrintSection {
    Paragraph {
        title: String,
        body: String,
    },
    Table {
        title: String,
        subtitle: String,
        columns: Vec<PrintColumn>,
        rows: Vec<Vec<PrintCell>>,
    },
}

#[derive(Clone)]
pub(in crate::app) struct PrintColumn {
    pub(in crate::app) title: String,
    pub(in crate::app) width: f64,
    pub(in crate::app) align: PrintAlign,
}

#[derive(Clone)]
pub(in crate::app) struct PrintCell {
    pub(in crate::app) text: String,
    pub(in crate::app) tone: PrintTone,
}

#[derive(Clone)]
pub(in crate::app) struct PrintPage {
    pub(in crate::app) elements: Vec<PrintElement>,
}

#[derive(Clone)]
pub(in crate::app) enum PrintElement {
    Metrics(Vec<PrintMetric>),
    SectionTitle {
        title: String,
        subtitle: String,
    },
    Paragraph {
        body: String,
    },
    TableHeader {
        columns: Vec<PrintColumn>,
    },
    TableRow {
        columns: Vec<PrintColumn>,
        cells: Vec<PrintCell>,
        index: usize,
    },
}

#[derive(Clone, Copy)]
pub(in crate::app) enum PrintAlign {
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub(in crate::app) enum PrintTone {
    Normal,
    Muted,
    Positive,
    Negative,
    Warning,
}
