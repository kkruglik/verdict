import pandas as pd
import numpy as np
import pyarrow as pa
import pyarrow.parquet as pq
from pathlib import Path

FIXTURES = Path(__file__).parent.parent / "fixtures"

N = 200
NULL_RATE = 0.1
START_DATE = pd.Timestamp("2020-01-01")
END_DATE = pd.Timestamp("2024-12-31")

rng = np.random.default_rng(42)

null_mask = lambda: rng.random(N) < NULL_RATE

date_range_days = (END_DATE - START_DATE).days
dates = pd.to_datetime(START_DATE.value + rng.integers(0, date_range_days, N) * 86400 * 10**9)
datetimes = pd.to_datetime(START_DATE.value + rng.integers(0, date_range_days * 86400, N) * 10**9)

# generate times as seconds-of-day, then format as HH:MM:SS
seconds_of_day = rng.integers(0, 86400, N)
millis_of_day = (seconds_of_day * 1000).astype(np.int32)
null_mask_time = null_mask()
millis_with_nulls = [None if null_mask_time[i] else int(millis_of_day[i]) for i in range(N)]

dates_col = pa.array(dates.date, type=pa.date32())
datetimes_col = pa.array(datetimes.astype("int64").values // 1000, type=pa.timestamp("us"))
time_col = pa.array(millis_of_day.tolist(), type=pa.time32("ms"))
time_nulls_col = pa.array(millis_with_nulls, type=pa.time32("ms"))

table = pa.table({
    "user_id": pa.array(np.arange(1, N + 1), type=pa.int64()),
    "created_date": dates_col,
    "created_at": datetimes_col,
    "event_time": time_col,
    "event_time_with_nulls": time_nulls_col,
})

out = FIXTURES / "sample_with_time.parquet"
pq.write_table(table, out)
print(f"written {N} rows -> {out}")
