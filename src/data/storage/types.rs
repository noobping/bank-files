#[derive(Debug, Clone, Copy, Default)]
pub struct CsvCopyResult {
    pub transaction_csvs: usize,
    pub config_csvs: usize,
    pub skipped: usize,
}

impl CsvCopyResult {
    pub fn imported(&self) -> usize {
        self.transaction_csvs + self.config_csvs
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EditableRule {
    pub priority: i32,
    pub active: bool,
    pub field: String,
    pub search: String,
    pub is_regex: bool,
    pub category: String,
    pub budget_code: String,
    pub direction: String,
    pub amount_min: String,
    pub amount_max: String,
    pub notes: String,
}

impl EditableRule {
    pub fn new_default() -> Self {
        Self {
            priority: 120,
            active: true,
            field: "any".to_string(),
            search: String::new(),
            is_regex: false,
            category: "New category".to_string(),
            budget_code: "OTHER".to_string(),
            direction: "expense".to_string(),
            amount_min: String::new(),
            amount_max: String::new(),
            notes: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EditableBudget {
    pub code: String,
    pub category: String,
    pub monthly_budget: String,
    pub yearly_budget: String,
    pub direction: String,
    pub income_basis: String,
    pub notes: String,
}

impl EditableBudget {
    pub fn new_default() -> Self {
        Self {
            code: "NEW".to_string(),
            category: "New category".to_string(),
            monthly_budget: "0".to_string(),
            yearly_budget: String::new(),
            direction: "expense".to_string(),
            income_basis: "real".to_string(),
            notes: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EditableAlias {
    pub canonical: String,
    pub alias: String,
}

impl EditableAlias {
    pub fn new_default() -> Self {
        Self {
            canonical: "description".to_string(),
            alias: String::new(),
        }
    }
}
