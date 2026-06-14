"""
Generates fixtures/time_checks/data.parquet, schema.json, and expected.json.
Pass columns: values satisfy the constraint.
Fail columns: values deliberately violate the constraint.
"""
import json
import pyarrow as pa
import pyarrow.parquet as pq
from pathlib import Path

OUT_DIR = Path(__file__).parent.parent / "fixtures" / "time_checks"
OUT_DIR.mkdir(parents=True, exist_ok=True)

N = 20

def ms(h, m=0, s=0):
    return (h * 3600 + m * 60 + s) * 1000

def us(h, m=0, s=0):
    return (h * 3600 + m * 60 + s) * 1_000_000

base_ms = [ms(7 + (i % 12)) for i in range(N)]   # 07:00–18:00
base_us = [us(7 + (i % 12)) for i in range(N)]

unique_ms = [ms(7) + i * 1000 for i in range(N)]
unique_us = [us(7) + i * 1_000_000 for i in range(N)]
dup_ms    = [ms(8), ms(8)] + [ms(9 + i) for i in range(N - 2)]  # first two are duplicates

eq_ms     = [ms(12)] * N
neq_ms    = [ms(11)] * N                           # all wrong for eq(12:00:00)

between_ms = [ms(8 + (i % 11)) for i in range(N)]
between_us = [us(8 + (i % 11)) for i in range(N)]
out_ms     = [ms(3)] * N                           # all outside between(08,18)

is_in_set_ms = [ms(8), ms(10), ms(12), ms(14), ms(16)]
is_in_ms     = [is_in_set_ms[i % 5] for i in range(N)]
not_in_ms    = [ms(9)] * N                         # 09:00 not in the set

nulls_ms  = [None] * 3 + base_ms[3:]              # 3 nulls at start

table = pa.table({
    # pass
    "time_ms_not_null":       pa.array(base_ms,     type=pa.time32("ms")),
    "time_us_not_null":       pa.array(base_us,     type=pa.time64("us")),
    "time_ms_unique":         pa.array(unique_ms,   type=pa.time32("ms")),
    "time_us_unique":         pa.array(unique_us,   type=pa.time64("us")),
    "time_ms_after":          pa.array(base_ms,     type=pa.time32("ms")),
    "time_us_after":          pa.array(base_us,     type=pa.time64("us")),
    "time_ms_before":         pa.array(base_ms,     type=pa.time32("ms")),
    "time_us_before":         pa.array(base_us,     type=pa.time64("us")),
    "time_ms_gt":             pa.array(base_ms,     type=pa.time32("ms")),
    "time_us_ge":             pa.array(base_us,     type=pa.time64("us")),
    "time_ms_lt":             pa.array(base_ms,     type=pa.time32("ms")),
    "time_us_le":             pa.array(base_us,     type=pa.time64("us")),
    "time_ms_eq":             pa.array(eq_ms,       type=pa.time32("ms")),
    "time_ms_between":        pa.array(between_ms,  type=pa.time32("ms")),
    "time_us_between":        pa.array(between_us,  type=pa.time64("us")),
    "time_ms_is_in":          pa.array(is_in_ms,    type=pa.time32("ms")),
    # fail
    "time_ms_fail_not_null":  pa.array(nulls_ms,    type=pa.time32("ms")),
    "time_ms_fail_unique":    pa.array(dup_ms,      type=pa.time32("ms")),
    "time_ms_fail_after":     pa.array([ms(3)] * N, type=pa.time32("ms")),
    "time_us_fail_after":     pa.array([us(3)] * N, type=pa.time64("us")),
    "time_ms_fail_before":    pa.array([ms(23)] * N, type=pa.time32("ms")),
    "time_us_fail_before":    pa.array([us(23)] * N, type=pa.time64("us")),
    "time_ms_fail_gt":        pa.array([ms(3)] * N, type=pa.time32("ms")),
    "time_us_fail_ge":        pa.array([us(3)] * N, type=pa.time64("us")),
    "time_ms_fail_lt":        pa.array([ms(23)] * N, type=pa.time32("ms")),
    "time_us_fail_le":        pa.array([us(23)] * N, type=pa.time64("us")),
    "time_ms_fail_eq":        pa.array(neq_ms,      type=pa.time32("ms")),
    "time_ms_fail_between":   pa.array(out_ms,      type=pa.time32("ms")),
    "time_ms_fail_is_in":     pa.array(not_in_ms,   type=pa.time32("ms")),
})

pq.write_table(table, OUT_DIR / "data.parquet")
print(f"written {N} rows -> {OUT_DIR / 'data.parquet'}")

def col(name, dtype, constraint, value):
    return {"name": name, "dtype": dtype, "constraints": [{"constraint": constraint, "value": value}]}

