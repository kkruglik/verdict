"""Benchmark: verdict vs pandera on 1,000,000 rows."""
import time
import pandas as pd
import pandera.pandas as pa
from verdict_py import Dataset, Schema, DataType, RuleBuilder, py_validate

CSV = "sample_1m.csv"
RUNS = 5

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
    *RuleBuilder("user_id").not_null().unique().build(),
    *RuleBuilder("score").not_null().between(0.0, 100.0).build(),
    *RuleBuilder("age").not_null().between(18.0, 90.0).build(),
    *RuleBuilder("is_active").not_null().build(),
    *RuleBuilder("country").not_null().is_in(["US", "UK", "DE", "FR", "JP"]).build(),
    *RuleBuilder("age_with_nulls").between(18.0, 90.0).build(),
    *RuleBuilder("score_with_nulls").between(0.0, 100.0).build(),
    *RuleBuilder("country_with_nulls").is_in(["US", "UK", "DE", "FR", "JP"]).build(),
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

# ── helpers ───────────────────────────────────────────────────────────────────

def bench(label, fn, runs=RUNS):
    times = []
    result = None
    for _ in range(runs):
        start = time.perf_counter()
        result = fn()
        times.append((time.perf_counter() - start) * 1000)
    avg = sum(times) / len(times)
    mn = min(times)
    mx = max(times)
    print(f"  {label:<35} avg: {avg:>8.1f} ms   min: {mn:>8.1f} ms   max: {mx:>8.1f} ms")
    return result


def pandera_validate(df):
    try:
        PANDERA_SCHEMA.validate(df, lazy=True)
    except pa.errors.SchemaErrors:
        pass


# ── run ───────────────────────────────────────────────────────────────────────

print(f"\n{'=' * 65}")
print(f"  verdict vs pandera — 1,000,000 rows  ({RUNS} runs each)")
print(f"{'=' * 65}\n")

print("  [load]")
verdict_ds = bench("verdict  (from_csv)", lambda: Dataset.from_csv(CSV, VERDICT_SCHEMA))
df = bench("pandera  (pd.read_csv)", lambda: pd.read_csv(CSV))
print()

print(f"  [validate — {len(VERDICT_RULES)} equivalent rules]")
bench("verdict  (py_validate)", lambda: py_validate(verdict_ds, VERDICT_RULES))
bench("pandera  (schema.validate)", lambda: pandera_validate(df))
print()

print(f"  [load + validate]")
bench("verdict  (end-to-end)", lambda: py_validate(
    Dataset.from_csv(CSV, VERDICT_SCHEMA), VERDICT_RULES
))
bench("pandera  (end-to-end)", lambda: pandera_validate(pd.read_csv(CSV)))
print()
