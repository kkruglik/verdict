# Verdict Development Plan

## Phase 1: Dataset Foundation

### 1.1 Dataset Accessors
- [ ] `get_column(name: &str) -> Option<&Column>`
- [ ] `get_column_index(name: &str) -> Option<usize>`
- [ ] Typed getters: `get_int_column`, `get_str_column`, etc.

### 1.2 Column Utilities
- [ ] `null_count()` / `not_null_count()`
- [ ] `is_empty()`
- [ ] `unique_values()` for each type
- [ ] `min()` / `max()` for numeric columns

---

## Phase 2: Validation

### 2.1 Validation Results
- [ ] Define `ValidationResult` struct
- [ ] Track: passed/failed, which rows failed, failure reasons
- [ ] Implement `Display` for human-readable output

### 2.2 Expectation System
- [ ] Define `Expectation` trait/enum
- [ ] Column-level: `not_null`, `unique`, `in_set`, `in_range`, `matches_regex`
- [ ] Row-level: `column_pair_unique`, `column_a_gt_b`

---

## Phase 3: Architecture Cleanup

### 3.1 Move CSV to verdict-csv
- [ ] `load_csv(path, schema) -> Dataset`
- [ ] Remove `csv` dependency from core
- [ ] Split errors

---

## Phase 4: Python Bindings

### 4.1 Basic Bindings
- [ ] Expose `Dataset`, `Schema`, `Field`, `DataType`
- [ ] `load_csv`, `shape()`

### 4.2 Validation Bindings
- [ ] Expose expectations + results
