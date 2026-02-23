"""Generate sample_1m.csv with 1,000,000 rows for benchmarking."""
import random
import csv
import sys

ROWS = 1_000_000
COUNTRIES = ["US", "UK", "DE", "FR", "JP"]
NULL_RATE = 0.05


def generate(path: str, rows: int):
    rng = random.Random(42)

    with open(path, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow([
            "user_id", "score", "score_with_nulls",
            "age", "age_with_nulls",
            "is_active", "is_active_with_nulls",
            "country", "country_with_nulls",
        ])
        for i in range(1, rows + 1):
            score = round(rng.uniform(0, 100), 2)
            age = rng.randint(18, 90)
            is_active = rng.choice([True, False])
            country = rng.choice(COUNTRIES)

            score_null = "" if rng.random() < NULL_RATE else round(rng.uniform(0, 100), 2)
            age_null = "" if rng.random() < NULL_RATE else rng.randint(18, 90)
            active_null = "" if rng.random() < NULL_RATE else rng.choice([True, False])
            country_null = "" if rng.random() < NULL_RATE else rng.choice(COUNTRIES)

            writer.writerow([
                i, score, score_null,
                age, age_null,
                is_active, active_null,
                country, country_null,
            ])

    print(f"wrote {rows:,} rows → {path}")


if __name__ == "__main__":
    path = sys.argv[1] if len(sys.argv) > 1 else "sample_1m.csv"
    generate(path, ROWS)
