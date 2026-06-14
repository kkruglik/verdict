"""
Generates 30 CLI integration test fixtures in fixtures/cli_tests/.
Each case: data.parquet or data.csv + schema.json + expected.json.

Cases 01-20: parquet, covering all supported parquet dtypes.
Cases 21-30: csv, covering int/float/str/date/datetime.

Run: python scripts/gen_cli_tests.py
"""
import csv
import datetime
import json
import os

import pyarrow as pa
import pyarrow.parquet as pq

BASE = os.path.join(os.path.dirname(__file__), "..", "fixtures", "cli_tests")


def ms(h, m=0, s=0):
    return (h * 3600 + m * 60 + s) * 1000


def us(h, m=0, s=0):
    return (h * 3600 + m * 60 + s) * 1_000_000


def write_parquet(name, table, schema_dict, expected):
    path = os.path.join(BASE, name)
    os.makedirs(path, exist_ok=True)
    pq.write_table(table, os.path.join(path, "data.parquet"))
    with open(os.path.join(path, "schema.json"), "w") as f:
        json.dump(schema_dict, f, indent=2)
    with open(os.path.join(path, "expected.json"), "w") as f:
        json.dump(expected, f, indent=2)
    print(f"  {name}")


def write_csv(name, rows, fieldnames, schema_dict, expected):
    path = os.path.join(BASE, name)
    os.makedirs(path, exist_ok=True)
    with open(os.path.join(path, "data.csv"), "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for row in rows:
            writer.writerow({k: ("" if v is None else v) for k, v in row.items()})
    with open(os.path.join(path, "schema.json"), "w") as f:
        json.dump(schema_dict, f, indent=2)
    with open(os.path.join(path, "expected.json"), "w") as f:
        json.dump(expected, f, indent=2)
    print(f"  {name}")


def col(name, dtype, constraints, fmt=None):
    c = {"name": name, "dtype": dtype, "constraints": constraints}
    if fmt:
        c["format"] = fmt
    return c


def c(key, value):
    return {"constraint": key, "value": value}


def e(column, constraint_name, passed):
    return {"column": column, "constraint": constraint_name, "passed": passed}


# ── Parquet cases ─────────────────────────────────────────────────────────────

def case_01_orders():
    """int64 + float64: realistic order data. Discount ceiling fails."""
    n = 30
    ids = list(range(1, n + 1))
    amounts = [round(50.0 + i * 30.0, 2) for i in range(n)]       # 50..920, all > 0
    quantities = [(i % 9) + 1 for i in range(n)]                   # 1..9, all > 0
    # 22 discounts <= 0.3, 8 discounts > 0.3
    discounts = [round(0.05 + (i % 6) * 0.04, 2) for i in range(22)] + \
                [round(0.35 + (i % 4) * 0.05, 2) for i in range(8)]

    table = pa.table({
        "order_id":  pa.array(ids,        type=pa.int64()),
        "amount":    pa.array(amounts,    type=pa.float64()),
        "quantity":  pa.array(quantities, type=pa.int64()),
        "discount":  pa.array(discounts,  type=pa.float64()),
    })
    schema_dict = {"columns": [
        col("order_id", "int",   [c("not_null", True), c("unique", True)]),
        col("amount",   "float", [c("ge", 0.0)]),
        col("quantity", "int",   [c("gt", 0)]),
        col("discount", "float", [c("le", 0.3)]),
    ]}
    expected = [
        e("order_id", "not_null", True),
        e("order_id", "unique",   True),
        e("amount",   "ge",       True),
        e("quantity", "gt",       True),
        e("discount", "le",       False),
    ]
    write_parquet("01_orders", table, schema_dict, expected)


def case_02_users():
    """int64 + utf8 + bool: user records. Status set and is_verified nullability fail."""
    n = 25
    ids      = list(range(1001, 1001 + n))
    emails   = [f"user{i}@example.com" for i in range(n)]
    # 15 valid statuses, 10 "pending" (not in allowed set)
    statuses = ["active", "inactive"] * 7 + ["pending"] * 11
    statuses = statuses[:n]
    # 18 non-null, 7 null
    verified = [True, False, True, True, False] * 3 + [None, None, None, None, None] + \
               [True, False, True, True, False] * 1
    verified = verified[:n]

    table = pa.table({
        "user_id":     pa.array(ids,      type=pa.int64()),
        "email":       pa.array(emails,   type=pa.utf8()),
        "status":      pa.array(statuses, type=pa.utf8()),
        "is_verified": pa.array(verified, type=pa.bool_()),
    })
    schema_dict = {"columns": [
        col("user_id",     "int",  [c("not_null", True), c("unique", True)]),
        col("email",       "str",  [c("contains", "@")]),
        col("status",      "str",  [c("is_in", ["active", "inactive"])]),
        col("is_verified", "bool", [c("not_null", True)]),
    ]}
    expected = [
        e("user_id",     "not_null", True),
        e("user_id",     "unique",   True),
        e("email",       "contains", True),
        e("status",      "is_in",    False),
        e("is_verified", "not_null", False),
    ]
    write_parquet("02_users", table, schema_dict, expected)


def case_03_products():
    """int64 + float64 + utf8: product catalog. Price ceiling and SKU prefix fail."""
    n = 30
    ids    = list(range(1, n + 1))
    # 20 prices <= 200, 10 > 200; all > 0
    prices = [round(9.99 + i * 9.5, 2) for i in range(20)] + \
             [round(250.0 + i * 25.0, 2) for i in range(10)]
    names  = [f"Product {i}" for i in range(1, n + 1)]
    # 20 valid SKU-XXXX, 10 without prefix
    skus   = [f"SKU-{i:04d}" for i in range(1, 21)] + \
             [f"PROD{i:04d}" for i in range(1, 11)]

    table = pa.table({
        "product_id": pa.array(ids,    type=pa.int64()),
        "price":      pa.array(prices, type=pa.float64()),
        "name":       pa.array(names,  type=pa.utf8()),
        "sku":        pa.array(skus,   type=pa.utf8()),
    })
    schema_dict = {"columns": [
        col("product_id", "int",   [c("ge", 1)]),
        col("price",      "float", [c("gt", 0.0), c("le", 200.0)]),
        col("name",       "str",   [c("not_null", True)]),
        col("sku",        "str",   [c("starts_with", "SKU-")]),
    ]}
    expected = [
        e("product_id", "ge",         True),
        e("price",      "gt",         True),
        e("price",      "le",         False),
        e("name",       "not_null",   True),
        e("sku",        "starts_with", False),
    ]
    write_parquet("03_products", table, schema_dict, expected)


def case_04_inventory():
    """int64 + float64: warehouse inventory. Max stock ceiling and reorder range fail."""
    n = 28
    item_ids    = list(range(100, 100 + n))
    stocks      = [(i * 7) % 150 for i in range(n)]                  # 0..147, all >= 0
    costs       = [round(0.5 + i * 0.75, 2) for i in range(n)]      # all > 0
    # 20 max_stock < 500, 8 >= 500
    max_stocks  = [100 + i * 18 for i in range(20)] + \
                  [500, 600, 750, 900, 1000, 1200, 1400, 1600]
    # 20 reorder_level in [1,50], 8 outside (0 or >50)
    reorders    = [1 + (i % 50) for i in range(20)] + \
                  [0, 0, 0, 60, 75, 100, 120, 200]

    table = pa.table({
        "item_id":       pa.array(item_ids,   type=pa.int64()),
        "stock":         pa.array(stocks,     type=pa.int64()),
        "cost_per_unit": pa.array(costs,      type=pa.float64()),
        "max_stock":     pa.array(max_stocks, type=pa.int64()),
        "reorder_level": pa.array(reorders,   type=pa.int64()),
    })
    schema_dict = {"columns": [
        col("item_id",       "int",   [c("unique", True)]),
        col("stock",         "int",   [c("ge", 0)]),
        col("cost_per_unit", "float", [c("gt", 0.0)]),
        col("max_stock",     "int",   [c("lt", 500)]),
        col("reorder_level", "int",   [c("between", [1, 50])]),
    ]}
    expected = [
        e("item_id",       "unique",  True),
        e("stock",         "ge",      True),
        e("cost_per_unit", "gt",      True),
        e("max_stock",     "lt",      False),
        e("reorder_level", "between", False),
    ]
    write_parquet("04_inventory", table, schema_dict, expected)


def case_05_measurements():
    """float64 + int64: sensor readings. Temperature range and pressure floor fail."""
    n = 30
    sensor_ids  = list(range(1, n + 1))
    humidity    = [round(20.0 + (i % 70), 1) for i in range(n)]    # 20..89, all in [0,100]
    # 20 in [-10,40], 5 above 40, 5 below -10
    temperature = [round(-5.0 + i * 2.0, 1) for i in range(20)] + \
                  [round(45.0 + i * 2.0, 1) for i in range(5)] + \
                  [round(-15.0 - i * 2.0, 1) for i in range(5)]
    # 20 > 900, 10 <= 900
    pressure    = [round(920.0 + i * 5.0, 1) for i in range(20)] + \
                  [round(850.0 + i * 5.0, 1) for i in range(10)]

    table = pa.table({
        "sensor_id":   pa.array(sensor_ids,  type=pa.int64()),
        "humidity":    pa.array(humidity,    type=pa.float64()),
        "temperature": pa.array(temperature, type=pa.float64()),
        "pressure":    pa.array(pressure,    type=pa.float64()),
    })
    schema_dict = {"columns": [
        col("sensor_id",   "int",   [c("not_null", True)]),
        col("humidity",    "float", [c("between", [0.0, 100.0])]),
        col("temperature", "float", [c("between", [-10.0, 40.0])]),
        col("pressure",    "float", [c("gt", 900.0)]),
    ]}
    expected = [
        e("sensor_id",   "not_null", True),
        e("humidity",    "between",  True),
        e("temperature", "between",  False),
        e("pressure",    "gt",       False),
    ]
    write_parquet("05_measurements", table, schema_dict, expected)


def case_06_text_ops():
    """utf8 only: text operations. Code regex and tag prefix fail."""
    n = 30
    categories   = (["A", "B", "C", "D"] * 8)[:n]                   # all in set
    descriptions = [f"Description for item {i}: sufficient detail." for i in range(n)]
    # 20 valid codes matching ^[A-Z]{3}-\d{4}$, 10 invalid
    codes = [f"ABC-{i:04d}" for i in range(1, 21)] + \
            ["invalid-1", "xy1", "ABCD1234", "ab-0001", "ABC_0001",
             "abc-0001", "AB-0001", "ABCDE-001", "ABC-12", "0AB-0001"]
    # 20 with "tag_" prefix, 10 without
    tags = [f"tag_{i}" for i in range(1, 21)] + \
           [f"label_{i}" for i in range(1, 6)] + \
           [f"item{i}" for i in range(1, 6)]

    table = pa.table({
        "category":    pa.array(categories,   type=pa.utf8()),
        "description": pa.array(descriptions, type=pa.utf8()),
        "code":        pa.array(codes,        type=pa.utf8()),
        "tag":         pa.array(tags,         type=pa.utf8()),
    })
    schema_dict = {"columns": [
        col("category",    "str", [c("is_in", ["A", "B", "C", "D"])]),
        col("description", "str", [c("length_between", [5, 200])]),
        col("code",        "str", [c("matches_regex", "^[A-Z]{3}-\\d{4}$")]),
        col("tag",         "str", [c("starts_with", "tag_")]),
    ]}
    expected = [
        e("category",    "is_in",          True),
        e("description", "length_between", True),
        e("code",        "matches_regex",  False),
        e("tag",         "starts_with",    False),
    ]
    write_parquet("06_text_ops", table, schema_dict, expected)


def case_07_dates_basic():
    """date32: date constraints. created_date range fails."""
    n = 25
    base = datetime.date(2021, 6, 1)
    event_dates  = [base + datetime.timedelta(days=i * 15) for i in range(n)]
    # last: 2021-06-01 + 360d = 2022-05-27, all after 2020-01-01 ✓
    expiry_dates = [datetime.date(2024, 6, 1) + datetime.timedelta(days=i * 30) for i in range(n)]
    # last: 2024-06-01 + 720d ≈ 2026-05-22, all before 2030-01-01 ✓
    # 17 created_dates in [2021-01-01, 2023-12-31], 8 outside
    created_in  = [datetime.date(2021, 6, 1) + datetime.timedelta(days=i * 40) for i in range(17)]
    created_out = [
        datetime.date(2020, 5, 15), datetime.date(2019, 12, 31),
        datetime.date(2024, 3, 1),  datetime.date(2025, 6, 15),
        datetime.date(2020, 1, 1),  datetime.date(2024, 7, 4),
        datetime.date(2025, 1, 1),  datetime.date(2026, 6, 30),
    ]
    created_dates = created_in + created_out

    table = pa.table({
        "event_date":   pa.array(event_dates,   type=pa.date32()),
        "expiry_date":  pa.array(expiry_dates,  type=pa.date32()),
        "created_date": pa.array(created_dates, type=pa.date32()),
    })
    schema_dict = {"columns": [
        col("event_date",   "date", [c("not_null", True), c("after", "2020-01-01")]),
        col("expiry_date",  "date", [c("before", "2030-01-01")]),
        col("created_date", "date", [c("between_dates", ["2021-01-01", "2023-12-31"])]),
    ]}
    expected = [
        e("event_date",   "not_null",      True),
        e("event_date",   "after",         True),
        e("expiry_date",  "before",        True),
        e("created_date", "between_dates", False),
    ]
    write_parquet("07_dates_basic", table, schema_dict, expected)


def case_08_dates_is_in():
    """date32: date membership and nullability. ref_date not_null and quarter_end before fail."""
    n = 20
    holidays = [
        datetime.date(2023, 1, 1), datetime.date(2023, 7, 4),
        datetime.date(2023, 12, 25), datetime.date(2023, 11, 23),
        datetime.date(2023, 5, 29),
    ]
    holiday_dates = (holidays * 4)[:n]                                # all in allowed set
    # 14 non-null, 6 null
    ref_dates = [datetime.date(2023, 1, i + 1) for i in range(14)] + [None] * 6
    # 12 before 2024-01-01, 8 in 2024+
    qe_pass = [
        datetime.date(2023, 3, 31), datetime.date(2023, 6, 30),
        datetime.date(2023, 9, 30), datetime.date(2022, 12, 31),
        datetime.date(2022, 3, 31), datetime.date(2022, 6, 30),
        datetime.date(2021, 9, 30), datetime.date(2021, 12, 31),
        datetime.date(2020, 3, 31), datetime.date(2020, 6, 30),
        datetime.date(2020, 9, 30), datetime.date(2020, 12, 31),
    ]
    qe_fail = [
        datetime.date(2024, 3, 31), datetime.date(2024, 6, 30),
        datetime.date(2024, 9, 30), datetime.date(2024, 12, 31),
        datetime.date(2025, 3, 31), datetime.date(2025, 6, 30),
        datetime.date(2025, 9, 30), datetime.date(2025, 12, 31),
    ]
    quarter_ends = qe_pass + qe_fail

    holiday_set_str = ["2023-01-01", "2023-07-04", "2023-12-25", "2023-11-23", "2023-05-29"]

    table = pa.table({
        "holiday_date": pa.array(holiday_dates, type=pa.date32()),
        "ref_date":     pa.array(ref_dates,     type=pa.date32()),
        "quarter_end":  pa.array(quarter_ends,  type=pa.date32()),
    })
    schema_dict = {"columns": [
        col("holiday_date", "date", [c("is_in", holiday_set_str)]),
        col("ref_date",     "date", [c("not_null", True)]),
        col("quarter_end",  "date", [c("before", "2024-01-01")]),
    ]}
    expected = [
        e("holiday_date", "is_in",    True),
        e("ref_date",     "not_null", False),
        e("quarter_end",  "before",   False),
    ]
    write_parquet("08_dates_is_in", table, schema_dict, expected)


def case_09_events_ts_ms():
    """timestamp(ms): event log. updated_at ceiling and uniqueness fail."""
    n = 25
    base = datetime.datetime(2022, 6, 1)
    created_at = [base + datetime.timedelta(days=i * 10) for i in range(n)]
    # all after 2022-01-01 ✓
    # 12 updated_at before 2024-01-01, 10 after, 3 duplicates
    upd_pass = [datetime.datetime(2023, 1, 1) + datetime.timedelta(days=i * 20) for i in range(12)]
    upd_fail = [datetime.datetime(2024, 2, 1) + datetime.timedelta(days=i * 30) for i in range(10)]
    dup      = [datetime.datetime(2023, 3, 15)] * 3
    updated_at = upd_pass + upd_fail + dup

    table = pa.table({
        "created_at": pa.array(created_at, type=pa.timestamp("ms")),
        "updated_at": pa.array(updated_at, type=pa.timestamp("ms")),
    })
    schema_dict = {"columns": [
        col("created_at", "datetime", [c("not_null", True), c("after", "2022-01-01T00:00:00")]),
        col("updated_at", "datetime", [c("before", "2024-01-01T00:00:00"), c("unique", True)]),
    ]}
    expected = [
        e("created_at", "not_null", True),
        e("created_at", "after",    True),
        e("updated_at", "before",   False),
        e("updated_at", "unique",   False),
    ]
    write_parquet("09_events_ts_ms", table, schema_dict, expected)


def case_10_events_ts_us():
    """timestamp(us): high-resolution event log. processed_at ceiling and uniqueness fail."""
    n = 25
    base = datetime.datetime(2021, 7, 1)
    logged_at = [base + datetime.timedelta(days=i * 14) for i in range(n)]
    # last ≈ 2022-06-02, all after 2021-06-01 ✓
    proc_pass = [datetime.datetime(2022, 1, 1) + datetime.timedelta(days=i * 15) for i in range(12)]
    # last ≈ 2022-06-15, all before 2023-01-01 ✓
    proc_fail = [datetime.datetime(2023, 3, 1) + datetime.timedelta(days=i * 20) for i in range(10)]
    dup       = [datetime.datetime(2022, 6, 15, 12, 0)] * 3
    processed_at = proc_pass + proc_fail + dup

    table = pa.table({
        "logged_at":    pa.array(logged_at,    type=pa.timestamp("us")),
        "processed_at": pa.array(processed_at, type=pa.timestamp("us")),
    })
    schema_dict = {"columns": [
        col("logged_at",    "datetime", [c("not_null", True), c("after", "2021-06-01T00:00:00")]),
        col("processed_at", "datetime", [c("before", "2023-01-01T00:00:00"), c("unique", True)]),
    ]}
    expected = [
        e("logged_at",    "not_null", True),
        e("logged_at",    "after",    True),
        e("processed_at", "before",   False),
        e("processed_at", "unique",   False),
    ]
    write_parquet("10_events_ts_us", table, schema_dict, expected)


def case_11_mixed_ts():
    """timestamp(ms) + timestamp(us): mixed precision timestamps. duration range fails."""
    n = 25
    start_ms = [datetime.datetime(2021, 1, 1) + datetime.timedelta(days=i * 30) for i in range(n)]
    # last ≈ 2022-12-22, all after 2020-01-01 ✓
    end_us = [datetime.datetime(2023, 6, 1) + datetime.timedelta(days=i * 20) for i in range(n)]
    # last ≈ 2024-09-23, all before 2026-01-01 ✓
    # 17 in [2022-01-01, 2024-01-01], 8 outside
    dur_in = [datetime.datetime(2022, 3, 1) + datetime.timedelta(days=i * 35) for i in range(17)]
    # last ≈ 2023-11-08, within range ✓
    dur_out = [
        datetime.datetime(2021, 6, 1),  datetime.datetime(2020, 12, 1),
        datetime.datetime(2019, 8, 15), datetime.datetime(2024, 3, 1),
        datetime.datetime(2024, 9, 15), datetime.datetime(2025, 1, 1),
        datetime.datetime(2025, 6, 30), datetime.datetime(2025, 11, 30),
    ]
    duration_ms = dur_in + dur_out

    table = pa.table({
        "start_ms":    pa.array(start_ms,    type=pa.timestamp("ms")),
        "end_us":      pa.array(end_us,      type=pa.timestamp("us")),
        "duration_ms": pa.array(duration_ms, type=pa.timestamp("ms")),
    })
    schema_dict = {"columns": [
        col("start_ms",    "datetime", [c("not_null", True), c("after", "2020-01-01T00:00:00")]),
        col("end_us",      "datetime", [c("before", "2026-01-01T00:00:00")]),
        col("duration_ms", "datetime", [c("between_dates", ["2022-01-01T00:00:00", "2024-01-01T00:00:00"])]),
    ]}
    expected = [
        e("start_ms",    "not_null",      True),
        e("start_ms",    "after",         True),
        e("end_us",      "before",        True),
        e("duration_ms", "between_dates", False),
    ]
    write_parquet("11_mixed_ts", table, schema_dict, expected)


def case_12_sessions_ms():
    """time32(ms): work sessions. break_time nullability and lunch window fail."""
    n = 25
    # open_time: all after 06:00:00
    open_times  = [ms(7 + (i % 4)) for i in range(n)]              # 07:00..10:00
    # close_time: all before 23:00:00
    close_times = [ms(17 + (i % 5)) for i in range(n)]             # 17:00..21:00
    # break_time: 18 non-null, 7 null
    break_vals  = [ms(10, 30), ms(11), ms(10), ms(15, 30)] * 4 + [ms(10, 30), ms(11)]
    break_times = break_vals[:18] + [None] * 7
    # lunch_time: 15 in [11:00,14:00], 10 outside
    lunch_in  = [ms(11), ms(11, 30), ms(12), ms(12, 30), ms(13), ms(13, 30), ms(14)] * 2 + [ms(11, 45)]
    lunch_out = [ms(9), ms(9, 30), ms(10), ms(10, 30), ms(15), ms(15, 30), ms(16), ms(8), ms(7, 30), ms(17)]
    lunch_times = lunch_in + lunch_out

    table = pa.table({
        "open_time":  pa.array(open_times,  type=pa.time32("ms")),
        "close_time": pa.array(close_times, type=pa.time32("ms")),
        "break_time": pa.array(break_times, type=pa.time32("ms")),
        "lunch_time": pa.array(lunch_times, type=pa.time32("ms")),
    })
    schema_dict = {"columns": [
        col("open_time",  "time", [c("after", "06:00:00")]),
        col("close_time", "time", [c("before", "23:00:00")]),
        col("break_time", "time", [c("not_null", True)]),
        col("lunch_time", "time", [c("between", ["11:00:00", "14:00:00"])]),
    ]}
    expected = [
        e("open_time",  "after",    True),
        e("close_time", "before",   True),
        e("break_time", "not_null", False),
        e("lunch_time", "between",  False),
    ]
    write_parquet("12_sessions_ms", table, schema_dict, expected)


def case_13_sessions_us():
    """time64(us): work sessions with microsecond precision. peak_hours window fails."""
    n = 25
    shift_starts = [us(8 + (i % 4)) for i in range(n)]             # 08:00..11:00, all > 07:00
    shift_ends   = [us(14 + (i % 5)) for i in range(n)]            # 14:00..18:00, all < 20:00
    # peak_hours: 15 in [10:00,16:00], 10 outside
    peak_in  = [us(10), us(11), us(12), us(13), us(14), us(15), us(16)] * 2 + [us(12, 30)]
    peak_out = [us(7), us(8), us(9), us(9, 30), us(17), us(18), us(19), us(6), us(5, 30), us(21)]
    peak_hours = peak_in + peak_out

    table = pa.table({
        "shift_start": pa.array(shift_starts, type=pa.time64("us")),
        "shift_end":   pa.array(shift_ends,   type=pa.time64("us")),
        "peak_hours":  pa.array(peak_hours,   type=pa.time64("us")),
    })
    schema_dict = {"columns": [
        col("shift_start", "time", [c("not_null", True), c("after", "07:00:00")]),
        col("shift_end",   "time", [c("before", "20:00:00")]),
        col("peak_hours",  "time", [c("between", ["10:00:00", "16:00:00"])]),
    ]}
    expected = [
        e("shift_start", "not_null", True),
        e("shift_start", "after",    True),
        e("shift_end",   "before",   True),
        e("peak_hours",  "between",  False),
    ]
    write_parquet("13_sessions_us", table, schema_dict, expected)


def case_14_time_is_in():
    """time32(ms) + time64(us): shift membership. shift_us has unlisted value."""
    n = 24
    shift_ms_vals = [ms(8), ms(16), ms(0)] * 8                     # all in allowed set
    shift_us_vals = [us(8), us(16)] * 8 + [us(12)] * 8            # 12:00 not in set

    table = pa.table({
        "shift_ms": pa.array(shift_ms_vals, type=pa.time32("ms")),
        "shift_us": pa.array(shift_us_vals, type=pa.time64("us")),
    })
    schema_dict = {"columns": [
        col("shift_ms", "time", [c("is_in", ["08:00:00", "16:00:00", "00:00:00"])]),
        col("shift_us", "time", [c("is_in", ["08:00:00", "16:00:00"])]),
    ]}
    expected = [
        e("shift_ms", "is_in", True),
        e("shift_us", "is_in", False),
    ]
    write_parquet("14_time_is_in", table, schema_dict, expected)


def case_15_bookings():
    """date32 + time32(ms) + timestamp(ms): booking system. is_in and checkout time fail."""
    n = 20
    allowed_dates = [
        datetime.date(2023, 6, 1),  datetime.date(2023, 7, 15),
        datetime.date(2023, 8, 20), datetime.date(2024, 1, 10),
        datetime.date(2024, 3, 5),
    ]
    # 10 booking_dates in allowed set, 10 not in set (all after 2023-01-01)
    booking_dates = allowed_dates * 2 + [datetime.date(2023, 4, 10)] * 10
    allowed_str   = ["2023-06-01", "2023-07-15", "2023-08-20", "2024-01-10", "2024-03-05"]

    check_in_times  = [ms(13 + (i % 6)) for i in range(n)]         # 13:00..18:00, all > 12:00
    # checkout: 12 before 11:00, 8 after
    co_pass = [ms(8), ms(9), ms(10), ms(10, 30)] * 3
    co_fail = [ms(11, 30), ms(12), ms(13), ms(14), ms(15), ms(16), ms(12, 30), ms(11, 15)]
    check_out_times = co_pass + co_fail

    reservation_ts  = [datetime.datetime(2023, 1, 1) + datetime.timedelta(days=i * 10) for i in range(n)]

    table = pa.table({
        "booking_date":   pa.array(booking_dates,   type=pa.date32()),
        "check_in_time":  pa.array(check_in_times,  type=pa.time32("ms")),
        "check_out_time": pa.array(check_out_times, type=pa.time32("ms")),
        "reservation_ts": pa.array(reservation_ts,  type=pa.timestamp("ms")),
    })
    schema_dict = {"columns": [
        col("booking_date",   "date",     [c("after", "2023-01-01"), c("is_in", allowed_str)]),
        col("check_in_time",  "time",     [c("after", "12:00:00")]),
        col("check_out_time", "time",     [c("before", "11:00:00")]),
        col("reservation_ts", "datetime", [c("not_null", True)]),
    ]}
    expected = [
        e("booking_date",   "after",    True),
        e("booking_date",   "is_in",    False),
        e("check_in_time",  "after",    True),
        e("check_out_time", "before",   False),
        e("reservation_ts", "not_null", True),
    ]
    write_parquet("15_bookings", table, schema_dict, expected)


def case_16_payments():
    """float64 + date32: payment records. Amount ceiling and settlement date fail."""
    n = 25
    # 17 amounts <= 5000, 8 > 5000; all > 0
    amounts_ok   = [round(100.0 + i * 200.0, 2) for i in range(17)]
    amounts_fail = [round(5500.0 + i * 500.0, 2) for i in range(8)]
    amounts      = amounts_ok + amounts_fail
    fees         = [round((i % 51) * 1.0, 2) for i in range(n)]    # 0..50, all in [0,50]
    payment_dates = [datetime.date(2023, 1, 1) + datetime.timedelta(days=i * 10) for i in range(n)]
    # 17 settlement before 2025-06-01, 8 after
    settle_pass  = [datetime.date(2023, 6, 1) + datetime.timedelta(days=i * 30) for i in range(17)]
    settle_fail  = [datetime.date(2025, 7, 1) + datetime.timedelta(days=i * 30) for i in range(8)]
    settlement   = settle_pass + settle_fail

    table = pa.table({
        "amount":          pa.array(amounts,  type=pa.float64()),
        "fee":             pa.array(fees,     type=pa.float64()),
        "payment_date":    pa.array(payment_dates, type=pa.date32()),
        "settlement_date": pa.array(settlement,    type=pa.date32()),
    })
    schema_dict = {"columns": [
        col("amount",          "float", [c("gt", 0.0), c("le", 5000.0)]),
        col("fee",             "float", [c("between", [0.0, 50.0])]),
        col("payment_date",    "date",  [c("not_null", True)]),
        col("settlement_date", "date",  [c("before", "2025-06-01")]),
    ]}
    expected = [
        e("amount",          "gt",       True),
        e("amount",          "le",       False),
        e("fee",             "between",  True),
        e("payment_date",    "not_null", True),
        e("settlement_date", "before",   False),
    ]
    write_parquet("16_payments", table, schema_dict, expected)


def case_17_employees_pq():
    """int64 + utf8 + date32: HR records. Dept membership and hire date fail."""
    n = 25
    emp_ids = list(range(1000, 1000 + n))
    names   = [f"Employee {i}" for i in range(1, n + 1)]
    depts   = ["Engineering", "Sales", "HR", "Finance"] * 4 + \
              ["Engineering", "Sales", "HR", "Finance", "Engineering", "Sales",
               "HR", "Finance", "Engineering"] + ["Legal"] * 7  # 17 valid, ... hmm
    # Simpler: 18 valid, 7 "Legal"
    valid_depts = ["Engineering", "Sales", "HR", "Finance"]
    depts = [valid_depts[i % 4] for i in range(18)] + ["Legal"] * 7
    # hire_date: 17 before 2024-01-01, 8 in 2024+
    hire_pass = [datetime.date(2015, 1, 1) + datetime.timedelta(days=i * 120) for i in range(17)]
    hire_fail = [datetime.date(2024, 2, 1) + datetime.timedelta(days=i * 30) for i in range(8)]
    hire_dates = hire_pass + hire_fail

    table = pa.table({
        "emp_id":    pa.array(emp_ids,    type=pa.int64()),
        "name":      pa.array(names,      type=pa.utf8()),
        "dept":      pa.array(depts,      type=pa.utf8()),
        "hire_date": pa.array(hire_dates, type=pa.date32()),
    })
    schema_dict = {"columns": [
        col("emp_id",    "int",  [c("not_null", True), c("unique", True), c("ge", 1000)]),
        col("name",      "str",  [c("not_null", True)]),
        col("dept",      "str",  [c("is_in", ["Engineering", "Sales", "HR", "Finance"])]),
        col("hire_date", "date", [c("before", "2024-01-01")]),
    ]}
    expected = [
        e("emp_id",    "not_null", True),
        e("emp_id",    "unique",   True),
        e("emp_id",    "ge",       True),
        e("name",      "not_null", True),
        e("dept",      "is_in",    False),
        e("hire_date", "before",   False),
    ]
    write_parquet("17_employees_pq", table, schema_dict, expected)


def case_18_logs_pq():
    """utf8 + timestamp(us): application logs. Source format fails regex."""
    n = 30
    levels   = ["INFO", "WARN", "ERROR"] * 10
    messages = [f"Log message {i}: operation completed successfully" for i in range(n)]
    ts       = [datetime.datetime(2023, 3, 1) + datetime.timedelta(hours=i * 6) for i in range(n)]
    # all after 2023-01-01 ✓
    # source: 20 valid (lowercase.lowercase), 10 invalid
    valid_sources = ["app.service", "db.connector", "api.handler", "auth.middleware", "cache.store"]
    sources_ok   = [valid_sources[i % 5] for i in range(20)]
    sources_fail = ["AppService", "DB", "API_Handler", "auth", "CACHE.store",
                    "123.service", "app.", ".handler", "123", "App.Service"]
    sources = sources_ok + sources_fail

    table = pa.table({
        "level":   pa.array(levels,   type=pa.utf8()),
        "message": pa.array(messages, type=pa.utf8()),
        "ts":      pa.array(ts,       type=pa.timestamp("us")),
        "source":  pa.array(sources,  type=pa.utf8()),
    })
    schema_dict = {"columns": [
        col("level",   "str",      [c("is_in", ["INFO", "WARN", "ERROR"])]),
        col("message", "str",      [c("not_null", True)]),
        col("ts",      "datetime", [c("after", "2023-01-01T00:00:00")]),
        col("source",  "str",      [c("matches_regex", "^[a-z]+\\.[a-z]+")]),
    ]}
    expected = [
        e("level",   "is_in",         True),
        e("message", "not_null",       True),
        e("ts",      "after",          True),
        e("source",  "matches_regex",  False),
    ]
    write_parquet("18_logs_pq", table, schema_dict, expected)


def case_19_sensors_us():
    """float64 + int64 + timestamp(us): IoT sensors. Uniqueness and quality ceiling fail."""
    n = 25
    # device_id: 17 unique + 8 duplicates
    device_ids   = list(range(1, 18)) + [1, 2, 3, 4, 5, 6, 7, 8]
    readings     = [round(-80.0 + i * 7.0, 2) for i in range(n)]   # -80..88, all in [-100,100]
    # quality: 17 in [0,1], 8 > 1.0
    quality_ok   = [round(i * 0.06, 3) for i in range(17)]          # 0.0..0.96
    quality_fail = [round(1.1 + i * 0.1, 2) for i in range(8)]      # 1.1..1.8
    quality      = quality_ok + quality_fail
    sampled_at   = [datetime.datetime(2024, 2, 1) + datetime.timedelta(hours=i * 4) for i in range(n)]
    # all after 2024-01-01 ✓

    table = pa.table({
        "device_id":    pa.array(device_ids, type=pa.int64()),
        "reading":      pa.array(readings,   type=pa.float64()),
        "quality_score":pa.array(quality,    type=pa.float64()),
        "sampled_at":   pa.array(sampled_at, type=pa.timestamp("us")),
    })
    schema_dict = {"columns": [
        col("device_id",     "int",      [c("unique", True)]),
        col("reading",       "float",    [c("between", [-100.0, 100.0])]),
        col("quality_score", "float",    [c("ge", 0.0), c("le", 1.0)]),
        col("sampled_at",    "datetime", [c("after", "2024-01-01T00:00:00")]),
    ]}
    expected = [
        e("device_id",     "unique",  False),
        e("reading",       "between", True),
        e("quality_score", "ge",      True),
        e("quality_score", "le",      False),
        e("sampled_at",    "after",   True),
    ]
    write_parquet("19_sensors_us", table, schema_dict, expected)


def case_20_full_mixed():
    """All 7 parquet dtypes: label membership and event_time window fail."""
    n = 25
    ids         = list(range(1, n + 1))
    scores      = [round(i * 0.4, 2) for i in range(n)]             # 0.0..9.6, all in [0,10]
    labels      = ["cat1", "cat2", "cat3"] * 5 + ["cat4"] * 10      # 15 valid, 10 "cat4"
    actives     = ([True, False, True] * 8 + [True])[:n]
    ref_dates   = [datetime.date(2021, 1, 1) + datetime.timedelta(days=i * 30) for i in range(n)]
    created_at  = [datetime.datetime(2022, 6, 1) + datetime.timedelta(days=i * 15) for i in range(n)]
    # event_time: 17 in [08:00,20:00], 8 outside
    evt_in  = [ms(8 + (i % 12)) for i in range(17)]                 # 08:00..19:00
    evt_out = [ms(4), ms(5), ms(6), ms(7), ms(21), ms(22), ms(23), ms(3)]
    event_times = evt_in + evt_out

    table = pa.table({
        "id":          pa.array(ids,         type=pa.int64()),
        "score":       pa.array(scores,      type=pa.float64()),
        "label":       pa.array(labels,      type=pa.utf8()),
        "active":      pa.array(actives,     type=pa.bool_()),
        "ref_date":    pa.array(ref_dates,   type=pa.date32()),
        "created_at":  pa.array(created_at,  type=pa.timestamp("ms")),
        "event_time":  pa.array(event_times, type=pa.time32("ms")),
    })
    schema_dict = {"columns": [
        col("id",         "int",      [c("unique", True)]),
        col("score",      "float",    [c("between", [0.0, 10.0])]),
        col("label",      "str",      [c("is_in", ["cat1", "cat2", "cat3"])]),
        col("active",     "bool",     [c("not_null", True)]),
        col("ref_date",   "date",     [c("after", "2020-01-01")]),
        col("created_at", "datetime", [c("not_null", True)]),
        col("event_time", "time",     [c("between", ["08:00:00", "20:00:00"])]),
    ]}
    expected = [
        e("id",         "unique",   True),
        e("score",      "between",  True),
        e("label",      "is_in",    False),
        e("active",     "not_null", True),
        e("ref_date",   "after",    True),
        e("created_at", "not_null", True),
        e("event_time", "between",  False),
    ]
    write_parquet("20_full_mixed", table, schema_dict, expected)


# ── CSV cases ─────────────────────────────────────────────────────────────────

def case_21_orders_csv():
    """int + float CSV: order data. Total ceiling and min_qty floor fail."""
    rows = []
    # 17 rows with total <= 9999; last 8 with total > 9999
    for i in range(17):
        rows.append({
            "order_id": i + 1,
            "total":    round(100.0 + i * 500.0, 2),   # 100..8600
            "min_qty":  i % 5,                          # 0 when i%5==0 → 4 rows fail ge 1
        })
    for i in range(8):
        rows.append({
            "order_id": i + 18,
            "total":    round(10500.0 + i * 1000.0, 2), # 10500..17500
            "min_qty":  1 + (i % 4),                    # all >= 1
        })
    fieldnames  = ["order_id", "total", "min_qty"]
    schema_dict = {"columns": [
        col("order_id", "int",   [c("not_null", True), c("unique", True)]),
        col("total",    "float", [c("gt", 0.0), c("le", 9999.0)]),
        col("min_qty",  "int",   [c("ge", 1)]),
    ]}
    expected = [
        e("order_id", "not_null", True),
        e("order_id", "unique",   True),
        e("total",    "gt",       True),
        e("total",    "le",       False),
        e("min_qty",  "ge",       False),
    ]
    write_csv("21_orders_csv", rows, fieldnames, schema_dict, expected)


def case_22_users_csv():
    """int + str CSV: user accounts. Username length and active nullability fail."""
    n = 25
    rows = []
    roles = ["admin", "user", "viewer"]
    for i in range(n):
        rows.append({
            "user_id":  i + 1,
            "email":    f"user{i}@example.com",
            "username": ("ab" if i % 6 == 0 else f"user_{i:03d}"),  # "ab" is too short (<3)
            "role":     roles[i % 3],
            "active":   ("" if i % 5 == 0 else ("true" if i % 2 == 0 else "false")),
        })
    fieldnames  = ["user_id", "email", "username", "role", "active"]
    schema_dict = {"columns": [
        col("user_id",  "int",  [c("unique", True)]),
        col("email",    "str",  [c("contains", "@")]),
        col("username", "str",  [c("length_between", [3, 30])]),
        col("role",     "str",  [c("is_in", ["admin", "user", "viewer"])]),
        col("active",   "bool", [c("not_null", True)]),
    ]}
    expected = [
        e("user_id",  "unique",         True),
        e("email",    "contains",       True),
        e("username", "length_between", False),
        e("role",     "is_in",          True),
        e("active",   "not_null",       False),
    ]
    write_csv("22_users_csv", rows, fieldnames, schema_dict, expected)


def case_23_scores_csv():
    """int + float + str CSV: student scores. Science ceiling and english floor fail."""
    n = 25
    grades = ["A", "B", "C", "D", "F"]
    rows = []
    for i in range(n):
        rows.append({
            "student_id": i + 1,
            "math":       min(100, 50 + i * 2),          # 50..100, all in [0,100]
            "science":    (80 + i * 2 if i < 10 else 100 + i * 3),  # some > 100
            "english":    (60 + i if i < 15 else -(i - 14) * 5),    # some negative
            "grade":      grades[i % 5],
        })
    fieldnames  = ["student_id", "math", "science", "english", "grade"]
    schema_dict = {"columns": [
        col("student_id", "int",   [c("unique", True)]),
        col("math",       "float", [c("between", [0.0, 100.0])]),
        col("science",    "float", [c("le", 100.0)]),
        col("english",    "float", [c("ge", 0.0)]),
        col("grade",      "str",   [c("is_in", ["A", "B", "C", "D", "F"])]),
    ]}
    expected = [
        e("student_id", "unique",  True),
        e("math",       "between", True),
        e("science",    "le",      False),
        e("english",    "ge",      False),
        e("grade",      "is_in",   True),
    ]
    write_csv("23_scores_csv", rows, fieldnames, schema_dict, expected)


def case_24_contacts_csv():
    """str CSV: contact info. Email regex and phone prefix fail."""
    n = 25
    rows = []
    companies  = ["Acme Corp", "Globex", "Initech", "Umbrella", "Hooli"]
    # website ends with ".com" — 20 valid, 5 invalid
    websites   = [f"https://company{i}.com" for i in range(20)] + \
                 ["https://example.org", "ftp://site.net", "nope", "company.io", "www.test.co.uk"]
    # email: 20 valid "@", 5 invalid (no @)
    emails     = [f"contact{i}@corp.com" for i in range(20)] + \
                 ["notanemail", "missing-at", "bad", "nope.com", "foo"]
    # phone: 20 start with "+1", 5 don't
    phones     = [f"+1-555-{i:04d}" for i in range(20)] + \
                 ["+44-20-7946-0958", "+49-30-1234", "07911123456", "555-1234", "0800-100-200"]
    for i in range(n):
        rows.append({
            "company": companies[i % 5],
            "website": websites[i],
            "email":   emails[i],
            "phone":   phones[i],
        })
    fieldnames  = ["company", "website", "email", "phone"]
    schema_dict = {"columns": [
        col("company", "str", [c("not_null", True)]),
        col("website", "str", [c("ends_with", ".com")]),
        col("email",   "str", [c("matches_regex", ".+@.+\\..+")]),
        col("phone",   "str", [c("starts_with", "+1")]),
    ]}
    expected = [
        e("company", "not_null",      True),
        e("website", "ends_with",     False),
        e("email",   "matches_regex", False),
        e("phone",   "starts_with",   False),
    ]
    write_csv("24_contacts_csv", rows, fieldnames, schema_dict, expected)


def case_25_dates_csv():
    """date + datetime CSV: event scheduling. created_at range fails."""
    n = 24
    rows = []
    for i in range(n):
        event_date = datetime.date(2021, 1, 1) + datetime.timedelta(days=i * 15)
        # created_at: 16 in [2021-01-01, 2024-12-31], 8 outside
        if i < 16:
            created_at = datetime.datetime(2021, 6, 1) + datetime.timedelta(days=i * 50)
        else:
            outside = [
                datetime.datetime(2020, 5, 1), datetime.datetime(2019, 12, 1),
                datetime.datetime(2025, 3, 1), datetime.datetime(2026, 1, 1),
                datetime.datetime(2018, 7, 4), datetime.datetime(2027, 6, 1),
                datetime.datetime(2017, 1, 1), datetime.datetime(2028, 9, 1),
            ][i - 16]
            created_at = outside
        processed_at = datetime.datetime(2022, 1, 1) + datetime.timedelta(days=i * 20)
        rows.append({
            "event_date":   event_date.strftime("%Y-%m-%d"),
            "created_at":   created_at.strftime("%Y-%m-%dT%H:%M:%S"),
            "processed_at": processed_at.strftime("%Y-%m-%dT%H:%M:%S"),
        })
    fieldnames  = ["event_date", "created_at", "processed_at"]
    schema_dict = {"columns": [
        col("event_date",   "date",     [c("not_null", True), c("after", "2020-01-01")], fmt="%Y-%m-%d"),
        col("created_at",   "datetime", [c("between_dates", ["2021-01-01T00:00:00", "2024-12-31T23:59:59"])], fmt="%Y-%m-%dT%H:%M:%S"),
        col("processed_at", "datetime", [c("before", "2026-01-01T00:00:00")],            fmt="%Y-%m-%dT%H:%M:%S"),
    ]}
    expected = [
        e("event_date",   "not_null",      True),
        e("event_date",   "after",         True),
        e("created_at",   "between_dates", False),
        e("processed_at", "before",        True),
    ]
    write_csv("25_dates_csv", rows, fieldnames, schema_dict, expected)


def case_26_inventory_csv():
    """int + float CSV: stock records. Quantity ceiling and total_value floor fail."""
    n = 25
    rows = []
    for i in range(n):
        qty        = 50 + i * 20                         # 50..530; some > 500
        unit_price = round(1.0 + i * 4.0, 2)            # all > 0
        total      = round(qty * unit_price, 2) if i % 7 != 0 else round(-10.0 - i, 2)
        rows.append({
            "item_id":    i + 1,
            "quantity":   qty,
            "unit_price": unit_price,
            "total_value": total,
        })
    fieldnames  = ["item_id", "quantity", "unit_price", "total_value"]
    schema_dict = {"columns": [
        col("item_id",     "int",   [c("not_null", True), c("unique", True)]),
        col("quantity",    "int",   [c("ge", 0), c("le", 500)]),
        col("unit_price",  "float", [c("gt", 0.0)]),
        col("total_value", "float", [c("gt", 0.0)]),
    ]}
    expected = [
        e("item_id",     "not_null", True),
        e("item_id",     "unique",   True),
        e("quantity",    "ge",       True),
        e("quantity",    "le",       False),
        e("unit_price",  "gt",       True),
        e("total_value", "gt",       False),
    ]
    write_csv("26_inventory_csv", rows, fieldnames, schema_dict, expected)


def case_27_articles_csv():
    """str + int CSV: content management. Word count ceiling and author nullability fail."""
    n = 25
    categories = ["tech", "health", "finance", "sports"]
    rows = []
    for i in range(n):
        rows.append({
            "article_id": i + 1,
            "title":      f"Article title number {i + 1}: interesting topic",  # 5..100 chars ✓
            "word_count": 200 + i * 210,        # 200..5240; last 2 rows > 5000
            "category":   categories[i % 4],
            "author":     (None if i % 5 == 0 else f"Author {i}"),
        })
    fieldnames  = ["article_id", "title", "word_count", "category", "author"]
    schema_dict = {"columns": [
        col("article_id", "int", [c("unique", True)]),
        col("title",      "str", [c("not_null", True), c("length_between", [5, 100])]),
        col("word_count", "int", [c("gt", 0), c("le", 5000)]),
        col("category",   "str", [c("is_in", ["tech", "health", "finance", "sports"])]),
        col("author",     "str", [c("not_null", True)]),
    ]}
    expected = [
        e("article_id", "unique",         True),
        e("title",      "not_null",       True),
        e("title",      "length_between", True),
        e("word_count", "gt",             True),
        e("word_count", "le",             False),
        e("category",   "is_in",          True),
        e("author",     "not_null",       False),
    ]
    write_csv("27_articles_csv", rows, fieldnames, schema_dict, expected)


def case_28_table_constraints_csv():
    """int + float CSV with table-level constraints. shape_equals and label not_null fail."""
    n = 10
    rows = []
    for i in range(n):
        rows.append({
            "col_id": i + 1,
            "value":  round(1.0 + i * 5.0, 2),   # all > 0
            "label":  (None if i % 3 == 0 else f"L{i}"),
        })
    fieldnames  = ["col_id", "value", "label"]
    schema_dict = {
        "table": {
            "constraints": [
                # rows_count_between passes (10 rows in [5, 200])
                c("rows_count_between", [5, 200]),
                # columns_exist passes (all 3 columns exist)
                c("columns_exist", ["col_id", "value", "label"]),
                # shape_equals fails (actual is [10, 3], not [10, 4])
                c("shape_equals", [10, 4]),
            ]
        },
        "columns": [
            col("col_id", "int",   [c("not_null", True), c("unique", True)]),
            col("value",  "float", [c("gt", 0.0)]),
            col("label",  "str",   [c("not_null", True)]),
        ],
    }
    expected = [
        e(None, "rows_count_between", True),
        e(None, "columns_exist",      True),
        e(None, "shape_equals",       False),
        e("col_id", "not_null",       True),
        e("col_id", "unique",         True),
        e("value",  "gt",             True),
        e("label",  "not_null",       False),
    ]
    write_csv("28_table_constraints_csv", rows, fieldnames, schema_dict, expected)


def case_29_shipments_csv():
    """str + date CSV: shipment tracking. Tracking prefix and carrier set fail."""
    n = 25
    carriers    = ["UPS", "FedEx", "USPS", "DHL"]
    rows = []
    for i in range(n):
        # tracking_id: 17 with "SHIP-" prefix, 8 without
        tracking = f"SHIP-{i:06d}" if i < 17 else f"TRK{i:06d}"
        ship_date = (datetime.date(2022, 6, 1) + datetime.timedelta(days=i * 10)).strftime("%Y-%m-%d")
        delivery  = (datetime.date(2022, 6, 15) + datetime.timedelta(days=i * 10)).strftime("%Y-%m-%d")
        # carrier: 18 valid, 7 "Express" (not in set)
        carrier   = carriers[i % 4] if i < 18 else "Express"
        rows.append({
            "tracking_id":   tracking,
            "ship_date":     ship_date,
            "delivery_date": delivery,
            "carrier":       carrier,
        })
    fieldnames  = ["tracking_id", "ship_date", "delivery_date", "carrier"]
    schema_dict = {"columns": [
        col("tracking_id",   "str",  [c("not_null", True), c("unique", True), c("starts_with", "SHIP-")]),
        col("ship_date",     "date", [c("after", "2022-01-01")], fmt="%Y-%m-%d"),
        col("delivery_date", "date", [c("before", "2030-01-01")], fmt="%Y-%m-%d"),
        col("carrier",       "str",  [c("is_in", ["UPS", "FedEx", "USPS", "DHL"])]),
    ]}
    expected = [
        e("tracking_id",   "not_null",    True),
        e("tracking_id",   "unique",      True),
        e("tracking_id",   "starts_with", False),
        e("ship_date",     "after",       True),
        e("delivery_date", "before",      True),
        e("carrier",       "is_in",       False),
    ]
    write_csv("29_shipments_csv", rows, fieldnames, schema_dict, expected)


def case_30_reviews_csv():
    """int + float + str CSV: product reviews. Review length and vote ceiling fail."""
    n = 25
    sentiments = ["positive", "negative", "neutral"]
    rows = []
    for i in range(n):
        # review_text: 17 with length >= 10, 8 too short (< 10 chars)
        review = f"Great product, highly recommend!" if i < 17 else "Short"
        rows.append({
            "product_id":    (i % 10) + 1,          # 1..10, all >= 1
            "rating":        round(1.0 + (i % 5) * 1.0, 1),  # 1.0..5.0
            "review_text":   review,
            "sentiment":     sentiments[i % 3],
            "helpful_votes": 100 + i * 25,           # 100..700; some > 500
        })
    fieldnames  = ["product_id", "rating", "review_text", "sentiment", "helpful_votes"]
    schema_dict = {"columns": [
        col("product_id",    "int",   [c("ge", 1)]),
        col("rating",        "float", [c("between", [1.0, 5.0])]),
        col("review_text",   "str",   [c("length_between", [10, 1000])]),
        col("sentiment",     "str",   [c("is_in", ["positive", "negative", "neutral"])]),
        col("helpful_votes", "int",   [c("ge", 0), c("le", 500)]),
    ]}
    expected = [
        e("product_id",    "ge",             True),
        e("rating",        "between",        True),
        e("review_text",   "length_between", False),
        e("sentiment",     "is_in",          True),
        e("helpful_votes", "ge",             True),
        e("helpful_votes", "le",             False),
    ]
    write_csv("30_reviews_csv", rows, fieldnames, schema_dict, expected)


if __name__ == "__main__":
    os.makedirs(BASE, exist_ok=True)
    print("Generating CLI test fixtures...")
    case_01_orders()
    case_02_users()
    case_03_products()
    case_04_inventory()
    case_05_measurements()
    case_06_text_ops()
    case_07_dates_basic()
    case_08_dates_is_in()
    case_09_events_ts_ms()
    case_10_events_ts_us()
    case_11_mixed_ts()
    case_12_sessions_ms()
    case_13_sessions_us()
    case_14_time_is_in()
    case_15_bookings()
    case_16_payments()
    case_17_employees_pq()
    case_18_logs_pq()
    case_19_sensors_us()
    case_20_full_mixed()
    case_21_orders_csv()
    case_22_users_csv()
    case_23_scores_csv()
    case_24_contacts_csv()
    case_25_dates_csv()
    case_26_inventory_csv()
    case_27_articles_csv()
    case_28_table_constraints_csv()
    case_29_shipments_csv()
    case_30_reviews_csv()
    print(f"\nDone — 30 fixtures in {BASE}/")
