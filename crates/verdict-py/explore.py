from typing import Self
from verdict_py import Dataset, Column, RuleBuilder, py_validate
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
    results = py_validate(dataset, rules)
    for r in results:
        print(r)


if __name__ == "__main__":
    explore_dataset()
    explore_validation()
