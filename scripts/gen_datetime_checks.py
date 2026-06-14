"""
Generates fixtures/datetime_checks/data.parquet, schema.json, and expected.json.
"""
import json
import datetime
import pyarrow as pa
import pyarrow.parquet as pq
from pathlib import Path

OUT_DIR = Path(__file__).parent.parent / "fixtures" / "datetime_checks"
OUT_DIR.mkdir(parents=True, exist_ok=True)

N = 20

def dt(year, month=1, day=1, hour=0, minute=0, second=0):
    return datetime.datetime(year, month, day, hour, minute, second)

base         = [dt(2021, 1, 1) + datetime.timedelta(days=i * 30) for i in range(N)]
unique_base  = [dt(2021, 1, 1) + datetime.timedelta(seconds=i) for i in range(N)]
dup          = [dt(2021, 1, 1), dt(2021, 1, 1)] + unique_base[2:]
eq_val       = dt(2023, 6, 15, 12)
eq           = [eq_val] * N
neq          = [dt(2022, 1, 1)] * N
out_range    = [dt(2019, 1, 1)] * N
future       = [dt(2026, 1, 1)] * N

is_in_set = [dt(2021,3,1,8), dt(2021,6,1,12), dt(2022,1,1), dt(2023,6,1,18), dt(2024,1,1,6)]
is_in     = [is_in_set[i % 5] for i in range(N)]
not_in    = [dt(2020, 5, 5)] * N

nulls = [None] * 3 + base[3:]

def ms_col(values): return pa.array(values, type=pa.timestamp("ms"))
def us_col(values): return pa.array(values, type=pa.timestamp("us"))

table = pa.table({
    # pass
    "dt_ms_not_null":      ms_col(base),
    "dt_us_not_null":      us_col(base),
    "dt_ms_unique":        ms_col(unique_base),
    "dt_us_unique":        us_col(unique_base),
    "dt_ms_after":         ms_col(base),
    "dt_us_after":         us_col(base),
    "dt_ms_before":        ms_col(base),
    "dt_us_before":        us_col(base),
    "dt_ms_between_dates": ms_col(base),
    "dt_ms_gt":            ms_col(base),
    "dt_us_ge":            us_col(base),
    "dt_ms_lt":            ms_col(base),
    "dt_us_le":            us_col(base),
    "dt_ms_eq":            ms_col(eq),
    "dt_ms_between":       ms_col(base),
    "dt_us_between":       us_col(base),
    "dt_ms_is_in":         ms_col(is_in),
    # fail
    "dt_ms_fail_not_null":      ms_col(nulls),
    "dt_ms_fail_unique":        ms_col(dup),
    "dt_ms_fail_after":         ms_col(out_range),
    "dt_us_fail_after":         us_col(out_range),
    "dt_ms_fail_before":        ms_col(future),
    "dt_us_fail_before":        us_col(future),
    "dt_ms_fail_between_dates": ms_col(out_range),
    "dt_ms_fail_gt":            ms_col(out_range),
    "dt_us_fail_ge":            us_col(out_range),
    "dt_ms_fail_lt":            ms_col(future),
    "dt_us_fail_le":            us_col(future),
    "dt_ms_fail_eq":            ms_col(neq),
    "dt_ms_fail_between":       ms_col(out_range),
    "dt_ms_fail_is_in":         ms_col(not_in),
})

pq.write_table(table, OUT_DIR / "data.parquet")
print(f"written {N} rows -> {OUT_DIR / 'data.parquet'}")

def col(name, dtype, constraint, value):
    return {"name": name, "dtype": dtype, "constraints": [{"constraint": constraint, "value": value}]}

after_val  = "2020-01-01T00:00:00"
before_val = "2025-01-01T00:00:00"
eq_str     = "2023-06-15T12:00:00"
bw         = ["2020-06-01T00:00:00", "2025-01-01T00:00:00"]
is_in_vals = ["2021-03-01T08:00:00", "2021-06-01T12:00:00", "2022-01-01T00:00:00", "2023-06-01T18:00:00", "2024-01-01T06:00:00"]

