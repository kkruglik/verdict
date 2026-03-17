# verdict-cli — Fast CSV Data Validation for CI/CD Pipelines

**Validate CSV files against a schema from the command line.** Define data quality rules in JSON, run verdict-cli in your pipeline, get structured results and a non-zero exit code when data fails.

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
verdict-cli data.csv schema.json --format text
verdict-cli data.csv schema.json --max-failed-samples 10
```

Exit code `0` — all rules pass. Exit code `1` — at least one rule fails.

## CI/CD integration

```yaml
# GitHub Actions
- name: Validate data
  run: verdict-cli data.csv schema.json
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
    { "name": "created_at", "dtype": "str" }
  ]
}
```

Supported dtypes: `int`, `float`, `str`, `bool`.

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
