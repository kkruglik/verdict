use regex::Regex;

use crate::dataframe::{Column, StringColumn};

pub trait StringOps {
    fn contains(&self, pat: &str) -> Vec<Option<bool>>;
    fn starts_with(&self, pat: &str) -> Vec<Option<bool>>;
    fn ends_with(&self, pat: &str) -> Vec<Option<bool>>;
    fn matches_regex(&self, pat: &str) -> Vec<Option<bool>>;
    fn length(&self) -> Vec<Option<usize>>;
}

impl StringOps for Column {
    fn contains(&self, pat: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.contains(pat),
            _ => vec![None; self.len()],
        }
    }

    fn starts_with(&self, pat: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.starts_with(pat),
            _ => vec![None; self.len()],
        }
    }

    fn ends_with(&self, pat: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.ends_with(pat),
            _ => vec![None; self.len()],
        }
    }

    fn matches_regex(&self, pat: &str) -> Vec<Option<bool>> {
        match self {
            Column::Str(col) => col.matches_regex(pat),
            _ => vec![None; self.len()],
        }
    }

    fn length(&self) -> Vec<Option<usize>> {
        match self {
            Column::Str(col) => col.length(),
            _ => vec![None; self.len()],
        }
    }
}

impl StringOps for StringColumn {
    fn contains(&self, pat: &str) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.contains(pat)))
            .collect()
    }

    fn starts_with(&self, pat: &str) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.starts_with(pat)))
            .collect()
    }

    fn ends_with(&self, pat: &str) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| s.ends_with(pat)))
            .collect()
    }

    fn matches_regex(&self, pat: &str) -> Vec<Option<bool>> {
        let re =
            Regex::new(pat).unwrap_or_else(|e| panic!("invalid regex pattern '{}': {}", pat, e));
        self.0
            .iter()
            .map(|v| v.as_ref().map(|s| re.is_match(s)))
            .collect()
    }

    fn length(&self) -> Vec<Option<usize>> {
        self.0.iter().map(|v| v.as_ref().map(|s| s.len())).collect()
    }
}
