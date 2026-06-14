"""
Generates fixtures/date_checks/data.parquet, schema.json, and expected.json.
"""
import json
import datetime
import pyarrow as pa
import pyarrow.parquet as pq
from pathlib import Path

OUT_DIR = Path(__file__).parent.parent / "fixtures" / "date_checks"
OUT_DIR.mkdir(parents=True, exist_ok=True)

N = 20

def d(year, month=1, day=1):
    return datetime.date(year, month, day)

base      = [d(2021, 1, 1 + i) for i in range(N)]
unique    = base[:]
dup       = [d(2021, 1, 1), d(2021, 1, 1)] + base[2:]
eq_val    = d(2023, 6, 15)
eq        = [eq_val] * N
neq       = [d(2022, 1, 1)] * N
between   = base[:]
out_range = [d(2019, 1, 1)] * N

is_in_set = [d(2021, 3, 1), d(2021, 6, 1), d(2022, 1, 1), d(2023, 6, 1), d(2024, 1, 1)]
is_in     = [is_in_set[i % 5] for i in range(N)]
not_in    = [d(2020, 5, 5)] * N

nulls     = [None] * 3 + base[3:]

def dc(values):
    return pa.array(values, type=pa.date32())

table = pa.table({
    # pass
    "date_not_null":      dc(base),
    "date_unique":        dc(unique),
    "date_after":         dc(base),
    "date_before":        dc(base),
    "date_between_dates": dc(base),
    "date_gt":            dc(base),
    "date_ge":            dc(base),
    "date_lt":            dc(base),
    "date_le":            dc(base),
    "date_eq":            dc(eq),
    "date_between":       dc(between),
    "date_is_in":         dc(is_in),
    # fail
    "date_fail_not_null":      dc(nulls),
    "date_fail_unique":        dc(dup),
    "date_fail_after":         dc(out_range),
    "date_fail_before":        dc([d(2026, 1, 1)] * N),
    "date_fail_between_dates": dc(out_range),
    "date_fail_gt":            dc(out_range),
    "date_fail_ge":            dc(out_range),
    "date_fail_lt":            dc([d(2026, 1, 1)] * N),
    "date_fail_le":            dc([d(2026, 1, 1)] * N),
    "date_fail_eq":            dc(neq),
    "date_fail_between":       dc(out_range),
    "date_fail_is_in":         dc(not_in),
})

pq.write_table(table, OUT_DIR / "data.parquet")
print(f"written {N} rows -> {OUT_DIR / 'data.parquet'}")

def col(name, dtype, constraint, value):
    return {"name": name, "dtype": dtype, "constraints": [{"constraint": constraint, "value": value}]}

schema = {"columns": [
    col("date_not_null",           "date", "not_null",      True),
    col("date_unique",             "date", "unique",        True),
    col("date_after",              "date", "after",         "2020-01-01"),
    col("date_before",             "date", "before",        "2025-01-01"),
    col("date_between_dates",      "date", "between_dates", ["2020-01-01", "2025-01-01"]),
    col("date_gt",                 "date", "gt",            "2020-01-01"),
    col("date_ge",                 "date", "ge",            "2020-01-01"),
    col("date_lt",                 "date", "lt",            "2025-01-01"),
    col("date_le",                 "date", "le",            "2025-01-01"),
    col("date_eq",                 "date", "eq",            "2023-06-15"),
    col("date_between",            "date", "between",       ["2021-01-01", "2024-01-01"]),
    col("date_is_in",              "date", "is_in",         ["2021-03-01", "2021-06-01", "2022-01-01", "2023-06-01", "2024-01-01"]),
    col("date_fail_not_null",      "date", "not_null",      True),
    col("date_fail_unique",        "date", "unique",        True),
    col("date_fail_after",         "date", "after",         "2020-01-01"),
    col("date_fail_before",        "date", "before",        "2025-01-01"),
    col("date_fail_between_dates", "date", "between_dates", ["2020-01-01", "2025-01-01"]),
    col("date_fail_gt",            "date", "gt",            "2020-01-01"),
    col("date_fail_ge",            "date", "ge",            "2020-01-01"),
    col("date_fail_lt",            "date", "lt",            "2025-01-01"),
    col("date_fail_le",            "date", "le",            "2025-01-01"),
    col("date_fail_eq",            "date", "eq",            "2023-06-15"),
    col("date_fail_between",       "date", "between",       ["2021-01-01", "2024-01-01"]),
    col("date_fail_is_in",         "date", "is_in",         ["2021-03-01", "2021-06-01", "2022-01-01", "2023-06-01", "2024-01-01"]),
]}

with open(OUT_DIR / "schema.json", "w") as f:
    json.dump(schema, f, indent=2)
print(f"written -> {OUT_DIR / 'schema.json'}")

expected = [
    {"column": "date_not_null",           "constraint": "not_null",      "passed": True},
    {"column": "date_unique",             "constraint": "unique",        "passed": True},
    {"column": "date_after",              "constraint": "after",         "passed": True},
    {"column": "date_before",             "constraint": "before",        "passed": True},
    {"column": "date_between_dates",      "constraint": "between_dates", "passed": True},
    {"column": "date_gt",                 "constraint": "gt",            "passed": True},
    {"column": "date_ge",                 "constraint": "ge",            "passed": True},
    {"column": "date_lt",                 "constraint": "lt",            "passed": True},
    {"column": "date_le",                 "constraint": "le",            "passed": True},
    {"column": "date_eq",                 "constraint": "eq",            "passed": True},
    {"column": "date_between",            "constraint": "between",       "passed": True},
    {"column": "date_is_in",              "constraint": "is_in",         "passed": True},
    {"column": "date_fail_not_null",      "constraint": "not_null",      "passed": False},
    {"column": "date_fail_unique",        "constraint": "unique",        "passed": False},
    {"column": "date_fail_after",         "constraint": "after",         "passed": False},
    {"column": "date_fail_before",        "constraint": "before",        "passed": False},
    {"column": "date_fail_between_dates", "constraint": "between_dates", "passed": False},
    {"column": "date_fail_gt",            "constraint": "gt",            "passed": False},
    {"column": "date_fail_ge",            "constraint": "ge",            "passed": False},
    {"column": "date_fail_lt",            "constraint": "lt",            "passed": False},
    {"column": "date_fail_le",            "constraint": "le",            "passed": False},
    {"column": "date_fail_eq",            "constraint": "eq",            "passed": False},
    {"column": "date_fail_between",       "constraint": "between",       "passed": False},
    {"column": "date_fail_is_in",         "constraint": "is_in",         "passed": False},
]

with open(OUT_DIR / "expected.json", "w") as f:
    json.dump(expected, f, indent=2)
print(f"written -> {OUT_DIR / 'expected.json'}")
