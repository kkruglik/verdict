import pytest
from verdict_py import (
    Dataset,
    Column,
    ColumnConstraint,
    ColumnRule,
    ColumnRuleBuilder,
    TableConstraint,
    TableRule,
    Schema,
    DataType,
    py_validate_columns,
    py_validate_table,
    col,
)


FIXTURE_CSV = """\
id,name,score,age,active
1,alice,9.5,25,true
2,bob,8.0,30,false
3,,7.5,35,true
4,diana,6.0,,false
"""


@pytest.fixture
def csv_path(tmp_path):
    p = tmp_path / "test.csv"
    p.write_text(FIXTURE_CSV)
    return str(p)


@pytest.fixture
def compare_dataset():
    # id: Int [1,2,3,4,5]  x: Float [1..5]  y: Float [6..10]  z: Float [28,1,0.5,4,0.9]
    return Dataset(
        headers=["id", "x", "y", "z"],
        columns=[
            Column.integer([1, 2, 3, 4, 5]),
            Column.floating([1.0, 2.0, 3.0, 4.0, 5.0]),
            Column.floating([6.0, 7.0, 8.0, 9.0, 10.0]),
            Column.floating([28.0, 1.0, 0.5, 4.0, 0.9]),
        ],
    )


@pytest.fixture
def compare_nulls_dataset():
    # a: nulls at rows 1,4 — b: null at row 2 — c: same as a — high: all 100.0
    return Dataset(
        headers=["a", "b", "c", "high"],
        columns=[
            Column.floating([1.0, None, 3.0, 4.0, None]),
            Column.floating([2.0, 5.0, None, 5.0, None]),
            Column.floating([1.0, None, 3.0, 4.0, None]),
            Column.floating([100.0, 100.0, 100.0, 100.0, 100.0]),
        ],
    )


@pytest.fixture
def dataset():
    return Dataset(
        headers=[
            "id",
            "name",
            "score",
            "age",
            "active",
            "id_with_nulls",
            "score_with_nulls",
        ],
        columns=[
            Column.integer([1, 2, 3, 4]),
            Column.string(["ann", "clark", "lana", "lex"]),
            Column.floating([20.3, 2.1, 3.9, 40.0]),
            Column.integer([20, None, 30, 40]),
            Column.boolean([True, False, True, False]),
            Column.integer([None, 2, None, 4]),
            Column.floating([1.5, None, 3.5, None]),
        ],
    )


# ── Column construction ───────────────────────────────────────────────────────
# NOTE: column ops commented out — see lib.rs for rationale.

# class TestColumnConstruction: ...
# class TestColumnBasicOps: ...
# class TestNumericOps: ...
# class TestComparisonOps: ...
# class TestStringOps: ...


# ── Dataset ───────────────────────────────────────────────────────────────────


class TestDataset:
    def test_shape(self, dataset):
        assert dataset.shape() == (4, 7)

    def test_get_column_by_name(self, dataset):
        col = dataset.get_column_by_name("id")
        assert col is not None

    def test_get_column_by_name_missing(self, dataset):
        assert dataset.get_column_by_name("nonexistent") is None

    def test_get_column_by_index(self, dataset):
        col = dataset.get_column_by_index(0)
        assert col is not None

    def test_get_column_index(self, dataset):
        assert dataset.get_column_index("id") == 0
        assert dataset.get_column_index("name") == 1
        assert dataset.get_column_index("score_with_nulls") == 6

    def test_get_column_index_missing(self, dataset):
        assert dataset.get_column_index("nonexistent") is None


# ── Validation ────────────────────────────────────────────────────────────────


