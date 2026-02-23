import pytest
from verdict_py import Dataset, Column, RuleBuilder, Schema, DataType, py_validate


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
        headers=["id", "name", "score", "age", "active", "id_with_nulls", "score_with_nulls"],
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

class TestColumnConstruction:
    def test_integer(self):
        col = Column.integer([1, 2, None, 4])
        assert col.len() == 4
        assert col.null_count() == 1

    def test_floating(self):
        col = Column.floating([1.5, None, 3.0])
        assert col.len() == 3
        assert col.null_count() == 1

    def test_string(self):
        col = Column.string(["a", "b", None])
        assert col.len() == 3
        assert col.null_count() == 1

    def test_boolean(self):
        col = Column.boolean([True, False, None])
        assert col.len() == 3
        assert col.null_count() == 1

    def test_all_nulls(self):
        assert Column.integer([None, None]).null_count() == 2
        assert Column.floating([None, None]).null_count() == 2
        assert Column.string([None, None]).null_count() == 2
        assert Column.boolean([None, None]).null_count() == 2

    def test_empty(self):
        assert Column.integer([]).is_empty()
        assert Column.floating([]).is_empty()
        assert Column.string([]).is_empty()
        assert Column.boolean([]).is_empty()


# ── Column basic ops ──────────────────────────────────────────────────────────

class TestColumnBasicOps:
    def test_len(self, dataset):
        assert dataset.get_column_by_name("id").len() == 4

    def test_is_empty(self):
        assert Column.integer([]).is_empty()
        assert not Column.integer([1]).is_empty()

    def test_null_count(self, dataset):
        age = dataset.get_column_by_name("age")
        assert age.null_count() == 1
        assert age.not_null_count() == 3

    def test_null_count_with_nulls_column(self, dataset):
        col = dataset.get_column_by_name("id_with_nulls")
        assert col.null_count() == 2
        assert col.not_null_count() == 2

    def test_is_null(self):
        col = Column.integer([1, None, 3])
        assert col.is_null() == [False, True, False]

    def test_unique_count(self):
        col = Column.integer([1, 1, 2, 3])
        assert col.unique_count() == 3

    def test_duplicates_count(self):
        col = Column.integer([1, 1, 2, 3])
        assert col.duplicates_count() == 1

    def test_boolean_basic_ops(self, dataset):
        col = dataset.get_column_by_name("active")
        assert col.len() == 4
        assert col.null_count() == 0
        assert col.not_null_count() == 4


# ── Numeric ops ───────────────────────────────────────────────────────────────

class TestNumericOps:
    def test_sum_int(self, dataset):
        col = dataset.get_column_by_name("id")
        assert col.sum() == 10.0

    def test_sum_with_nulls(self, dataset):
        col = dataset.get_column_by_name("age")
        assert col.sum() == 90.0

    def test_mean(self, dataset):
        col = dataset.get_column_by_name("id")
        assert col.mean() == 2.5

    def test_min_max_int(self, dataset):
        col = dataset.get_column_by_name("id")
        assert col.min() == 1.0
        assert col.max() == 4.0

    def test_min_max_float(self, dataset):
        col = dataset.get_column_by_name("score")
        assert col.min() == pytest.approx(2.1)
        assert col.max() == pytest.approx(40.0)

    def test_returns_none_for_all_nulls(self):
        col = Column.integer([None, None])
        assert col.sum() is None
        assert col.mean() is None
        assert col.min() is None
        assert col.max() is None

    def test_float_with_nulls(self, dataset):
        col = dataset.get_column_by_name("score_with_nulls")
        assert col.sum() == pytest.approx(5.0)
        assert col.null_count() == 2


# ── Comparison ops ────────────────────────────────────────────────────────────

class TestComparisonOps:
    def test_gt(self):
        col = Column.integer([1, 2, 3])
        assert col.gt(2.0) == [False, False, True]

    def test_ge(self):
        col = Column.integer([1, 2, 3])
        assert col.ge(2.0) == [False, True, True]

    def test_lt(self):
        col = Column.integer([1, 2, 3])
        assert col.lt(2.0) == [True, False, False]

    def test_le(self):
        col = Column.integer([1, 2, 3])
        assert col.le(2.0) == [True, True, False]

    def test_equal_numeric(self):
        col = Column.integer([1, 2, 3])
        assert col.equal(2.0) == [False, True, False]

    def test_between(self):
        col = Column.integer([1, 2, 3, 4])
        assert col.between(2.0, 3.0) == [False, True, True, False]

    def test_null_propagation(self):
        col = Column.integer([1, None, 3])
        assert col.gt(0.0) == [True, None, True]

    def test_float_comparison(self, dataset):
        col = dataset.get_column_by_name("score")
        assert col.gt(10.0) == [True, False, False, True]


