# verdict-cli — Rust-Based Data Validation for Pipelines and CI

**Validate CSV files against a schema from the command line.** A single static binary — no Python, no pip, no dependency conflicts. Installing pandas and pandera in CI typically takes 40–80 seconds; verdict-cli downloads in 1–3 seconds and runs immediately.

Built on [verdict-core](https://crates.io/crates/verdict-core) — a Rust validation engine with zero I/O overhead.

[![crates.io](https://img.shields.io/crates/v/verdict-cli)](https://crates.io/crates/verdict-cli)

## Installation

```bash
cargo install verdict-cli
```

Or download a pre-built binary for Linux, macOS, or Windows from the [releases page](https://github.com/kkruglik/verdict/releases).

## Usage

```bash
verdict-cli data.csv schema.json
verdict-cli data.csv schema.yaml
verdict-cli data.csv schema.json --format text
verdict-cli data.csv schema.json --max-failed-samples 10
```

Schema format is detected from the file extension: `.yaml` / `.yml` → YAML, anything else → JSON.

Exit code `0` — all rules pass. Exit code `1` — at least one rule fails.

## CI/CD integration

```yaml
# GitHub Actions — no Rust or Python required
- uses: kkruglik/verdict@main
  with:
    csv: data/output.csv
    schema: data/schema.yaml
```

```bash
# pre-commit / shell script
verdict-cli data.csv schema.json || exit 1
```

## Flags

| Flag | Default | Description |
|---|---|---|
| `--format` | `json` | Output format: `json` or `text` |
| `--max-failed-samples` | `100` | Max failed row samples per rule in the report |

## Schema format

**JSON:**

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
    { "name": "country", "dtype": "str", "constraints": [
      { "constraint": "is_in", "value": ["US", "UK", "DE", "FR", "JP"] }
    ]},
    { "name": "created_date", "dtype": "date", "format": "%Y-%m-%d", "constraints": [
      { "constraint": "after", "value": "2020-01-01" }
    ]},
    { "name": "created_at", "dtype": "datetime", "format": "%Y-%m-%dT%H:%M:%S" }
  ]
}
```

**YAML:**

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
  - name: country
    dtype: str
    constraints:
      - constraint: is_in
        value: ["US", "UK", "DE", "FR", "JP"]
  - name: created_date
    dtype: date
    format: "%Y-%m-%d"
    constraints:
      - constraint: after
        value: "2020-01-01"
  - name: created_at
    dtype: datetime
    format: "%Y-%m-%dT%H:%M:%S"
```

Supported dtypes: `int`, `float`, `str`, `bool`, `date`, `datetime`.

Date and DateTime columns accept an optional `format` string. If omitted, standard formats are tried (`%Y-%m-%d` for dates, `%Y-%m-%dT%H:%M:%S` and `%Y-%m-%d %H:%M:%S` for datetimes).

Columns without `constraints` are still required in the schema — they are loaded and type-checked but not validated against any rules.

## Supported constraints

| Constraint | Dtypes | Example value |
|---|---|---|
| `not_null` | all | `true` |
| `unique` | all | `true` |
| `gt` / `ge` / `lt` / `le` | int, float | `18` |
| `equal` | int, float, str | `"active"` |
| `between` | int, float | `[0, 100]` |
| `is_in` | int, float, str | `["US", "UK", "DE"]` |
| `matches_regex` | str | `"^[A-Z]{2}$"` |
| `contains` | str | `"@"` |
| `starts_with` / `ends_with` | str | `"prod_"` |
| `length_between` | str | `[2, 50]` |
| `after` | date, datetime | `"2020-01-01"` |
| `before` | date, datetime | `"2025-01-01"` |
| `between_dates` | date, datetime | `["2020-01-01", "2025-01-01"]` |

## Output

**JSON** (default):

```json
{
  "passed": false,
  "total_rules": 3,
  "passed_count": 2,
  "failed_count": 1,
  "results": [
    {
      "column": "age",
      "constraint": "gt(18)",
      "passed": false,
      "failed_count": 2,
      "failed_values": [[3, "15"], [7, "12"]]
    }
  ]
}
```

**Text** (`--format text`):

```
Validation Report: FAILED (2/3 rules passed)
  FAIL: column 'age' — gt(18) — 2 values failed
    row 3: 15
    row 7: 12
```

## License

MIT
