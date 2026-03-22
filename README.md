# verdict

A zero-dependency data validation binary for pipelines and CI — no Python, no pandas, no runtime overhead.

[![Rust Build & Test](https://github.com/kkruglik/verdict/actions/workflows/rust-build-test.yml/badge.svg)](https://github.com/kkruglik/verdict/actions/workflows/rust-build-test.yml)
[![verdict-core on crates.io](https://img.shields.io/crates/v/verdict-core?label=verdict-core)](https://crates.io/crates/verdict-core)
[![verdict-cli on crates.io](https://img.shields.io/crates/v/verdict-cli?label=verdict-cli)](https://crates.io/crates/verdict-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Drop a single static binary into any environment, point it at a CSV and a schema file, get a structured pass/fail result. Works anywhere you can run a shell command — CI runners, Docker images, Airflow workers, shell scripts — without installing Python or managing dependencies.

---

## Features

- **17 constraint types** — null checks, uniqueness, numeric comparisons, string patterns, set membership, date ranges, and more
- **Column-to-column comparisons** — validate that one column's values are greater than another's
- **Null-aware operations** — nulls are skipped in comparisons; use `not_null` to enforce presence
- **Failed row samples** — each result includes the row index and value of up to N failed rows
- **Structured report** — `ValidationReport` with pass/fail summary, counts, and per-rule results
- **JSON export** — `--format json` in the CLI or `.to_json()` in Rust (behind `json` feature flag)
- **6 typed column kinds** — `Int`, `Float`, `Str`, `Bool`, `Date`, `DateTime`, each with appropriate operations
- **Date and DateTime support** — stored as epoch integers internally, parsed from strings on CSV load
- **CSV loading** — feature-gated CSV reader with typed schema enforcement
- **Python bindings** — clean PyO3 API with a fluent `RuleBuilder`
- **CLI binary** — validates CSV against a JSON or YAML schema, text or JSON output, exit codes for CI
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

report = py_validate(dataset, rules)

print(f"passed: {report.passed} ({report.passed_count}/{report.total_rules})")

for r in report.results:
    if not r.is_passed:
        print(f"FAIL: {r.column} / {r.constraint} — {r.failed_count} failures")
        for idx, val in (r.failed_values or []):
            print(f"  row {idx}: {val}")
```

---

## Constraints

| Constraint | Applies to | Description |
|---|---|---|
| `not_null()` | All | No null values in the column |
| `unique()` | All | All values are distinct |
| `gt(value)` | Int, Float, Str, col | Every value > threshold |
| `ge(value)` | Int, Float, Str, col | Every value >= threshold |
| `lt(value)` | Int, Float, Str, col | Every value < threshold |
| `le(value)` | Int, Float, Str, col | Every value <= threshold |
| `equal(value)` | Int, Float, Str, col | Every value == target |
| `between(min, max)` | Int, Float, Str, col | `min <= value <= max` |
| `matches_regex(pattern)` | Str | Value matches regex pattern |
| `contains(substr)` | Str | Value contains substring |
| `starts_with(prefix)` | Str | Value starts with prefix |
| `ends_with(suffix)` | Str | Value ends with suffix |
| `length_between(min, max)` | Str | String length in `[min, max]` |
| `is_in(values)` | Int, Float, Str | Value is a member of the given set |
| `after(date)` | Date, DateTime | Every value is strictly after the given date |
| `before(date)` | Date, DateTime | Every value is strictly before the given date |
| `between_dates(min, max)` | Date, DateTime | Every value is within `[min, max]` |

Pass `col("name")` instead of a literal to compare two columns row-by-row:

```python
Rule("high", Constraint.gt(col("low")))   # validates high > low for every row
RuleBuilder("high").gt(col("low")).build()  # same via builder
```

Null values are skipped in all comparisons — they never count as failures. Use `not_null()` separately to enforce presence.

---

## CLI

```bash
cargo build --release -p verdict-cli
./target/release/verdict-cli data.csv schema.json
./target/release/verdict-cli data.csv schema.yaml
```

Schema can be JSON or YAML — detected from the file extension (`.yaml` / `.yml` → YAML, anything else → JSON).

JSON schema (`schema.json`):

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
    { "name": "created_date", "dtype": "date", "format": "%Y-%m-%d", "constraints": [
      { "constraint": "after", "value": "2020-01-01" }
    ]},
    { "name": "country", "dtype": "str" }
  ]
}
```

YAML schema (`schema.yaml`):

```yaml
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
  - name: created_date
    dtype: date
    format: "%Y-%m-%d"
    constraints:
      - constraint: after
        value: "2020-01-01"
  - name: country
    dtype: str
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
verdict-cli data.csv schema.yaml --format text

# cap failed samples
verdict-cli data.csv schema.yaml --max-failed-samples 10

# CI usage
verdict-cli data.csv schema.yaml && echo "data quality OK"
```

### GitHub Actions

Use the official action to validate data in CI without installing Rust or Python:

```yaml
- uses: kkruglik/verdict@main
  with:
    csv: data/output.csv
    schema: data/schema.yaml
```

**Inputs:**

| Input | Required | Default | Description |
|---|---|---|---|
| `csv` | yes | — | Path to the CSV file |
| `schema` | yes | — | Path to the schema file (JSON or YAML) |
| `version` | no | latest | verdict-cli release tag (e.g. `verdict-cli-v0.1.5`) |
| `format` | no | `text` | Output format: `text` or `json` |
| `max-failed-samples` | no | `100` | Max failed row samples per rule |

The action downloads a pre-built binary for the current runner OS — no build step required. Exit code follows the CLI: `0` if all rules pass, `1` if any fail.

---

## CSV Loading

`Dataset.from_csv(path, schema)` parses each column according to its declared type. Empty cells become `None`. Booleans accept `true/True/TRUE/1` and `false/False/FALSE/0`.

Date and DateTime columns require a `format` field in the schema (e.g. `"%Y-%m-%d"`, `"%Y-%m-%dT%H:%M:%S"`). If omitted, standard formats are tried. Internally, dates are stored as `i32` epoch days and datetimes as `i64` epoch microseconds — comparisons run on integers, chrono is only called at parse time and when displaying failed values.

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