# ── String ops ────────────────────────────────────────────────────────────────

class TestStringOps:
    def test_equal_string(self):
        col = Column.string(["ann", "clark", "lana"])
        assert col.equal("clark") == [False, True, False]

    def test_contains(self):
        col = Column.string(["ann", "clark", "lana"])
        assert col.contains("an") == [True, False, True]

    def test_starts_with(self):
        col = Column.string(["ann", "clark", "lana"])
        assert col.starts_with("a") == [True, False, False]

    def test_ends_with(self):
        col = Column.string(["ann", "clark", "lana"])
        assert col.ends_with("a") == [False, False, True]

    def test_matches_regex(self):
        col = Column.string(["ann", "clark", "123"])
        assert col.matches_regex("^[a-z]+$") == [True, True, False]

    def test_str_length(self):
        col = Column.string(["hi", "hello", None])
        assert col.str_length() == [2, 5, None]

    def test_is_in_strings(self):
        col = Column.string(["ann", "clark", "lana"])
        assert col.is_in(["ann", "lana"]) == [True, False, True]

    def test_is_in_integers(self):
        col = Column.integer([1, 2, 3, 4])
        assert col.is_in([1, 3]) == [True, False, True, False]

    def test_is_in_floats(self):
        col = Column.floating([1.5, 2.5, 3.5])
        assert col.is_in([1.5, 3.5]) == [True, False, True]

    def test_null_in_string_ops(self):
        col = Column.string(["ann", None, "lana"])
        assert col.contains("an") == [True, None, True]


# ── Dataset ───────────────────────────────────────────────────────────────────

class TestDataset:
    def test_shape(self, dataset):
        assert dataset.shape() == (4, 7)

    def test_get_column_by_name(self, dataset):
        col = dataset.get_column_by_name("id")
        assert col is not None
        assert col.len() == 4

    def test_get_column_by_name_missing(self, dataset):
        assert dataset.get_column_by_name("nonexistent") is None

    def test_get_column_by_index(self, dataset):
        col = dataset.get_column_by_index(0)
        assert col is not None
        assert col.len() == 4

    def test_get_column_index(self, dataset):
        assert dataset.get_column_index("id") == 0
        assert dataset.get_column_index("name") == 1
        assert dataset.get_column_index("score_with_nulls") == 6

    def test_get_column_index_missing(self, dataset):
        assert dataset.get_column_index("nonexistent") is None


# ── Validation ────────────────────────────────────────────────────────────────

class TestValidation:
    def test_passing_rule(self, dataset):
        results = py_validate(dataset, RuleBuilder("id").not_null().build())
        assert len(results) == 1
        assert results[0].is_passed

    def test_failing_rule(self, dataset):
        results = py_validate(dataset, RuleBuilder("age").not_null().build())
        assert not results[0].is_passed
        assert results[0].failed_count == 1

    def test_multiple_rules(self, dataset):
        rules = [
            *RuleBuilder("id").not_null().unique().build(),
            *RuleBuilder("age").not_null().build(),
        ]
        results = py_validate(dataset, rules)
        assert results[0].is_passed
        assert results[1].is_passed
        assert not results[2].is_passed

    def test_missing_column(self, dataset):
        results = py_validate(dataset, RuleBuilder("nonexistent").not_null().build())
        assert not results[0].is_passed

    def test_result_fields(self, dataset):
        results = py_validate(dataset, RuleBuilder("age").not_null().build())
        r = results[0]
        assert r.column == "age"
        assert r.constraint is not None
        assert r.failed_count == 1
        assert r.error is not None

    def test_all_constraints(self, dataset):
        rules = [
            *RuleBuilder("id").not_null().unique().gt(0.0).ge(1.0).lt(10.0).le(4.0).between(1.0, 4.0).build(),
            *RuleBuilder("name").contains("a").starts_with("a").ends_with("x").matches_regex("^[a-z]+").length_between(2, 10).is_in(["ann", "clark", "lana", "lex"]).build(),
        ]
        results = py_validate(dataset, rules)
        assert len(results) == 13

    def test_with_nulls_column(self, dataset):
        results = py_validate(dataset, RuleBuilder("id_with_nulls").not_null().build())
        assert not results[0].is_passed
        assert results[0].failed_count == 2


# ── CSV loading ───────────────────────────────────────────────────────────────

