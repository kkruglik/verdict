# verdict

A high-performance data validation library written in Rust, with Python bindings via PyO3.

Verdict lets you define typed schemas and validation rules for tabular data, then run them against datasets loaded from CSV files or constructed in memory. All validation logic lives in pure Rust; Python integration is a thin binding layer on top.

---

## Features

- **14 constraint types** — null checks, uniqueness, numeric comparisons, string patterns, set membership, and more
- **Column-to-column comparisons** — validate that one column's values are greater than another's
- **Null-aware operations** — nulls are skipped in comparisons; use `not_null` to enforce presence
- **Failed row samples** — each result includes the row index and value of up to N failed rows
- **Structured report** — `ValidationReport` with pass/fail summary, counts, and per-rule results
- **JSON export** — `--format json` in the CLI or `.to_json()` in Rust (behind `json` feature flag)
- **4 typed column kinds** — `Int`, `Float`, `Str`, `Bool`, each with appropriate operations
- **CSV loading** — feature-gated CSV reader with typed schema enforcement
- **Python bindings** — clean PyO3 API with a fluent `RuleBuilder`
- **CLI binary** — validates CSV against a JSON schema, text or JSON output, exit codes for CI
- **Zero I/O in core** — `verdict-core` has no filesystem dependencies by default

---

## Architecture

```
verdict-core  ←  verdict-py
verdict-core  ←  verdict-cli
```

| Crate | Description |
|---|---|
| `verdict-core` | Pure Rust validation engine. CSV support behind the `csv` feature flag. |
| `verdict-py` | PyO3 bindings that expose verdict to Python. |
| `verdict-cli` | CLI binary for CI/CD pipelines. Reads CSV + JSON schema, outputs results. |

---

## Installation

```bash
cd crates/verdict-py
pip install maturin
maturin develop
```

---

## Quick Start

```python
from verdict_py import Dataset, Schema, DataType, RuleBuilder, py_validate

schema = Schema([
    ("user_id", DataType.integer()),
    ("score",   DataType.float()),
    ("country", DataType.string()),
])

dataset = Dataset.from_csv("data.csv", schema)

rules = [
    *RuleBuilder("user_id").not_null().unique().build(),
    *RuleBuilder("score").between(0.0, 100.0).build(),
    *RuleBuilder("country").is_in(["US", "UK", "DE", "FR", "JP"]).build(),
]

results = py_validate(dataset, rules)

for r in results:
    status = "PASS" if r.is_passed else "FAIL"
    print(f"[{status}] {r.column} / {r.constraint} — {r.failed_count} failures")
```

---

## Constraints

| Constraint | Applies to | Description |
|---|---|---|
| `not_null()` | All | No null values in the column |
| `unique()` | All | All values are distinct |
| `gt(value)` | Int, Float | Every value > threshold |
| `ge(value)` | Int, Float | Every value >= threshold |
| `lt(value)` | Int, Float | Every value < threshold |
| `le(value)` | Int, Float | Every value <= threshold |
| `equal(value)` | Int, Float, Str | Every value == target |
| `between(min, max)` | Int, Float | `min <= value <= max` |
| `matches_regex(pattern)` | Str | Value matches regex pattern |
| `contains(substr)` | Str | Value contains substring |
| `starts_with(prefix)` | Str | Value starts with prefix |
| `ends_with(suffix)` | Str | Value ends with suffix |
| `length_between(min, max)` | Str | String length in `[min, max]` |
| `is_in(values)` | All | Value is a member of the given set |

Pass a column name instead of a literal to compare two columns row-by-row:

```python
Rule("high", Constraint.gt("low"))  # validates high > low for every row
```

---

## CLI

```bash
cargo build --release -p verdict-cli
./target/release/verdict-cli data.csv schema.json
```

Schema format (`schema.json`):

```json
{
  "columns": [
    { "name": "user_id", "dtype": "int", "constraints": [
      { "constraint": "not_null", "value": true },
      { "constraint": "unique",   "value": true }
    ]},
    { "name": "score", "dtype": "float", "constraints": [
      { "constraint": "between", "value": [0, 100] }
    ]},
    { "name": "country", "dtype": "str" }
  ]
}
```

**Flags:**

| Flag | Default | Description |
|---|---|---|
| `--format` | `json` | Output format: `json` or `text` |
| `--max-failed-samples` | `100` | Max failed row samples per rule in the report |

- Columns without `constraints` are still required for schema — they're just not validated
- Exit code `0` — all rules pass; `1` — at least one fails
- JSON output includes `failed_values` with row index and value for each failed row (up to `--max-failed-samples`)

```bash
# text output
verdict-cli data.csv schema.json --format text

# cap failed samples
verdict-cli data.csv schema.json --max-failed-samples 10

# CI usage
verdict-cli data.csv schema.json && echo "data quality OK"
```

---

## CSV Loading

`Dataset.from_csv(path, schema)` parses each column according to its declared type. Empty cells become `None`. Booleans accept `true/True/TRUE/1` and `false/False/FALSE/0`.

---

## Performance

CSV loading benchmarks (Criterion, release build, Apple Silicon):

| Dataset | Load time |
|---|---|
| 10k rows | ~1.5 ms |
| 100k rows | ~15.5 ms |
| 1M rows | ~173 ms |

---

## Development

```bash
cargo build --all-features   # build
cargo test --all-features    # test

cd crates/verdict-py
maturin develop && pytest tests/
```

---

## License

MIT
