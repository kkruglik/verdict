# verdict

A static binary that validates CSV data against a schema — no Python, no pandas, no runtime overhead.

[![Rust Build & Test](https://github.com/kkruglik/verdict/actions/workflows/rust-build-test.yml/badge.svg)](https://github.com/kkruglik/verdict/actions/workflows/rust-build-test.yml)
[![verdict-core on crates.io](https://img.shields.io/crates/v/verdict-core?label=verdict-core)](https://crates.io/crates/verdict-core)
[![verdict-cli on crates.io](https://img.shields.io/crates/v/verdict-cli?label=verdict-cli)](https://crates.io/crates/verdict-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Drop a single binary into any environment, point it at a CSV and a schema file, get a structured pass/fail result. Works anywhere you can run a shell command — CI runners, Docker images, Airflow workers, shell scripts — without installing Python or managing dependencies.

---

## Quick Start

```bash
verdict-cli data.csv schema.yaml
```

```yaml
# schema.yaml
columns:
  - name: user_id
    dtype: int
    constraints:
      - constraint: not_null
        value: true
      - constraint: unique
        value: true
  - name: score
    dtype: float
    constraints:
      - constraint: between
        value: [0, 100]
  - name: country
    dtype: str
    constraints:
      - constraint: is_in
        value: ["US", "UK", "DE", "FR", "JP"]
table:
  - constraint: rows_count_between
    value: [1000, 2000000]
  - constraint: columns_exist
    value: ["user_id", "score", "country"]
```

Exit code `0` — all rules pass. `1` — at least one fails.

---

## Schema Format

Schema can be JSON or YAML — detected from the file extension. Both formats use the same structure.

**Column fields:**

| Field | Required | Description |
|---|---|---|
| `name` | yes | Column name as it appears in the CSV header |
| `dtype` | yes | `int`, `float`, `str`, `bool`, `date`, `datetime` |
| `format` | date/datetime only | Chrono format string, e.g. `"%Y-%m-%d"` |
| `constraints` | no | List of `{ constraint, value }` objects |

Columns without `constraints` are still loaded and type-checked — they just aren't validated beyond that.

---

## Column Constraints

| Constraint | Value | Applies to |
|---|---|---|
| `not_null` | `true` | All |
| `unique` | `true` | All |
| `gt` | number or `{"col": "name"}` | Int, Float, Str |
| `ge` | number or `{"col": "name"}` | Int, Float, Str |
| `lt` | number or `{"col": "name"}` | Int, Float, Str |
| `le` | number or `{"col": "name"}` | Int, Float, Str |
| `equal` | number or `{"col": "name"}` | Int, Float, Str |
| `between` | `[min, max]` or column refs | Int, Float, Str |
| `is_in` | `["a", "b"]` or `[1, 2]` | Int, Float, Str |
| `matches_regex` | pattern string | Str |
| `contains` | substring | Str |
| `starts_with` | prefix | Str |
| `ends_with` | suffix | Str |
| `length_between` | `[min, max]` | Str |
| `after` | date string | Date, DateTime |
| `before` | date string | Date, DateTime |
| `between_dates` | `["date", "date"]` | Date, DateTime |

**Column-to-column comparisons:** use `{"col": "name"}` as the value to compare two columns row-by-row:

```yaml
- constraint: gt
  value: {"col": "low_price"}
```

Null values are skipped in all comparisons — they never count as failures. Use `not_null` separately to enforce presence.

---

## Table Constraints

Table constraints validate the dataset itself — shape, row count, column existence — before any per-column checks run.

| Constraint | Value | Description |
|---|---|---|
| `rows_count_between` | `[min, max]` | Row count is within range |
| `rows_count_ge` | number | Row count >= threshold |
| `rows_count_gt` | number | Row count > threshold |
| `rows_count_le` | number | Row count <= threshold |
| `rows_count_lt` | number | Row count < threshold |
| `columns_count_between` | `[min, max]` | Column count within range |
| `columns_count_ge` | number | Column count >= threshold |
| `columns_count_gt` | number | Column count > threshold |
| `columns_count_le` | number | Column count <= threshold |
| `columns_count_lt` | number | Column count < threshold |
| `columns_exist` | `["col_a", "col_b"]` | Named columns are present |
| `shape_equals` | `[rows, columns]` | Exact shape match |

---

## Flags

| Flag | Default | Description |
|---|---|---|
| `--format` | `text` | Output format: `text` or `json` |
| `--max-failed-samples` | `100` | Max failed row samples per rule |

```bash
# JSON output
verdict-cli data.csv schema.yaml --format json

# Cap failed samples
verdict-cli data.csv schema.yaml --max-failed-samples 10

# CI usage
verdict-cli data.csv schema.yaml && echo "OK"
```

---

## GitHub Actions

Use the official action to validate data in CI without installing Rust or Python:

```yaml
- uses: kkruglik/verdict@main
  with:
    csv: data/output.csv
    schema: data/schema.yaml
```

| Input | Required | Default | Description |
|---|---|---|---|
| `csv` | yes | — | Path to the CSV file |
| `schema` | yes | — | Path to the schema file (JSON or YAML) |
| `version` | no | latest | verdict-cli release tag |
| `format` | no | `text` | Output format: `text` or `json` |
| `max-failed-samples` | no | `100` | Max failed row samples per rule |

The action downloads a pre-built binary for the current runner OS — no build step required.

---

## Architecture

```
verdict-core  ←  verdict-cli
verdict-core  ←  verdict-py
```

| Crate | Description |
|---|---|
| `verdict-core` | Pure Rust validation engine. CSV loading behind the `csv` feature flag. |
| `verdict-cli` | Static binary for CI/CD pipelines. Reads CSV + schema, outputs results. |
| `verdict-py` | PyO3 Python bindings. |

---

## Python API

```bash
cd crates/verdict-py && pip install maturin && maturin develop
```

```python
from verdict_py import Dataset, Schema, DataType, ColumnRuleBuilder, py_validate_columns
from verdict_py import TableConstraint, TableRule, py_validate_table

schema = Schema([("user_id", DataType.integer()), ("score", DataType.float())])
dataset = Dataset.from_csv("data.csv", schema)

col_rules = [
    *ColumnRuleBuilder("user_id").not_null().unique().build(),
    *ColumnRuleBuilder("score").between(0.0, 100.0).build(),
]
table_rules = [TableRule(TableConstraint.rows_count_between(1000, 2000000))]

report = py_validate_table(dataset, table_rules).merge(py_validate_columns(dataset, col_rules))
print(f"passed: {report.passed} ({report.passed_count}/{report.total_rules})")
```

All column and table constraints from the CLI schema are available as Python methods. `ValidationResult.column` and `.failed_count` are `None` for table-level results.

---

## Development

```bash
cargo build --all-features
cargo test --all-features
cargo test -p verdict-cli
```

---

## License

MIT