schema = {"columns": [
    col("dt_ms_not_null",           "datetime", "not_null",      True),
    col("dt_us_not_null",           "datetime", "not_null",      True),
    col("dt_ms_unique",             "datetime", "unique",        True),
    col("dt_us_unique",             "datetime", "unique",        True),
    col("dt_ms_after",              "datetime", "after",         after_val),
    col("dt_us_after",              "datetime", "after",         after_val),
    col("dt_ms_before",             "datetime", "before",        before_val),
    col("dt_us_before",             "datetime", "before",        before_val),
    col("dt_ms_between_dates",      "datetime", "between_dates", [after_val, before_val]),
    col("dt_ms_gt",                 "datetime", "gt",            after_val),
    col("dt_us_ge",                 "datetime", "ge",            after_val),
    col("dt_ms_lt",                 "datetime", "lt",            before_val),
    col("dt_us_le",                 "datetime", "le",            before_val),
    col("dt_ms_eq",                 "datetime", "eq",            eq_str),
    col("dt_ms_between",            "datetime", "between",       bw),
    col("dt_us_between",            "datetime", "between",       bw),
    col("dt_ms_is_in",              "datetime", "is_in",         is_in_vals),
    col("dt_ms_fail_not_null",      "datetime", "not_null",      True),
    col("dt_ms_fail_unique",        "datetime", "unique",        True),
    col("dt_ms_fail_after",         "datetime", "after",         after_val),
    col("dt_us_fail_after",         "datetime", "after",         after_val),
    col("dt_ms_fail_before",        "datetime", "before",        before_val),
    col("dt_us_fail_before",        "datetime", "before",        before_val),
    col("dt_ms_fail_between_dates", "datetime", "between_dates", [after_val, before_val]),
    col("dt_ms_fail_gt",            "datetime", "gt",            after_val),
    col("dt_us_fail_ge",            "datetime", "ge",            after_val),
    col("dt_ms_fail_lt",            "datetime", "lt",            before_val),
    col("dt_us_fail_le",            "datetime", "le",            before_val),
    col("dt_ms_fail_eq",            "datetime", "eq",            eq_str),
    col("dt_ms_fail_between",       "datetime", "between",       bw),
    col("dt_ms_fail_is_in",         "datetime", "is_in",         is_in_vals),
]}

with open(OUT_DIR / "schema.json", "w") as f:
    json.dump(schema, f, indent=2)
print(f"written -> {OUT_DIR / 'schema.json'}")

expected = [
    {"column": "dt_ms_not_null",           "constraint": "not_null",      "passed": True},
    {"column": "dt_us_not_null",           "constraint": "not_null",      "passed": True},
    {"column": "dt_ms_unique",             "constraint": "unique",        "passed": True},
    {"column": "dt_us_unique",             "constraint": "unique",        "passed": True},
    {"column": "dt_ms_after",              "constraint": "after",         "passed": True},
    {"column": "dt_us_after",              "constraint": "after",         "passed": True},
    {"column": "dt_ms_before",             "constraint": "before",        "passed": True},
    {"column": "dt_us_before",             "constraint": "before",        "passed": True},
    {"column": "dt_ms_between_dates",      "constraint": "between_dates", "passed": True},
    {"column": "dt_ms_gt",                 "constraint": "gt",            "passed": True},
    {"column": "dt_us_ge",                 "constraint": "ge",            "passed": True},
    {"column": "dt_ms_lt",                 "constraint": "lt",            "passed": True},
    {"column": "dt_us_le",                 "constraint": "le",            "passed": True},
    {"column": "dt_ms_eq",                 "constraint": "eq",            "passed": True},
    {"column": "dt_ms_between",            "constraint": "between",       "passed": True},
    {"column": "dt_us_between",            "constraint": "between",       "passed": True},
    {"column": "dt_ms_is_in",              "constraint": "is_in",         "passed": True},
    {"column": "dt_ms_fail_not_null",      "constraint": "not_null",      "passed": False},
    {"column": "dt_ms_fail_unique",        "constraint": "unique",        "passed": False},
    {"column": "dt_ms_fail_after",         "constraint": "after",         "passed": False},
    {"column": "dt_us_fail_after",         "constraint": "after",         "passed": False},
    {"column": "dt_ms_fail_before",        "constraint": "before",        "passed": False},
    {"column": "dt_us_fail_before",        "constraint": "before",        "passed": False},
    {"column": "dt_ms_fail_between_dates", "constraint": "between_dates", "passed": False},
    {"column": "dt_ms_fail_gt",            "constraint": "gt",            "passed": False},
    {"column": "dt_us_fail_ge",            "constraint": "ge",            "passed": False},
    {"column": "dt_ms_fail_lt",            "constraint": "lt",            "passed": False},
    {"column": "dt_us_fail_le",            "constraint": "le",            "passed": False},
    {"column": "dt_ms_fail_eq",            "constraint": "eq",            "passed": False},
    {"column": "dt_ms_fail_between",       "constraint": "between",       "passed": False},
    {"column": "dt_ms_fail_is_in",         "constraint": "is_in",         "passed": False},
]

with open(OUT_DIR / "expected.json", "w") as f:
    json.dump(expected, f, indent=2)
print(f"written -> {OUT_DIR / 'expected.json'}")
