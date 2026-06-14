"""
Generates small parquet fixture files for verdict-core unit tests.
Output: crates/verdict-core/tests/fixtures/parquet/
Run: uv run --project crates/verdict-py python3 scripts/gen_parquet_test_fixtures.py
"""
import os
import pyarrow as pa
import pyarrow.parquet as pq

OUT = "crates/verdict-core/tests/fixtures/parquet"
os.makedirs(OUT, exist_ok=True)

ids = list(range(1, 11))
scores = [round(i * 1.1, 1) for i in range(1, 11)]
labels = [f"row_{i}" for i in range(1, 11)]
active = [i % 2 == 0 for i in range(10)]

# date32: days since epoch; 2024-01-01 = 19723
dates = list(range(19723, 19733))

# timestamp("ms"): 2024-01-01T12:00:00 UTC = 1704110400000 ms
ts_ms_base = 1704110400000
ts_ms_vals = [ts_ms_base + i * 86_400_000 for i in range(10)]

# timestamp("us"): same instants in microseconds — verdict-core normalises ms→us on load
ts_us_vals = [v * 1000 for v in ts_ms_vals]

# time32("ms"): 08:00:00 = 28_800_000 ms, incrementing 1 h per row
time_ms_base = 8 * 3600 * 1000
time_ms_vals = [time_ms_base + i * 3_600_000 for i in range(10)]

# time64("us"): same times in microseconds — verdict-core normalises ms→us on load
time_us_vals = [v * 1000 for v in time_ms_vals]


def make_table(ids_, scores_, labels_, active_, dates_, ts_ms_, ts_us_, time_ms_, time_us_):
    return pa.table({
        "id":       pa.array(ids_,     type=pa.int64()),
        "score":    pa.array(scores_,  type=pa.float64()),
        "label":    pa.array(labels_,  type=pa.utf8()),
        "active":   pa.array(active_,  type=pa.bool_()),
        "date_col": pa.array(dates_,   type=pa.date32()),
        "ts_ms":    pa.array(ts_ms_,   type=pa.timestamp("ms")),
        "ts_us":    pa.array(ts_us_,   type=pa.timestamp("us")),
        "time_ms":  pa.array(time_ms_, type=pa.time32("ms")),
        "time_us":  pa.array(time_us_, type=pa.time64("us")),
    })


def nulls_at(lst, indices=(3, 7)):
    return [None if i in indices else v for i, v in enumerate(lst)]


# all_types.parquet — 10 rows, no nulls, one column per supported dtype
all_types = make_table(ids, scores, labels, active, dates,
                       ts_ms_vals, ts_us_vals, time_ms_vals, time_us_vals)
pq.write_table(all_types, f"{OUT}/all_types.parquet")
print(f"Written {OUT}/all_types.parquet  ({len(all_types)} rows, {len(all_types.schema)} cols)")

# with_nulls.parquet — same schema, 2 nulls per column at rows 3 and 7
null_table = make_table(
    nulls_at(ids),    nulls_at(scores), nulls_at(labels), nulls_at(active),
    nulls_at(dates),  nulls_at(ts_ms_vals), nulls_at(ts_us_vals),
    nulls_at(time_ms_vals), nulls_at(time_us_vals),
)
pq.write_table(null_table, f"{OUT}/with_nulls.parquet")
print(f"Written {OUT}/with_nulls.parquet  ({len(null_table)} rows, {len(null_table.schema)} cols)")

print("Done.")
