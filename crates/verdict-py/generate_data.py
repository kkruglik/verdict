"""Generate sample CSV data for benchmarking. Usage: generate_data.py <rows> <path>"""
import sys
import numpy as np
import pandas as pd

COUNTRIES = ["US", "UK", "DE", "FR", "JP"]
NULL_RATE = 0.05


def generate(rows: int, path: str):
    rng = np.random.default_rng(42)

    n = rows
    null_mask = lambda: rng.random(n) < NULL_RATE

    user_id = np.arange(1, n + 1, dtype=np.int64)
    score = np.round(rng.uniform(0, 100, n), 2)
    age = rng.integers(18, 91, n)
    is_active = rng.integers(0, 2, n).astype(bool)
    country_codes = rng.integers(0, len(COUNTRIES), n)
    country = np.array(COUNTRIES)[country_codes]

    score_with_nulls = score.astype(object)
    score_with_nulls[null_mask()] = np.nan

    age_with_nulls = age.astype(object)
    age_with_nulls[null_mask()] = pd.NA

    active_with_nulls = is_active.astype(object)
    active_with_nulls[null_mask()] = pd.NA

    country_with_nulls = country.astype(object)
    country_with_nulls[null_mask()] = None

    df = pd.DataFrame({
        "user_id": user_id,
        "score": score,
        "score_with_nulls": score_with_nulls,
        "age": age,
        "age_with_nulls": age_with_nulls,
        "is_active": is_active,
        "is_active_with_nulls": active_with_nulls,
        "country": country,
        "country_with_nulls": country_with_nulls,
    })

    df.to_csv(path, index=False)
    print(f"wrote {rows:,} rows → {path}")


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("usage: generate_data.py <rows> <path>")
        sys.exit(1)
    generate(int(sys.argv[1]), sys.argv[2])
