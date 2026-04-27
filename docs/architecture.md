# Architecture

## Crate Structure

```
verdict-core  ←  verdict-cli
verdict-core  ←  verdict-py
```

| Crate | Description |
|---|---|
| `verdict-core` | Pure validation logic. No I/O by default. CSV loading behind the `csv` feature flag. |
| `verdict-cli` | Static binary. Reads CSV + JSON/YAML schema, runs validation, outputs results. |
| `verdict-py` | PyO3 bindings exposing verdict to Python. |

No dependency cycles. `verdict-core` knows nothing about CLI or Python.

---

## verdict-core

### Type Hierarchy

```
DataFrame
├── columns: Vec<Column>
├── schema: Schema
│   └── fields: Vec<Field>
│       ├── name: String
│       └── dtype: DataType (Int, Float, Str, Bool, Date, DateTime)
└── accessors: get_column_by_name, get_column_by_index, get_column_index, shape

Column (enum) — delegates to typed columns
├── common: len, is_empty, null_count, not_null_count, is_null
└── variants:
    ├── IntColumn (Vec<Option<i64>>)
    │   ├── NumericOps         → sum, min, max, mean, std, median
    │   ├── ComparableOps<i64> → gt, ge, lt, le, equal, between
    │   └── ComparableOps<f64> → gt, ge, lt, le, equal, between
    ├── FloatColumn (Vec<Option<f64>>)
    │   ├── NumericOps         → sum, min, max, mean, std, median
    │   └── ComparableOps<f64> → gt, ge, lt, le, equal, between
    ├── StrColumn (Vec<Option<String>>)
    │   ├── ComparableOps<&str> → gt, ge, lt, le, equal, between
    │   └── StringOps           → contains, starts_with, ends_with, matches_regex, length
    ├── BoolColumn (Vec<Option<bool>>)
    │   └── (common ops only)
    ├── DateColumn (Vec<Option<i32>>)      — epoch days
    │   └── ComparableOps<i32>
    └── DateTimeColumn (Vec<Option<i64>>)  — epoch microseconds
        └── ComparableOps<i64>
```

### Rules

```
ColumnRule { column: String, constraint: ColumnConstraint }
TableRule  { constraint: TableConstraint }
```

**ColumnConstraint** (17 variants) — operate on a single column's values: `NotNull`, `Unique`, `GreaterThan`, `GreaterThanOrEqual`, `LessThan`, `LessThanOrEqual`, `Equal`, `Between`, `InSet`, `MatchesRegex`, `Contains`, `StartsWith`, `EndsWith`, `LengthBetween`, `After`, `Before`, `BetweenDates`.

**TableConstraint** (12 variants) — operate on the dataset shape: `RowsCountBetween`, `RowsCountGreaterOrEqual`, `RowCountGreaterThan`, `RowsCountLessOrEqual`, `RowCountLessThan`, `ColumnsCountBetween`, `ColumnsCountGreaterOrEqual`, `ColumnsCountGreaterThan`, `ColumnsCountLessOrEqual`, `ColumnsCountLessThan`, `ColumnsExist`, `ShapeEquals`.

### Validation

```rust
validate_columns(data: &DataFrame, rules: &[ColumnRule], config: ValidationConfig) -> ValidationReport
validate_table(data: &DataFrame, rules: &[TableRule], config: ValidationConfig) -> ValidationReport

report.merge(other: ValidationReport) -> ValidationReport
```

**ValidationResult** fields: `constraint`, `passed`, `column: Option<String>`, `failed_count: Option<usize>`, `failed_values`, `error`. Column and table results share the same type — `column` and `failed_count` are `None` for table-level checks.

### Feature Flags

- `csv` — enables `DatasetCsvExt` trait (`DataFrame::from_csv(path, schema)`) and `CsvLoadingError`
- `json` — enables `ValidationReport::to_json()`

---

## verdict-cli

Reads a CSV and a JSON or YAML schema file, runs `validate_table` + `validate_columns`, and prints results.

**Schema format:** `{ columns: [...], table: [...] }`. The `table` block is optional.

**Exit codes:** `0` — all rules pass, `1` — at least one fails.

**Output formats:** `text` (default) and `json` (via `--format json`).

---

## verdict-py

PyO3 bindings. Exposes `Dataset`, `Column`, `Schema`, `DataType`, `ColumnConstraint`, `ColumnRule`, `ColumnRuleBuilder`, `TableConstraint`, `TableRule`, `ValidationResult`, `ValidationReport`, `py_validate_columns`, `py_validate_table`.

`Option<T>` fields map to Python `T | None` automatically.