class TestCsvLoading:
    def test_from_csv(self, csv_path):
        schema = Schema([
            ("id", DataType.integer()),
            ("name", DataType.string()),
            ("score", DataType.float()),
            ("age", DataType.integer()),
            ("active", DataType.boolean()),
        ])
        ds = Dataset.from_csv(csv_path, schema)
        assert ds.shape() == (4, 5)
        assert ds.get_column_by_name("id").null_count() == 0
        assert ds.get_column_by_name("name").null_count() == 1
        assert ds.get_column_by_name("age").null_count() == 1

    def test_from_csv_invalid_type(self, tmp_path):
        csv = tmp_path / "bad.csv"
        csv.write_text("id\nnot_a_number\n")
        schema = Schema([("id", DataType.integer())])
        with pytest.raises(ValueError):
            Dataset.from_csv(str(csv), schema)

    def test_from_csv_and_validate(self, csv_path):
        schema = Schema([
            ("id", DataType.integer()),
            ("name", DataType.string()),
            ("score", DataType.float()),
            ("age", DataType.integer()),
            ("active", DataType.boolean()),
        ])
        ds = Dataset.from_csv(csv_path, schema)
        rules = [
            *RuleBuilder("id").not_null().unique().build(),
            *RuleBuilder("score").between(0.0, 10.0).build(),
        ]
        results = py_validate(ds, rules)
        assert all(r.is_passed for r in results)


# ── Column pair validation ────────────────────────────────────────────────────