schema = {"columns": [
    col("time_ms_not_null",      "time", "not_null", True),
    col("time_us_not_null",      "time", "not_null", True),
    col("time_ms_unique",        "time", "unique",   True),
    col("time_us_unique",        "time", "unique",   True),
    col("time_ms_after",         "time", "after",    "06:00:00"),
    col("time_us_after",         "time", "after",    "06:00:00"),
    col("time_ms_before",        "time", "before",   "22:00:00"),
    col("time_us_before",        "time", "before",   "22:00:00"),
    col("time_ms_gt",            "time", "gt",       "06:00:00"),
    col("time_us_ge",            "time", "ge",       "06:00:00"),
    col("time_ms_lt",            "time", "lt",       "22:00:00"),
    col("time_us_le",            "time", "le",       "22:00:00"),
    col("time_ms_eq",            "time", "eq",       "12:00:00"),
    col("time_ms_between",       "time", "between",  ["08:00:00", "18:00:00"]),
    col("time_us_between",       "time", "between",  ["08:00:00", "18:00:00"]),
    col("time_ms_is_in",         "time", "is_in",    ["08:00:00", "10:00:00", "12:00:00", "14:00:00", "16:00:00"]),
    col("time_ms_fail_not_null", "time", "not_null", True),
    col("time_ms_fail_unique",   "time", "unique",   True),
    col("time_ms_fail_after",    "time", "after",    "06:00:00"),
    col("time_us_fail_after",    "time", "after",    "06:00:00"),
    col("time_ms_fail_before",   "time", "before",   "22:00:00"),
    col("time_us_fail_before",   "time", "before",   "22:00:00"),
    col("time_ms_fail_gt",       "time", "gt",       "06:00:00"),
    col("time_us_fail_ge",       "time", "ge",       "06:00:00"),
    col("time_ms_fail_lt",       "time", "lt",       "22:00:00"),
    col("time_us_fail_le",       "time", "le",       "22:00:00"),
    col("time_ms_fail_eq",       "time", "eq",       "12:00:00"),
    col("time_ms_fail_between",  "time", "between",  ["08:00:00", "18:00:00"]),
    col("time_ms_fail_is_in",    "time", "is_in",    ["08:00:00", "10:00:00", "12:00:00", "14:00:00", "16:00:00"]),
]}

with open(OUT_DIR / "schema.json", "w") as f:
    json.dump(schema, f, indent=2)
print(f"written -> {OUT_DIR / 'schema.json'}")

expected = [
    {"column": "time_ms_not_null",      "constraint": "not_null",                    "passed": True},
    {"column": "time_us_not_null",      "constraint": "not_null",                    "passed": True},
    {"column": "time_ms_unique",        "constraint": "unique",                      "passed": True},
    {"column": "time_us_unique",        "constraint": "unique",                      "passed": True},
    {"column": "time_ms_after",         "constraint": "after",                       "passed": True},
    {"column": "time_us_after",         "constraint": "after",                       "passed": True},
    {"column": "time_ms_before",        "constraint": "before",                      "passed": True},
    {"column": "time_us_before",        "constraint": "before",                      "passed": True},
    {"column": "time_ms_gt",            "constraint": "gt",                          "passed": True},
    {"column": "time_us_ge",            "constraint": "ge",                          "passed": True},
    {"column": "time_ms_lt",            "constraint": "lt",                          "passed": True},
    {"column": "time_us_le",            "constraint": "le",                          "passed": True},
    {"column": "time_ms_eq",            "constraint": "eq",                          "passed": True},
    {"column": "time_ms_between",       "constraint": "between",                     "passed": True},
    {"column": "time_us_between",       "constraint": "between",                     "passed": True},
    {"column": "time_ms_is_in",         "constraint": "is_in",                       "passed": True},
    {"column": "time_ms_fail_not_null", "constraint": "not_null",                    "passed": False},
    {"column": "time_ms_fail_unique",   "constraint": "unique",                      "passed": False},
    {"column": "time_ms_fail_after",    "constraint": "after",                       "passed": False},
    {"column": "time_us_fail_after",    "constraint": "after",                       "passed": False},
    {"column": "time_ms_fail_before",   "constraint": "before",                      "passed": False},
    {"column": "time_us_fail_before",   "constraint": "before",                      "passed": False},
    {"column": "time_ms_fail_gt",       "constraint": "gt",                          "passed": False},
    {"column": "time_us_fail_ge",       "constraint": "ge",                          "passed": False},
    {"column": "time_ms_fail_lt",       "constraint": "lt",                          "passed": False},
    {"column": "time_us_fail_le",       "constraint": "le",                          "passed": False},
    {"column": "time_ms_fail_eq",       "constraint": "eq",                          "passed": False},
    {"column": "time_ms_fail_between",  "constraint": "between",                     "passed": False},
    {"column": "time_ms_fail_is_in",    "constraint": "is_in",                       "passed": False},
]

with open(OUT_DIR / "expected.json", "w") as f:
    json.dump(expected, f, indent=2)
print(f"written -> {OUT_DIR / 'expected.json'}")
