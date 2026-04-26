import time
from verdict_py import (
    Dataset,
    Schema,
    DataType,
    ColumnRuleBuilder,
    py_validate_columns,
)


SCHEMA = Schema(
    [
        ("user_id", DataType.integer()),
        ("score", DataType.float()),
        ("score_with_nulls", DataType.float()),
        ("age", DataType.integer()),
        ("age_with_nulls", DataType.integer()),
        ("is_active", DataType.boolean()),
        ("is_active_with_nulls", DataType.boolean()),
        ("country", DataType.string()),
        ("country_with_nulls", DataType.string()),
    ]
)

RULES = [
    *ColumnRuleBuilder("user_id").not_null().unique().build(),
    *ColumnRuleBuilder("score").not_null().between(0.0, 100.0).build(),
    *ColumnRuleBuilder("age").not_null().between(18.0, 90.0).build(),
    *ColumnRuleBuilder("is_active").not_null().build(),
    *ColumnRuleBuilder("country").not_null().is_in(["US", "UK", "DE", "FR", "JP"]).build(),
    *ColumnRuleBuilder("age_with_nulls").between(18.0, 90.0).build(),
    *ColumnRuleBuilder("score_with_nulls").between(0.0, 100.0).build(),
    *ColumnRuleBuilder("country_with_nulls").is_in(["US", "UK", "DE", "FR", "JP"]).build(),
]

RUNS = 10


def benchmark(label, fn, runs=RUNS):
    times = []
    result = None
    for _ in range(runs):
        start = time.perf_counter()
        result = fn()
        times.append((time.perf_counter() - start) * 1000)
    avg = sum(times) / len(times)
    mn = min(times)
    mx = max(times)
    print(
        f"{label:<30} avg: {avg:.2f} ms   min: {mn:.2f} ms   max: {mx:.2f} ms   ({runs} runs)"
    )
    return result


print(f"{'=' * 60}")
print(f"  Benchmark — sample.csv (100,000 rows, {RUNS} runs each)")
print(f"{'=' * 60}\n")

dataset = benchmark("load from csv", lambda: Dataset.from_csv("../../fixtures/sample.csv", SCHEMA))
print(f"  loaded: {dataset}\n")

report = benchmark(
    f"validate ({len(RULES)} rules)", lambda: py_validate_columns(dataset, RULES)
)

print(f"\n  {report.passed_count} passed / {report.failed_count} failed\n")

print(f"{'=' * 60}")
print(f"  Results")
print(f"{'=' * 60}")
for r in report.results:
    print(f"  {r}")