class TestColumnPairValidation:

    # ── gt ───────────────────────────────────────────────────────────────────

    def test_gt_passes(self, compare_dataset):
        # y=[6..10] > x=[1..5] — always true
        results = py_validate(compare_dataset, RuleBuilder("y").gt("x").build())
        assert results[0].is_passed
        assert results[0].failed_count == 0

    def test_gt_fails(self, compare_dataset):
        # x=[1,2,3,4,5] > z=[28,1,0.5,4,0.9]: row 0: 1>28 false, row 3: 4>4 false → 2 failures
        results = py_validate(compare_dataset, RuleBuilder("x").gt("z").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 2

    # ── ge ───────────────────────────────────────────────────────────────────

    def test_ge_passes(self, compare_dataset):
        # y >= x always
        results = py_validate(compare_dataset, RuleBuilder("y").ge("x").build())
        assert results[0].is_passed
        assert results[0].failed_count == 0

    def test_ge_equal_values_pass(self, compare_dataset):
        # x=[1,2,3,4,5] >= z=[28,1,0.5,4,0.9]: row 3: 4>=4 true; row 0: 1>=28 false → 1 failure
        results = py_validate(compare_dataset, RuleBuilder("x").ge("z").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 1

    # ── lt ───────────────────────────────────────────────────────────────────

    def test_lt_passes(self, compare_dataset):
        # x < y always
        results = py_validate(compare_dataset, RuleBuilder("x").lt("y").build())
        assert results[0].is_passed
        assert results[0].failed_count == 0

    def test_lt_fails(self, compare_dataset):
        # z=[28,1,0.5,4,0.9] < x=[1,2,3,4,5]: row 0: 28<1 false, row 3: 4<4 false → 2 failures
        results = py_validate(compare_dataset, RuleBuilder("z").lt("x").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 2

    # ── le ───────────────────────────────────────────────────────────────────

    def test_le_passes(self, compare_dataset):
        # x <= y always
        results = py_validate(compare_dataset, RuleBuilder("x").le("y").build())
        assert results[0].is_passed
        assert results[0].failed_count == 0

    def test_le_fails(self, compare_dataset):
        # x=[1,2,3,4,5] <= z=[28,1,0.5,4,0.9]: rows 1,2,4 fail → 3 failures
        results = py_validate(compare_dataset, RuleBuilder("x").le("z").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 3

    # ── equal ─────────────────────────────────────────────────────────────────

    def test_equal_same_column(self, compare_dataset):
        # x == x: every value equals itself
        results = py_validate(compare_dataset, RuleBuilder("x").equal("x").build())
        assert results[0].is_passed
        assert results[0].failed_count == 0

    def test_equal_fails(self, compare_dataset):
        # x=[1..5] != y=[6..10] for every row → 5 failures
        results = py_validate(compare_dataset, RuleBuilder("x").equal("y").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 5

    # ── between ───────────────────────────────────────────────────────────────

    def test_between_literal_col_passes(self, compare_dataset):
        # 0.0 <= x <= y: x=[1..5], y=[6..10] — all pass
        results = py_validate(compare_dataset, RuleBuilder("x").between(0.0, "y").build())
        assert results[0].is_passed
        assert results[0].failed_count == 0

    def test_between_col_literal_passes(self, compare_dataset):
        # x <= y <= 100.0: y=[6..10], x=[1..5] — all pass
        results = py_validate(compare_dataset, RuleBuilder("y").between("x", 100.0).build())
        assert results[0].is_passed
        assert results[0].failed_count == 0

    def test_between_col_col_fails(self, compare_dataset):
        # z between x and y: rows 0,1,2,4 fail → 4 failures
        results = py_validate(compare_dataset, RuleBuilder("z").between("x", "y").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 4

    # ── nulls ─────────────────────────────────────────────────────────────────

    def test_null_counts_as_failure(self, compare_nulls_dataset):
        # a < b: rows 0,3 pass; rows 1,2,4 have at least one null → 3 failures
        results = py_validate(compare_nulls_dataset, RuleBuilder("a").lt("b").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 3

    def test_one_sided_null_is_failure(self, compare_nulls_dataset):
        # a < high: high has no nulls; a is null at rows 1,4 → 2 failures
        results = py_validate(compare_nulls_dataset, RuleBuilder("a").lt("high").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 2

    def test_both_null_is_failure(self, compare_nulls_dataset):
        # a == c: same values/nulls; rows 1,4 both null → 2 failures
        results = py_validate(compare_nulls_dataset, RuleBuilder("a").equal("c").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 2

    def test_between_with_nulls(self, compare_nulls_dataset):
        # 0.0 <= a <= high: a null at rows 1,4 → 2 failures
        results = py_validate(compare_nulls_dataset, RuleBuilder("a").between(0.0, "high").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 2

    # ── str ───────────────────────────────────────────────────────────────────

    def test_str_equal_passes_with_null(self):
        ds = Dataset(
            headers=["a", "b"],
            columns=[
                Column.string(["foo", "bar", None]),
                Column.string(["foo", "bar", None]),
            ],
        )
        # rows 0,1 match; row 2 both null → None → 1 failure
        results = py_validate(ds, RuleBuilder("a").equal("b").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 1

    def test_str_lt_passes(self):
        ds = Dataset(
            headers=["a", "b"],
            columns=[
                Column.string(["apple", "cat"]),
                Column.string(["banana", "dog"]),
            ],
        )
        # "apple" < "banana", "cat" < "dog" lexicographically
        results = py_validate(ds, RuleBuilder("a").lt("b").build())
        assert results[0].is_passed
        assert results[0].failed_count == 0

    def test_str_lt_fails(self):
        ds = Dataset(
            headers=["a", "b"],
            columns=[
                Column.string(["zoo", "cat"]),
                Column.string(["apple", "dog"]),
            ],
        )
        # "zoo" < "apple" false → 1 failure
        results = py_validate(ds, RuleBuilder("a").lt("b").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 1

    # ── bool ──────────────────────────────────────────────────────────────────

    def test_bool_equal_passes_with_null(self):
        ds = Dataset(
            headers=["a", "b"],
            columns=[
                Column.boolean([True, False, None]),
                Column.boolean([True, False, None]),
            ],
        )
        # rows 0,1 match; row 2 both null → None → 1 failure
        results = py_validate(ds, RuleBuilder("a").equal("b").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 1

    def test_bool_gt(self):
        ds = Dataset(
            headers=["a", "b"],
            columns=[
                Column.boolean([True, False]),
                Column.boolean([False, True]),
            ],
        )
        # true>false passes, false>true fails → 1 failure
        results = py_validate(ds, RuleBuilder("a").gt("b").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 1

    # ── edge cases ────────────────────────────────────────────────────────────

    def test_type_mismatch_all_fail(self, compare_dataset):
        # id (Int) vs x (Float) → type mismatch → all None → all fail
        results = py_validate(compare_dataset, RuleBuilder("id").gt("x").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 5

    def test_all_null_left_all_fail(self):
        ds = Dataset(
            headers=["a", "b"],
            columns=[
                Column.floating([None, None, None]),
                Column.floating([1.0, 2.0, 3.0]),
            ],
        )
        results = py_validate(ds, RuleBuilder("a").lt("b").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 3

    def test_between_type_mismatch_all_fail(self, compare_dataset):
        # id (Int) between x (Float) and y (Float) → type mismatch → all None → all fail
        results = py_validate(compare_dataset, RuleBuilder("id").between("x", "y").build())
        assert not results[0].is_passed
        assert results[0].failed_count == 5

    def test_missing_column_error(self, compare_dataset):
        results = py_validate(compare_dataset, RuleBuilder("x").gt("nonexistent").build())
        assert not results[0].is_passed
        assert results[0].error is not None
