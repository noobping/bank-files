use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MonthKey {
    pub year: i32,
    pub month: u32,
}

impl MonthKey {
    pub fn new(year: i32, month: u32) -> Self {
        Self { year, month }
    }

    pub fn previous(self) -> Self {
        if self.month == 1 {
            Self::new(self.year - 1, 12)
        } else {
            Self::new(self.year, self.month - 1)
        }
    }

    pub fn next(self) -> Self {
        if self.month == 12 {
            Self::new(self.year + 1, 1)
        } else {
            Self::new(self.year, self.month + 1)
        }
    }
}

impl fmt::Display for MonthKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}
