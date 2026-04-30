use std::fmt::Display;

pub struct ValidationConfig {
    pub max_failed_samples: usize,
}

pub enum CheckScope {
    Column,
    Table,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_failed_samples: 100,
        }
    }
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub constraint: String,
    pub passed: bool,
    pub column: Option<String>,
    pub failed_count: Option<usize>,
    pub error: Option<String>,
    pub failed_values: Option<Vec<(usize, String)>>,
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub passed: bool,
    pub total_rules: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub results: Vec<ValidationResult>,
}

impl ValidationReport {
    pub fn merge(self, other: ValidationReport) -> ValidationReport {
        let results: Vec<ValidationResult> =
            self.results.into_iter().chain(other.results).collect();
        let total_rules = results.len();
        let passed_count = results.iter().filter(|r| r.passed).count();
        let failed_count = total_rules - passed_count;
        ValidationReport {
            passed: failed_count == 0,
            total_rules,
            passed_count,
            failed_count,
            results,
        }
    }

    #[cfg(feature = "json")]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

impl Display for ValidationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.passed {
            writeln!(
                f,
                "Validation Report: PASSED ({}/{} rules passed)",
                self.passed_count, self.total_rules
            )?;
        } else {
            writeln!(
                f,
                "Validation Report: FAILED ({}/{} rules passed)",
                self.passed_count, self.total_rules
            )?;
            for result in self.results.iter().filter(|r| !r.passed) {
                writeln!(
                    f,
                    "  FAIL: '{}' — {} — {} values failed: {}",
                    result.column.as_deref().unwrap_or("table"),
                    result.constraint,
                    result.failed_count.unwrap_or(0),
                    result.error.as_deref().unwrap_or("unknown error")
                )?;
                if let Some(values) = &result.failed_values {
                    for (idx, val) in values {
                        writeln!(f, "    row {}: {}", idx, val)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl ValidationResult {
    pub fn passed(constraint: &str, column: Option<&str>, check_scope: CheckScope) -> Self {
        match check_scope {
            CheckScope::Column => ValidationResult {
                column: column.map(String::from),
                constraint: constraint.to_string(),
                passed: true,
                failed_count: Some(0),
                error: None,
                failed_values: None,
            },
            CheckScope::Table => ValidationResult {
                column: column.map(String::from),
                constraint: constraint.to_string(),
                passed: true,
                failed_count: None,
                error: None,
                failed_values: None,
            },
        }
    }

    pub fn failed(
        constraint: &str,
        error: &str,
        column: Option<&str>,
        failed_count: Option<usize>,
        failed_values: Option<Vec<(usize, String)>>,
    ) -> Self {
        ValidationResult {
            column: column.map(String::from),
            constraint: constraint.to_string(),
            passed: false,
            error: Some(error.to_string()),
            failed_values,
            failed_count,
        }
    }
}

impl std::fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.passed {
            write!(
                f,
                "PASS: column '{}' — {}",
                self.column.as_deref().unwrap_or("table"),
                self.constraint
            )
        } else {
            write!(
                f,
                "FAIL: column '{}' — {} — {} values failed: {}",
                self.column.as_deref().unwrap_or("table"),
                self.constraint,
                self.failed_count.unwrap_or(0),
                self.error.as_deref().unwrap_or("unknown error")
            )
        }
    }
}
