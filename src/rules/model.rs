use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct Rule {
    pub priority: i32,
    pub active: bool,
    pub field: String,
    pub pattern: String,
    pub category: String,
    pub budget_code: String,
    pub direction: String,
    pub amount_min: Option<Decimal>,
    pub amount_max: Option<Decimal>,
    pub notes: String,
}