class TestValidation:
    def test_passing_rule(self, dataset):
        report = py_validate_columns(
            dataset, ColumnRuleBuilder("id").not_null().build()
        )
        assert len(report.results) == 1
        assert report.results[0].is_passed

    def test_failing_rule(self, dataset):
        report = py_validate_columns(
            dataset, ColumnRuleBuilder("age").not_null().build()
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 1

    def test_multiple_rules(self, dataset):
        rules = [
            *ColumnRuleBuilder("id").not_null().unique().build(),
            *ColumnRuleBuilder("age").not_null().build(),
        ]
        report = py_validate_columns(dataset, rules)
        assert report.results[0].is_passed
        assert report.results[1].is_passed
        assert not report.results[2].is_passed

    def test_missing_column(self, dataset):
        report = py_validate_columns(
            dataset, ColumnRuleBuilder("nonexistent").not_null().build()
        )
        assert not report.results[0].is_passed

    def test_result_fields(self, dataset):
        report = py_validate_columns(
            dataset, ColumnRuleBuilder("age").not_null().build()
        )
        r = report.results[0]
        assert r.column == "age"
        assert r.constraint is not None
        assert r.failed_count == 1
        assert r.error is not None

    def test_all_constraints(self, dataset):
        rules = [
            *ColumnRuleBuilder("id")
            .not_null()
            .unique()
            .gt(0.0)
            .ge(1.0)
            .lt(10.0)
            .le(4.0)
            .between(1.0, 4.0)
            .build(),
            *ColumnRuleBuilder("name")
            .contains("a")
            .starts_with("a")
            .ends_with("x")
            .matches_regex("^[a-z]+")
            .length_between(2, 10)
            .is_in(["ann", "clark", "lana", "lex"])
            .build(),
        ]
        report = py_validate_columns(dataset, rules)
        assert len(report.results) == 13

    def test_with_nulls_column(self, dataset):
        report = py_validate_columns(
            dataset, ColumnRuleBuilder("id_with_nulls").not_null().build()
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 2

    def test_report_all_pass(self, dataset):
        rules = [*ColumnRuleBuilder("id").not_null().unique().build()]
        report = py_validate_columns(dataset, rules)
        assert report.passed is True
        assert report.total_rules == 2
        assert report.passed_count == 2
        assert report.failed_count == 0

    def test_report_partial_fail(self, dataset):
        rules = [
            *ColumnRuleBuilder("id").not_null().unique().build(),
            *ColumnRuleBuilder("age").not_null().build(),
        ]
        report = py_validate_columns(dataset, rules)
        assert report.passed is False
        assert report.total_rules == 3
        assert report.passed_count == 2
        assert report.failed_count == 1

    def test_report_failed_values(self, dataset):
        report = py_validate_columns(
            dataset, ColumnRuleBuilder("age").not_null().build()
        )
        r = report.results[0]
        assert r.failed_values is not None
        assert len(r.failed_values) == 1
        idx, val = r.failed_values[0]
        assert isinstance(idx, int)
        assert isinstance(val, str)


# ── CSV loading ───────────────────────────────────────────────────────────────


class TestCsvLoading:
    def test_from_csv(self, csv_path):
        schema = Schema(
            [
                ("id", DataType.integer()),
                ("name", DataType.string()),
                ("score", DataType.float()),
                ("age", DataType.integer()),
                ("active", DataType.boolean()),
            ]
        )
        ds = Dataset.from_csv(csv_path, schema)
        assert ds.shape() == (4, 5)

    def test_from_csv_invalid_type(self, tmp_path):
        csv = tmp_path / "bad.csv"
        csv.write_text("id\nnot_a_number\n")
        schema = Schema([("id", DataType.integer())])
        with pytest.raises(ValueError):
            Dataset.from_csv(str(csv), schema)

    def test_from_csv_schema_too_few_columns(self, csv_path):
        schema = Schema(
            [
                ("id", DataType.integer()),
                ("name", DataType.string()),
            ]
        )
        with pytest.raises(ValueError):
            Dataset.from_csv(csv_path, schema)

    def test_from_csv_schema_too_many_columns(self, csv_path):
        schema = Schema(
            [
                ("id", DataType.integer()),
                ("name", DataType.string()),
                ("score", DataType.float()),
                ("age", DataType.integer()),
                ("active", DataType.boolean()),
                ("extra", DataType.integer()),
            ]
        )
        with pytest.raises(ValueError):
            Dataset.from_csv(csv_path, schema)

    def test_from_csv_and_validate(self, csv_path):
        schema = Schema(
            [
                ("id", DataType.integer()),
                ("name", DataType.string()),
                ("score", DataType.float()),
                ("age", DataType.integer()),
                ("active", DataType.boolean()),
            ]
        )
        ds = Dataset.from_csv(csv_path, schema)
        rules = [
            *ColumnRuleBuilder("id").not_null().unique().build(),
            *ColumnRuleBuilder("score").between(0.0, 10.0).build(),
        ]
        report = py_validate_columns(ds, rules)
        assert all(r.is_passed for r in report.results)


# ── Column pair validation ────────────────────────────────────────────────────


class TestColumnPairValidation:
    # ── gt ───────────────────────────────────────────────────────────────────

    def test_gt_passes(self, compare_dataset):
        # y=[6..10] > x=[1..5] — always true
        report = py_validate_columns(
            compare_dataset, ColumnRuleBuilder("y").gt(col("x")).build()
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_gt_fails(self, compare_dataset):
        # x=[1,2,3,4,5] > z=[28,1,0.5,4,0.9]: row 0: 1>28 false, row 3: 4>4 false → 2 failures
        report = py_validate_columns(
            compare_dataset, ColumnRuleBuilder("x").gt(col("z")).build()
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 2

    # ── ge ───────────────────────────────────────────────────────────────────

    def test_ge_passes(self, compare_dataset):
        # y >= x always
        report = py_validate_columns(
            compare_dataset, ColumnRuleBuilder("y").ge(col("x")).build()
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_ge_equal_values_pass(self, compare_dataset):
        # x=[1,2,3,4,5] >= z=[28,1,0.5,4,0.9]: row 3: 4>=4 true; row 0: 1>=28 false → 1 failure
        report = py_validate_columns(
            compare_dataset, ColumnRuleBuilder("x").ge(col("z")).build()
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 1

    # ── lt ───────────────────────────────────────────────────────────────────

    def test_lt_passes(self, compare_dataset):
        # x < y always
        report = py_validate_columns(
            compare_dataset, ColumnRuleBuilder("x").lt(col("y")).build()
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_lt_fails(self, compare_dataset):
        # z=[28,1,0.5,4,0.9] < x=[1,2,3,4,5]: row 0: 28<1 false, row 3: 4<4 false → 2 failures
        report = py_validate_columns(
            compare_dataset, ColumnRuleBuilder("z").lt(col("x")).build()
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 2

    # ── le ───────────────────────────────────────────────────────────────────

    def test_le_passes(self, compare_dataset):
        # x <= y always
        report = py_validate_columns(
            compare_dataset, ColumnRuleBuilder("x").le(col("y")).build()
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_le_fails(self, compare_dataset):
        # x=[1,2,3,4,5] <= z=[28,1,0.5,4,0.9]: rows 1,2,4 fail → 3 failures
        report = py_validate_columns(
            compare_dataset, ColumnRuleBuilder("x").le(col("z")).build()
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 3

    # ── equal ─────────────────────────────────────────────────────────────────

    def test_equal_same_column(self, compare_dataset):
        # x == x: every value equals itself
        report = py_validate_columns(
            compare_dataset, ColumnRuleBuilder("x").equal(col("x")).build()
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_equal_fails(self, compare_dataset):
        # x=[1..5] != y=[6..10] for every row → 5 failures
        report = py_validate_columns(
            compare_dataset, ColumnRuleBuilder("x").equal(col("y")).build()
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 5

    # ── between ───────────────────────────────────────────────────────────────

    # NOTE: mixed literal+col between (e.g. between(0.0, col("y"))) not yet supported in core.
    # def test_between_literal_col_passes: ...
    # def test_between_col_literal_passes: ...

    def test_between_col_col_fails(self, compare_dataset):
        # z between x and y: rows 0,1,2,4 fail → 4 failures
        report = py_validate_columns(
            compare_dataset, ColumnRuleBuilder("z").between(col("x"), col("y")).build()
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 4

    # ── nulls ─────────────────────────────────────────────────────────────────

    def test_null_skipped_in_col_comparison(self, compare_nulls_dataset):
        # a < b: rows 0,3 non-null pass; rows 1,2,4 have nulls → skipped → passes
        report = py_validate_columns(
            compare_nulls_dataset, ColumnRuleBuilder("a").lt(col("b")).build()
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_one_sided_null_skipped(self, compare_nulls_dataset):
        # a < high: high has no nulls; a is null at rows 1,4 → skipped → passes
        report = py_validate_columns(
            compare_nulls_dataset, ColumnRuleBuilder("a").lt(col("high")).build()
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_both_null_skipped(self, compare_nulls_dataset):
        # a == c: same values/nulls; rows 1,4 both null → skipped → passes
        report = py_validate_columns(
            compare_nulls_dataset, ColumnRuleBuilder("a").equal(col("c")).build()
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    # NOTE: mixed literal+col between not yet supported in core.
    # def test_between_with_nulls: ...

    # ── str ───────────────────────────────────────────────────────────────────

    def test_str_equal_null_skipped(self):
        ds = Dataset(
            headers=["a", "b"],
            columns=[
                Column.string(["foo", "bar", None]),
                Column.string(["foo", "bar", None]),
            ],
        )
        # rows 0,1 match; row 2 both null → skipped → passes
        report = py_validate_columns(ds, ColumnRuleBuilder("a").equal(col("b")).build())
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_str_lt_passes(self):
        ds = Dataset(
            headers=["a", "b"],
            columns=[
                Column.string(["apple", "cat"]),
                Column.string(["banana", "dog"]),
            ],
        )
        # "apple" < "banana", "cat" < "dog" lexicographically
        report = py_validate_columns(ds, ColumnRuleBuilder("a").lt(col("b")).build())
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_str_lt_fails(self):
        ds = Dataset(
            headers=["a", "b"],
            columns=[
                Column.string(["zoo", "cat"]),
                Column.string(["apple", "dog"]),
            ],
        )
        # "zoo" < "apple" false → 1 failure
        report = py_validate_columns(ds, ColumnRuleBuilder("a").lt(col("b")).build())
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 1

    # ── bool ──────────────────────────────────────────────────────────────────

    def test_bool_equal_null_skipped(self):
        ds = Dataset(
            headers=["a", "b"],
            columns=[
                Column.boolean([True, False, None]),
                Column.boolean([True, False, None]),
            ],
        )
        # rows 0,1 match; row 2 both null → skipped → passes
        report = py_validate_columns(ds, ColumnRuleBuilder("a").equal(col("b")).build())
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_bool_gt(self):
        ds = Dataset(
            headers=["a", "b"],
            columns=[
                Column.boolean([True, False]),
                Column.boolean([False, True]),
            ],
        )
        # true>false passes, false>true fails → 1 failure
        report = py_validate_columns(ds, ColumnRuleBuilder("a").gt(col("b")).build())
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 1

    # ── edge cases ────────────────────────────────────────────────────────────

    def test_type_mismatch_all_skipped(self, compare_dataset):
        # id (Int) vs x (Float) → type mismatch → all None → all skipped → passes
        report = py_validate_columns(
            compare_dataset, ColumnRuleBuilder("id").gt(col("x")).build()
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_all_null_left_all_skipped(self):
        ds = Dataset(
            headers=["a", "b"],
            columns=[
                Column.floating([None, None, None]),
                Column.floating([1.0, 2.0, 3.0]),
            ],
        )
        report = py_validate_columns(ds, ColumnRuleBuilder("a").lt("b").build())
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_between_type_mismatch_all_skipped(self, compare_dataset):
        # id (Int) between x (Float) and y (Float) → type mismatch → all None → all skipped → passes
        report = py_validate_columns(
            compare_dataset, ColumnRuleBuilder("id").between(col("x"), col("y")).build()
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_missing_column_error(self, compare_dataset):
        report = py_validate_columns(
            compare_dataset, ColumnRuleBuilder("x").gt(col("nonexistent")).build()
        )
        assert not report.results[0].is_passed
        assert report.results[0].error is not None


# ── Date and DateTime constraints ─────────────────────────────────────────────
#
# Epoch days (i32) since 1970-01-01:
#   18262 = 2020-01-01
#   19358 = 2023-01-01
#   19723 = 2024-01-01
#   20089 = 2025-01-01
#
# Epoch microseconds (i64) since 1970-01-01T00:00:00:
#   1577836800000000 = 2020-01-01T00:00:00
#   1672531200000000 = 2023-01-01T00:00:00
#   1704067200000000 = 2024-01-01T00:00:00
#   1735689600000000 = 2025-01-01T00:00:00


class TestDateConstraints:
    @pytest.fixture
    def date_ds(self):
        # rows: 2020-01-01, 2023-01-01, 2024-01-01, null
        return Dataset(
            headers=["d"],
            columns=[Column.date([18262, 19358, 19723, None])],
        )

    def test_after_passes(self, date_ds):
        report = py_validate_columns(
            date_ds, [ColumnRule("d", ColumnConstraint.after("2019-12-31"))]
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_after_fails(self, date_ds):
        # threshold 2023-06-01 — rows 0 (2020) and 1 (2023-01) fail, row 3 null skipped
        report = py_validate_columns(
            date_ds, [ColumnRule("d", ColumnConstraint.after("2023-06-01"))]
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 2

    def test_after_null_skipped(self, date_ds):
        # null row does not count as failure
        report = py_validate_columns(
            date_ds, [ColumnRule("d", ColumnConstraint.after("2019-12-31"))]
        )
        assert report.results[0].failed_count == 0

    def test_before_passes(self, date_ds):
        report = py_validate_columns(
            date_ds, [ColumnRule("d", ColumnConstraint.before("2025-01-01"))]
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_before_fails(self, date_ds):
        # threshold 2022-01-01 — rows 1 (2023) and 2 (2024) fail
        report = py_validate_columns(
            date_ds, [ColumnRule("d", ColumnConstraint.before("2022-01-01"))]
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 2

    def test_between_dates_passes(self, date_ds):
        report = py_validate_columns(
            date_ds,
            [
                ColumnRule(
                    "d", ColumnConstraint.between_dates("2019-12-31", "2025-01-01")
                )
            ],
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_between_dates_fails(self, date_ds):
        # only 2023-01-01 and 2024-01-01 are in range [2022-06-01, 2024-06-01]
        report = py_validate_columns(
            date_ds,
            [
                ColumnRule(
                    "d", ColumnConstraint.between_dates("2022-06-01", "2024-06-01")
                )
            ],
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 1  # 2020-01-01 fails

    def test_after_boundary_exclusive(self):
        # after("2024-01-01") — row with exactly 2024-01-01 should fail (exclusive)
        ds = Dataset(headers=["d"], columns=[Column.date([19723])])
        report = py_validate_columns(
            ds, [ColumnRule("d", ColumnConstraint.after("2024-01-01"))]
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 1

    def test_before_boundary_exclusive(self):
        # before("2024-01-01") — row with exactly 2024-01-01 should fail (exclusive)
        ds = Dataset(headers=["d"], columns=[Column.date([19723])])
        report = py_validate_columns(
            ds, [ColumnRule("d", ColumnConstraint.before("2024-01-01"))]
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 1

    def test_failed_values_reported(self, date_ds):
        report = py_validate_columns(
            date_ds, [ColumnRule("d", ColumnConstraint.after("2023-06-01"))]
        )
        r = report.results[0]
        assert r.failed_values is not None
        assert len(r.failed_values) == 2
        # values should be human-readable date strings, not raw integers
        idxs = [idx for idx, _ in r.failed_values]
        vals = [v for _, v in r.failed_values]
        assert 0 in idxs
        assert 1 in idxs
        assert all("-" in v for v in vals)


class TestDateTimeConstraints:
    @pytest.fixture
    def dt_ds(self):
        # rows: 2020-01-01, 2023-01-01, 2024-01-01, null
        return Dataset(
            headers=["ts"],
            columns=[
                Column.datetime(
                    [1577836800000000, 1672531200000000, 1704067200000000, None]
                )
            ],
        )

    def test_after_passes(self, dt_ds):
        report = py_validate_columns(
            dt_ds, [ColumnRule("ts", ColumnConstraint.after("2019-12-31T00:00:00"))]
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_after_fails(self, dt_ds):
        # threshold 2023-06-01 — rows 0 (2020) and 1 (2023-01) fail
        report = py_validate_columns(
            dt_ds, [ColumnRule("ts", ColumnConstraint.after("2023-06-01T00:00:00"))]
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 2

    def test_after_null_skipped(self, dt_ds):
        report = py_validate_columns(
            dt_ds, [ColumnRule("ts", ColumnConstraint.after("2019-12-31T00:00:00"))]
        )
        assert report.results[0].failed_count == 0

    def test_before_passes(self, dt_ds):
        report = py_validate_columns(
            dt_ds, [ColumnRule("ts", ColumnConstraint.before("2025-01-01T00:00:00"))]
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_before_fails(self, dt_ds):
        # threshold 2022-01-01 — rows 1 and 2 fail
        report = py_validate_columns(
            dt_ds, [ColumnRule("ts", ColumnConstraint.before("2022-01-01T00:00:00"))]
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 2

    def test_between_dates_passes(self, dt_ds):
        report = py_validate_columns(
            dt_ds,
            [
                ColumnRule(
                    "ts",
                    ColumnConstraint.between_dates(
                        "2019-12-31T00:00:00", "2025-01-01T00:00:00"
                    ),
                )
            ],
        )
        assert report.results[0].is_passed
        assert report.results[0].failed_count == 0

    def test_between_dates_fails(self, dt_ds):
        report = py_validate_columns(
            dt_ds,
            [
                ColumnRule(
                    "ts",
                    ColumnConstraint.between_dates(
                        "2022-06-01T00:00:00", "2024-06-01T00:00:00"
                    ),
                )
            ],
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 1  # 2020-01-01 fails

    def test_after_boundary_exclusive(self):
        ds = Dataset(headers=["ts"], columns=[Column.datetime([1704067200000000])])
        report = py_validate_columns(
            ds, [ColumnRule("ts", ColumnConstraint.after("2024-01-01T00:00:00"))]
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 1

    def test_before_boundary_exclusive(self):
        ds = Dataset(headers=["ts"], columns=[Column.datetime([1704067200000000])])
        report = py_validate_columns(
            ds, [ColumnRule("ts", ColumnConstraint.before("2024-01-01T00:00:00"))]
        )
        assert not report.results[0].is_passed
        assert report.results[0].failed_count == 1

    def test_failed_values_reported(self, dt_ds):
        report = py_validate_columns(
            dt_ds, [ColumnRule("ts", ColumnConstraint.after("2023-06-01T00:00:00"))]
        )
        r = report.results[0]
        assert r.failed_values is not None
        assert len(r.failed_values) == 2
        vals = [v for _, v in r.failed_values]
        assert all("-" in v for v in vals)


class TestDateFromCsv:
    DATE_CSV = """\
created_date,created_at
2020-06-11,2023-09-04T14:18:42
2023-11-14,2024-05-29T21:21:34
,
"""

    @pytest.fixture
    def csv_path(self, tmp_path):
        p = tmp_path / "dates.csv"
        p.write_text(self.DATE_CSV)
        return str(p)

    def test_load_date_datetime_columns(self, csv_path):
        schema = Schema(
            [
                ("created_date", DataType.date()),
                ("created_at", DataType.datetime()),
            ]
        )
        ds = Dataset.from_csv(csv_path, schema)
        assert ds.shape() == (3, 2)

    def test_validate_after_from_csv(self, csv_path):
        schema = Schema(
            [
                ("created_date", DataType.date()),
                ("created_at", DataType.datetime()),
            ]
        )
        ds = Dataset.from_csv(csv_path, schema)
        rules = [ColumnRule("created_date", ColumnConstraint.after("2019-12-31"))]
        report = py_validate_columns(ds, rules)
        assert report.results[0].is_passed

    def test_validate_before_from_csv(self, csv_path):
        schema = Schema(
            [
                ("created_date", DataType.date()),
                ("created_at", DataType.datetime()),
            ]
        )
        ds = Dataset.from_csv(csv_path, schema)
        rules = [
            ColumnRule("created_at", ColumnConstraint.before("2025-01-01T00:00:00"))
        ]
        report = py_validate_columns(ds, rules)
        assert report.results[0].is_passed

    def test_invalid_date_format_raises(self, tmp_path):
        bad = tmp_path / "bad_dates.csv"
        bad.write_text("created_date\nnot-a-date\n")
        schema = Schema([("created_date", DataType.date())])
        with pytest.raises(ValueError):
            Dataset.from_csv(str(bad), schema)


# ── Table constraints ─────────────────────────────────────────────────────────


class TestTableConstraints:
    @pytest.fixture
    def ds(self):
        return Dataset(
            headers=["id", "name", "score"],
            columns=[
                Column.integer([1, 2, 3]),
                Column.string(["a", "b", "c"]),
                Column.floating([1.0, 2.0, 3.0]),
            ],
        )

    def test_rows_count_between_passes(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.rows_count_between(1, 5))])
        assert report.results[0].is_passed

    def test_rows_count_between_fails(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.rows_count_between(10, 20))])
        assert not report.results[0].is_passed

    def test_rows_count_ge_passes(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.rows_count_ge(3))])
        assert report.results[0].is_passed

    def test_rows_count_ge_fails(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.rows_count_ge(4))])
        assert not report.results[0].is_passed

    def test_rows_count_gt_passes(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.rows_count_gt(2))])
        assert report.results[0].is_passed

    def test_rows_count_gt_fails_equal(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.rows_count_gt(3))])
        assert not report.results[0].is_passed

    def test_rows_count_le_passes(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.rows_count_le(3))])
        assert report.results[0].is_passed

    def test_rows_count_le_fails(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.rows_count_le(2))])
        assert not report.results[0].is_passed

    def test_rows_count_lt_passes(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.rows_count_lt(4))])
        assert report.results[0].is_passed

    def test_rows_count_lt_fails_equal(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.rows_count_lt(3))])
        assert not report.results[0].is_passed

    def test_columns_count_between_passes(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.columns_count_between(2, 5))])
        assert report.results[0].is_passed

    def test_columns_count_between_fails(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.columns_count_between(5, 10))])
        assert not report.results[0].is_passed

    def test_columns_count_ge_passes(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.columns_count_ge(3))])
        assert report.results[0].is_passed

    def test_columns_count_ge_fails(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.columns_count_ge(4))])
        assert not report.results[0].is_passed

    def test_columns_exist_all_present_passes(self, ds):
        report = py_validate_table(
            ds, [TableRule(TableConstraint.columns_exist(["id", "name", "score"]))]
        )
        assert report.results[0].is_passed

    def test_columns_exist_missing_fails_with_error_message(self, ds):
        report = py_validate_table(
            ds, [TableRule(TableConstraint.columns_exist(["id", "ghost"]))]
        )
        assert not report.results[0].is_passed
        assert report.results[0].error is not None
        assert "ghost" in report.results[0].error

    def test_shape_equals_passes(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.shape_equals(3, 3))])
        assert report.results[0].is_passed

    def test_shape_equals_fails_wrong_rows(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.shape_equals(99, 3))])
        assert not report.results[0].is_passed

    def test_multiple_rules_report_count(self, ds):
        rules = [
            TableRule(TableConstraint.rows_count_ge(1)),
            TableRule(TableConstraint.columns_count_ge(1)),
            TableRule(TableConstraint.shape_equals(3, 3)),
        ]
        report = py_validate_table(ds, rules)
        assert len(report.results) == 3

    def test_validate_table_result_has_no_column(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.rows_count_ge(1))])
        assert report.results[0].column is None

    def test_validate_table_result_has_no_failed_count(self, ds):
        report = py_validate_table(ds, [TableRule(TableConstraint.rows_count_ge(1))])
        assert report.results[0].failed_count is None
