# verdict-cli

CLI tool for validating CSV files against a JSON schema.

## Installation

Download a pre-built binary from the [GitHub releases page](https://github.com/kkruglik/verdict/releases), or build from source:

```bash
cargo install verdict-cli
```

## Usage

```bash
verdict-cli data.csv schema.json
```

Exit code `0` — all rules pass. Exit code `1` — at least one rule fails.

## Flags

| Flag | Default | Description |
|---|---|---|
| `--format` | `json` | Output format: `json` or `text` |
| `--max-failed-samples` | `100` | Max failed row samples per rule |

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
    { "name": "country", "dtype": "str" }
  ]
}
```

Supported dtypes: `int`, `float`, `str`, `bool`.

Columns without `constraints` are still declared in the schema — they're loaded and type-checked but not validated against any rules.

## Output

**Text:**

```
Validation Report: FAILED (2/3 rules passed)
  FAIL: column 'age' — gt(18) — 5 values failed
    row 3: 15
    row 7: 12
```

**JSON:**

```json
{
  "passed": false,
  "total_rules": 3,
  "passed_count": 2,
  "failed_count": 1,
  "results": [...]
}
```

## CI usage

```bash
verdict-cli data.csv schema.json && echo "data quality OK"
```

## License

MIT
