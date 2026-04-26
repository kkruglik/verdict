"""Benchmark: verdict vs pandera across multiple dataset sizes."""
import sys
import time
import pandas as pd
import pandera.pandas as pa
from verdict_py import Dataset, Schema, DataType, ColumnRuleBuilder, py_validate_columns

# ── schemas ───────────────────────────────────────────────────────────────────

VERDICT_SCHEMA = Schema([
    ("user_id", DataType.integer()),
    ("score", DataType.float()),
    ("score_with_nulls", DataType.float()),
    ("age", DataType.integer()),
    ("age_with_nulls", DataType.integer()),
    ("is_active", DataType.boolean()),
    ("is_active_with_nulls", DataType.boolean()),
    ("country", DataType.string()),
    ("country_with_nulls", DataType.string()),
])

VERDICT_RULES = [
    *ColumnRuleBuilder("user_id").not_null().unique().build(),
    *ColumnRuleBuilder("score").not_null().between(0.0, 100.0).build(),
    *ColumnRuleBuilder("age").not_null().between(18.0, 90.0).build(),
    *ColumnRuleBuilder("is_active").not_null().build(),
    *ColumnRuleBuilder("country").not_null().is_in(["US", "UK", "DE", "FR", "JP"]).build(),
    *ColumnRuleBuilder("age_with_nulls").between(18.0, 90.0).build(),
    *ColumnRuleBuilder("score_with_nulls").between(0.0, 100.0).build(),
    *ColumnRuleBuilder("country_with_nulls").is_in(["US", "UK", "DE", "FR", "JP"]).build(),
]

PANDERA_SCHEMA = pa.DataFrameSchema(
    columns={
        "user_id": pa.Column(int, nullable=False, unique=True),
        "score": pa.Column(float, pa.Check.between(0, 100), nullable=False),
        "score_with_nulls": pa.Column(float, pa.Check.between(0, 100), nullable=True),
        "age": pa.Column(int, pa.Check.between(18, 90), nullable=False),
        "age_with_nulls": pa.Column(int, pa.Check.between(18, 90), nullable=True),
        "is_active": pa.Column(bool, nullable=False),
        "is_active_with_nulls": pa.Column(bool, nullable=True),
        "country": pa.Column(str, pa.Check.isin(["US", "UK", "DE", "FR", "JP"]), nullable=False),
        "country_with_nulls": pa.Column(str, pa.Check.isin(["US", "UK", "DE", "FR", "JP"]), nullable=True),
    },
)

REGEX_PATTERN = r"^[A-Z]{2}$"

VERDICT_REGEX_RULES = [
    *ColumnRuleBuilder("country").matches_regex(REGEX_PATTERN).build(),
    *ColumnRuleBuilder("country_with_nulls").matches_regex(REGEX_PATTERN).build(),
]

PANDERA_REGEX_SCHEMA = pa.DataFrameSchema(
    columns={
        "country": pa.Column(str, pa.Check.str_matches(REGEX_PATTERN), nullable=False),
        "country_with_nulls": pa.Column(str, pa.Check.str_matches(REGEX_PATTERN), nullable=True),
    },
)


def pandera_regex_validate(df):
    try:
        PANDERA_REGEX_SCHEMA.validate(df, lazy=True)
    except pa.errors.SchemaErrors:
        pass

# ── helpers ───────────────────────────────────────────────────────────────────

def bench(label, fn, runs):
    times = []
    result = None
    for _ in range(runs):
        start = time.perf_counter()
        result = fn()
        times.append((time.perf_counter() - start) * 1000)
    avg = sum(times) / len(times)
    mn = min(times)
    mx = max(times)
    print(f"  {label:<35} avg: {avg:>9.1f} ms   min: {mn:>9.1f} ms   max: {mx:>9.1f} ms")
    return result


def pandera_validate(df):
    try:
        PANDERA_SCHEMA.validate(df, lazy=True)
    except pa.errors.SchemaErrors:
        pass


# ── sizes ─────────────────────────────────────────────────────────────────────

SIZES = [
    ("10K",  "../../fixtures/sample_10k.csv",  10),
    ("100K", "../../fixtures/sample_100k.csv", 10),
    ("1M",   "../../fixtures/sample_1m.csv",    5),
    ("10M",  "../../fixtures/sample_10m.csv",   3),
]

# allow filtering from CLI: python benchmark_pandera.py 1M 10M
filter_sizes = set(sys.argv[1:]) if len(sys.argv) > 1 else None

# ── run ───────────────────────────────────────────────────────────────────────

for label, csv_path, runs in SIZES:
    if filter_sizes and label not in filter_sizes:
        continue

    w = 65
    print(f"\n{'=' * w}")
    print(f"  verdict vs pandera — {label} rows  ({runs} runs each)")
    print(f"{'=' * w}\n")

    print("  [load]")
    verdict_ds = bench("verdict  from_csv", lambda p=csv_path: Dataset.from_csv(p, VERDICT_SCHEMA), runs)
    df = bench("pandera  pd.read_csv", lambda p=csv_path: pd.read_csv(p), runs)
    print()

    print(f"  [validate — {len(VERDICT_RULES)} equivalent rules]")
    bench("verdict  py_validate", lambda: py_validate_columns(verdict_ds, VERDICT_RULES), runs)
    bench("pandera  schema.validate", lambda: pandera_validate(df), runs)
    print()

    print(f"  [load + validate]")
    bench("verdict  end-to-end", lambda p=csv_path: py_validate_columns(
        Dataset.from_csv(p, VERDICT_SCHEMA), VERDICT_RULES
    ), runs)
    bench("pandera  end-to-end", lambda p=csv_path: pandera_validate(pd.read_csv(p)), runs)
    print()

    print(f"  [regex — {len(VERDICT_REGEX_RULES)} rules, pattern: '{REGEX_PATTERN}']")
    bench("verdict  matches_regex", lambda: py_validate_columns(verdict_ds, VERDICT_REGEX_RULES), runs)
    bench("pandera  str_matches",   lambda: pandera_regex_validate(df), runs)
    print()
