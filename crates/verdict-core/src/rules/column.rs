use crate::dataframe::ValuesSet;
use crate::rules::Operand;

pub fn col(name: &str) -> Operand {
    Operand::Column(name.to_string())
}

#[derive(Debug, Clone)]
pub enum ColumnConstraint {
    // Null checks
    NotNull,
    Unique,

    // Numeric comparisons
    GreaterThan(Operand),
    GreaterThanOrEqual(Operand),
    LessThan(Operand),
    LessThanOrEqual(Operand),
    Equal(Operand),
    Between { min: Operand, max: Operand },

    // String checks
    InSet(ValuesSet),
    MatchesRegex(String),
    Contains(String),
    StartsWith(String),
    EndsWith(String),
    LengthBetween { min: usize, max: usize },

    // Datetime checks
    After(String),
    Before(String),
    BetweenDates { min: String, max: String },
}

impl std::fmt::Display for ColumnConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnConstraint::NotNull => write!(f, "not_null"),
            ColumnConstraint::Unique => write!(f, "unique"),
            ColumnConstraint::GreaterThan(op) => write!(f, "gt({})", op),
            ColumnConstraint::GreaterThanOrEqual(op) => write!(f, "ge({})", op),
            ColumnConstraint::LessThan(op) => write!(f, "lt({})", op),
            ColumnConstraint::LessThanOrEqual(op) => write!(f, "le({})", op),
            ColumnConstraint::Equal(op) => write!(f, "eq({})", op),
            ColumnConstraint::Between { min, max } => write!(f, "between({}, {})", min, max),
            ColumnConstraint::InSet(_) => write!(f, "in_set"),
            ColumnConstraint::MatchesRegex(p) => write!(f, "matches_regex({})", p),
            ColumnConstraint::Contains(p) => write!(f, "contains({})", p),
            ColumnConstraint::StartsWith(p) => write!(f, "starts_with({})", p),
            ColumnConstraint::EndsWith(p) => write!(f, "ends_with({})", p),
            ColumnConstraint::LengthBetween { min, max } => {
                write!(f, "length_between({}, {})", min, max)
            }
            ColumnConstraint::After(op) => write!(f, "after({})", op),
            ColumnConstraint::Before(op) => write!(f, "before({})", op),
            ColumnConstraint::BetweenDates { min, max } => {
                write!(f, "between_dates({}, {})", min, max)
            }
        }
    }
}

#[derive(Clone)]
pub struct ColumnRule {
    pub column: String,
    pub constraint: ColumnConstraint,
}

impl ColumnRule {
    pub fn new(column: impl Into<String>, constraint: ColumnConstraint) -> ColumnRule {
        ColumnRule { column: column.into(), constraint }
    }
}

#[derive(Default)]
pub struct ColumnRuleBuilder {
    pub column: String,
    pub constraint: Vec<ColumnConstraint>,
}

impl ColumnRuleBuilder {
    pub fn not_null(mut self) -> Self {
        self.constraint.push(ColumnConstraint::NotNull);
        self
    }

    pub fn unique(mut self) -> Self {
        self.constraint.push(ColumnConstraint::Unique);
        self
    }

    pub fn gt(mut self, compare: impl Into<Operand>) -> Self {
        self.constraint
            .push(ColumnConstraint::GreaterThan(compare.into()));
        self
    }

    pub fn ge(mut self, compare: impl Into<Operand>) -> Self {
        self.constraint
            .push(ColumnConstraint::GreaterThanOrEqual(compare.into()));
        self
    }

    pub fn lt(mut self, compare: impl Into<Operand>) -> Self {
        self.constraint
            .push(ColumnConstraint::LessThan(compare.into()));
        self
    }

    pub fn le(mut self, compare: impl Into<Operand>) -> Self {
        self.constraint
            .push(ColumnConstraint::LessThanOrEqual(compare.into()));
        self
    }

    pub fn equal(mut self, compare: impl Into<Operand>) -> Self {
        self.constraint
            .push(ColumnConstraint::Equal(compare.into()));
        self
    }

    pub fn between(mut self, min: impl Into<Operand>, max: impl Into<Operand>) -> Self {
        self.constraint.push(ColumnConstraint::Between {
            min: min.into(),
            max: max.into(),
        });
        self
    }

    pub fn in_set(mut self, values: ValuesSet) -> Self {
        self.constraint.push(ColumnConstraint::InSet(values));
        self
    }

    pub fn matches_regex(mut self, pattern: &str) -> Self {
        self.constraint
            .push(ColumnConstraint::MatchesRegex(pattern.to_string()));
        self
    }

    pub fn contains(mut self, pattern: &str) -> Self {
        self.constraint
            .push(ColumnConstraint::Contains(pattern.to_string()));
        self
    }

    pub fn starts_with(mut self, pattern: &str) -> Self {
        self.constraint
            .push(ColumnConstraint::StartsWith(pattern.to_string()));
        self
    }

    pub fn ends_with(mut self, pattern: &str) -> Self {
        self.constraint
            .push(ColumnConstraint::EndsWith(pattern.to_string()));
        self
    }

    pub fn length_between(mut self, min: usize, max: usize) -> Self {
        self.constraint
            .push(ColumnConstraint::LengthBetween { min, max });
        self
    }

    pub fn build(self) -> Vec<ColumnRule> {
        self.constraint
            .into_iter()
            .map(|c| ColumnRule {
                column: self.column.clone(),
                constraint: c,
            })
            .collect()
    }
}

pub fn rule(col_name: &str) -> ColumnRuleBuilder {
    ColumnRuleBuilder {
        column: col_name.to_string(),
        constraint: vec![],
    }
}
