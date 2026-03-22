import pandas as pd
import numpy as np
from pathlib import Path

FIXTURES = Path(__file__).parent.parent / "fixtures"

SIZES = {
    "sample": 100_000,
    "sample_10k": 10_000,
    "sample_100k": 100_000,
    "sample_1m": 1_000_000,
    "sample_10m": 10_000_000,
}

NULL_RATE = 0.1
COUNTRIES = ["US", "DE", "JP", "UK", "FR", "CA", "AU", "BR"]
START_DATE = pd.Timestamp("2020-01-01")
END_DATE = pd.Timestamp("2024-12-31")


def generate(n: int, rng: np.random.Generator) -> pd.DataFrame:
    null_mask = lambda: rng.random(n) < NULL_RATE

    date_range_days = (END_DATE - START_DATE).days
    dates = pd.to_datetime(START_DATE.value + rng.integers(0, date_range_days, n) * 86400 * 10**9)
    datetimes = pd.to_datetime(START_DATE.value + rng.integers(0, date_range_days * 86400, n) * 10**9)

    date_null_mask = null_mask()
    dt_null_mask = null_mask()

    dates_str = dates.strftime("%Y-%m-%d")
    datetimes_str = datetimes.strftime("%Y-%m-%dT%H:%M:%S")

    df = pd.DataFrame({
        "user_id": np.arange(1, n + 1),
        "score": rng.uniform(0, 100, n).round(2),
        "score_with_nulls": np.where(null_mask(), None, rng.uniform(0, 100, n).round(2)),
        "age": rng.integers(0, 120, n),
        "age_with_nulls": np.where(null_mask(), None, rng.integers(0, 120, n)),
        "is_active": rng.choice([True, False], n),
        "is_active_with_nulls": np.where(null_mask(), None, rng.choice([True, False], n)),
        "country": rng.choice(COUNTRIES, n),
        "country_with_nulls": np.where(null_mask(), None, rng.choice(COUNTRIES, n)),
        "created_date": dates_str,
        "created_date_with_nulls": np.where(date_null_mask, "", dates_str),
        "created_at": datetimes_str,
        "created_at_with_nulls": np.where(dt_null_mask, "", datetimes_str),
    })

    return df


rng = np.random.default_rng(42)

for name, n in SIZES.items():
    print(f"generating {name} ({n:,} rows)...")
    df = generate(n, rng)
    out = FIXTURES / f"{name}_with_dates.csv"
    df.to_csv(out, index=False)
    print(f"  -> {out}")

print("done")
