from typing import Self
from verdict_py import Dataset, Column, Constraint, Rule, Schema, DataType, RuleBuilder, py_validate
from enum import StrEnum


class Dtype(StrEnum):
    string = "STRING"
    floating = "FLOATING"
    integer = "INTEGER"


# NOTE: API design sketches below — not functional, kept for reference.
# class BaseColumn: ...
# class BaseDataset: ...
# class VerdictColumn: ...


dataset = Dataset(
    headers=["id", "name", "share", "age"],
    columns=[
        Column.integer([1, 2, 3, 4]),
        Column.string(["ann", "clark", "lana", "lex"]),
        Column.floating([20.3, 2.1, 3.9, 40.0]),
        Column.integer([20, None, 30, 40]),
    ],
)


def section(title):
    print(f"\n{'=' * 50}")
    print(f"  {title}")
    print(f"{'=' * 50}")


def explore_dataset():
    section("Dataset")
    print(f"repr:              {dataset}")
    print(f"shape:             {dataset.shape()}")
    print(f"get_by_name(id):   {dataset.get_column_by_name('id')}")
    print(f"get_by_name(name): {dataset.get_column_by_name('name')}")
    print(f"get_by_index(0):   {dataset.get_column_by_index(0)}")
    print(f"get_index(id):     {dataset.get_column_index('id')}")
    print(f"missing column:    {dataset.get_column_by_name('nonexistent')}")


# NOTE: column ops commented out — see lib.rs for rationale.
# def explore_basic_ops(): ...
# def explore_numeric_ops(): ...
# def explore_comparison_ops(): ...
# def explore_string_ops(): ...


def explore_validation():
    section("Validation")
    rules = [
        *RuleBuilder("id").not_null().unique().build(),
        *RuleBuilder("age").not_null().gt(0.0).between(18.0, 99.0).build(),
        *RuleBuilder("name")
        .not_null()
        .contains("a")
        .is_in(["ann", "clark", "lana", "lex"])
        .starts_with("a")
        .matches_regex("^[a-z]+")
        .length_between(2, 10)
        .build(),
        *RuleBuilder("share").gt(0.0).between(1.0, 50.0).build(),
    ]
    report = py_validate(dataset, rules)
    for r in report.results:
        print(r)


def explore_datetime():
    section("Date and DateTime columns")

    # Epoch days since Unix epoch (1970-01-01):
    #   18262 = 2020-01-01
    #   19723 = 2024-01-01
    #   19800 = 2024-03-18
    # Epoch microseconds:
    #   1577836800000000 = 2020-01-01T00:00:00
    #   1704067200000000 = 2024-01-01T00:00:00
    #   1710720000000000 = 2024-03-18T00:00:00

    date_col = Column.date([18262, 19723, 19800, None])
    dt_col = Column.datetime([1577836800000000, 1704067200000000, 1710720000000000, None])

    print(f"date column:     {date_col}")
    print(f"datetime column: {dt_col}")

    ds = Dataset(
        headers=["signup_date", "last_seen_at"],
        columns=[date_col, dt_col],
    )
    print(f"dataset:         {ds}")

    rules = [
        Rule("signup_date", Constraint.after("2019-12-31")),
        Rule("signup_date", Constraint.before("2025-01-01")),
        Rule("last_seen_at", Constraint.between_dates("2019-12-31T00:00:00", "2025-01-01T00:00:00")),
    ]
    report = py_validate(ds, rules)
    print(f"passed: {report.passed}")
    for r in report.results:
        print(f"  {r}")


def explore_datetime_from_csv():
    section("Date and DateTime from CSV")

    schema = Schema([
        ("created_date", DataType.date()),
        ("created_at", DataType.datetime()),
    ])

    # Schema for from_csv requires format — use a schema JSON file via CLI,
    # or build via the Field-level API. The Python Schema class doesn't expose
    # format yet (format lives on Field, not DataType). Use from_csv via CLI
    # or pass a pre-built dataset for now.
    # Shown here: loading via the full schema using from_csv with a config file.
    import os
    fixture = os.path.join(
        os.path.dirname(__file__),
        "../../fixtures/sample_with_dates.csv",
    )
    schema_full = Schema([
        ("user_id", DataType.integer()),
        ("score", DataType.float()),
        ("score_with_nulls", DataType.float()),
        ("age", DataType.integer()),
        ("age_with_nulls", DataType.integer()),
        ("is_active", DataType.boolean()),
        ("is_active_with_nulls", DataType.boolean()),
        ("country", DataType.string()),
        ("country_with_nulls", DataType.string()),
        ("created_date", DataType.date()),
        ("created_date_with_nulls", DataType.date()),
        ("created_at", DataType.datetime()),
        ("created_at_with_nulls", DataType.datetime()),
    ])
    try:
        ds = Dataset.from_csv(fixture, schema_full)
        print(f"loaded: {ds}")
        rules = [
            Rule("created_date", Constraint.after("2019-12-31")),
            Rule("created_date", Constraint.before("2025-01-01")),
            Rule("created_at", Constraint.between_dates("2019-12-31T00:00:00", "2025-01-01T00:00:00")),
        ]
        report = py_validate(ds, rules)
        print(f"passed: {report.passed}")
        for r in report.results:
            print(f"  {r}")
    except Exception as e:
        print(f"error: {e}")


if __name__ == "__main__":
    explore_dataset()
    explore_validation()
    explore_datetime()
    explore_datetime_from_csv()
