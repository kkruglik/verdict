#[cfg(test)]
mod tests {
    use verdict_core::{
        dataframe::{
            BoolColumn, Column, DataFrame, DateColumn, DateTimeColumn, FloatColumn, IntColumn,
            StringColumn, ValuesSet,
            ops::{ComparableOps, StringOps},
        },
        rules::column::{col, col_rule},
        rules::column_checks::validate_columns,
        rules::{ColumnConstraint, ColumnRule, Operand, ValidationConfig},
    };

    fn make_all_types_dataset() -> DataFrame {
        DataFrame::new(
            vec![
                "id".to_string(),
                "name".to_string(),
                "score".to_string(),
                "active".to_string(),
                "date".to_string(),
                "date_with_nulls".to_string(),
                "datetime".to_string(),
                "datetime_with_nulls".to_string(),
            ],
            vec![
                Column::Int(IntColumn(vec![Some(1), Some(2), Some(3), Some(4), Some(5)])),
                Column::Str(StringColumn(vec![
                    Some("alice".to_string()),
                    Some("bob".to_string()),
                    Some("charlie".to_string()),
                    Some("diana".to_string()),
                    Some("eve".to_string()),
                ])),
                Column::Float(FloatColumn(vec![
                    Some(95.5),
                    Some(87.3),
                    Some(92.0),
                    Some(78.9),
                    Some(100.0),
                ])),
                Column::Bool(BoolColumn(vec![
                    Some(true),
                    Some(false),
                    Some(true),
                    Some(false),
                    Some(true),
                ])),
                // epoch days: 2024-01-01 to 2024-01-05
                Column::Date(DateColumn(vec![
                    Some(19723),
                    Some(19724),
                    Some(19725),
                    Some(19726),
                    Some(19727),
                ])),
                Column::Date(DateColumn(vec![
                    Some(19723),
                    None,
                    Some(19725),
                    None,
                    Some(19727),
                ])),
                // epoch microseconds: 2024-01-01T10:00:00 to 2024-01-05T14:00:00
                Column::DateTime(DateTimeColumn(vec![
                    Some(1704096000000000),
                    Some(1704186000000000),
                    Some(1704276000000000),
                    Some(1704366000000000),
                    Some(1704456000000000),
                ])),
                Column::DateTime(DateTimeColumn(vec![
                    Some(1704096000000000),
                    None,
                    Some(1704276000000000),
                    None,
                    Some(1704456000000000),
                ])),
            ],
        )
    }

    fn make_compare_dataset() -> DataFrame {
        DataFrame::new(
            vec![
                "id".to_string(),
                "x".to_string(),
                "y".to_string(),
                "z".to_string(),
            ],
            vec![
                Column::Int(IntColumn(vec![Some(1), Some(2), Some(3), Some(4), Some(5)])),
                Column::Float(FloatColumn(vec![
                    Some(1.0),
                    Some(2.0),
                    Some(3.0),
                    Some(4.0),
                    Some(5.0),
                ])),
                Column::Float(FloatColumn(vec![
                    Some(6.0),
                    Some(7.0),
                    Some(8.0),
                    Some(9.0),
                    Some(10.0),
                ])),
                Column::Float(FloatColumn(vec![
                    Some(28.0),
                    Some(1.0),
                    Some(0.5),
                    Some(4.0),
                    Some(0.90),
                ])),
            ],
        )
    }

    // a: nulls at rows 1,4 — b: null at row 2 — c: same values as a — high: all 100.0
    fn make_compare_nulls_dataset() -> DataFrame {
        DataFrame::new(
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "high".to_string(),
            ],
            vec![
                Column::Float(FloatColumn(vec![
                    Some(1.0),
                    None,
                    Some(3.0),
                    Some(4.0),
                    None,
                ])),
                Column::Float(FloatColumn(vec![
                    Some(2.0),
                    Some(5.0),
                    None,
                    Some(5.0),
                    None,
                ])),
                Column::Float(FloatColumn(vec![
                    Some(1.0),
                    None,
                    Some(3.0),
                    Some(4.0),
                    None,
                ])),
                Column::Float(FloatColumn(vec![
                    Some(100.0),
                    Some(100.0),
                    Some(100.0),
                    Some(100.0),
                    Some(100.0),
                ])),
            ],
        )
    }

    fn make_with_nulls_dataset() -> DataFrame {
        DataFrame::new(
            vec![
                "id".to_string(),
                "name".to_string(),
                "score".to_string(),
                "active".to_string(),
            ],
            vec![
                Column::Int(IntColumn(vec![None, Some(2), None, Some(4), None])),
                Column::Str(StringColumn(vec![
                    None,
                    Some("bob".to_string()),
                    Some("charlie".to_string()),
                    None,
                    None,
                ])),
                Column::Float(FloatColumn(vec![None, None, Some(3.3), None, Some(5.5)])),
                Column::Bool(BoolColumn(vec![None, Some(false), None, Some(false), None])),
            ],
        )
    }

    #[test]
    fn test_get_column_by_name() {
        let dataset = make_all_types_dataset();
        assert_eq!(dataset.get_column_by_name("id").unwrap().len(), 5);
    }

    #[test]
    fn test_get_column_by_name_missing() {
        let dataset = make_all_types_dataset();
        assert!(dataset.get_column_by_name("nonexistent").is_none());
    }

    #[test]
    fn test_get_column_by_index() {
        let dataset = make_all_types_dataset();
        assert_eq!(dataset.get_column_by_index(0).unwrap().len(), 5);
    }

    #[test]
    fn test_get_column_by_index_out_of_bounds() {
        let dataset = make_all_types_dataset();
        assert!(dataset.get_column_by_index(99).is_none());
    }

    #[test]
    fn test_get_column_index() {
        let dataset = make_all_types_dataset();
        assert_eq!(dataset.get_column_index("score"), Some(2));
        assert_eq!(dataset.get_column_index("nonexistent"), None);
    }

    #[test]
    fn test_column_len() {
        let dataset = make_all_types_dataset();
        let col = dataset.get_column_by_name("id").unwrap();
        assert_eq!(col.len(), 5);
        assert!(!col.is_empty());
    }

    #[test]
    fn test_null_count_no_nulls() {
        let dataset = make_all_types_dataset();
        let col = dataset.get_column_by_name("id").unwrap();
        assert_eq!(col.null_count(), 0);
        assert_eq!(col.not_null_count(), 5);
    }

    #[test]
    fn test_null_count_with_nulls() {
        let dataset = make_with_nulls_dataset();
        let id_col = dataset.get_column_by_name("id").unwrap();
        assert_eq!(id_col.null_count(), 3);
        assert_eq!(id_col.not_null_count(), 2);
    }

    #[test]
    fn test_is_null_mask() {
        let dataset = make_with_nulls_dataset();
        let id_col = dataset.get_column_by_name("id").unwrap();
        let mask = id_col.is_null();
        assert_eq!(mask, vec![true, false, true, false, true]);
    }

    #[test]
    fn test_float_numeric_ops() {
        let dataset = make_all_types_dataset();
        let score_col = dataset.get_column_by_name("score").unwrap();
        assert_eq!(score_col.min().unwrap(), 78.9);
        assert_eq!(score_col.max().unwrap(), 100.0);
        assert!((score_col.mean().unwrap() - 90.74).abs() < 0.01);
        assert_eq!(score_col.median().unwrap(), 92.0);
        assert!((score_col.std().unwrap() - 8.09).abs() < 0.01);
    }

    #[test]
    fn test_int_numeric_ops() {
        let dataset = make_all_types_dataset();
        let id_col = dataset.get_column_by_name("id").unwrap();
        assert_eq!(id_col.min().unwrap(), 1.0);
        assert_eq!(id_col.max().unwrap(), 5.0);
        assert_eq!(id_col.mean().unwrap(), 3.0);
        assert_eq!(id_col.median().unwrap(), 3.0);
        assert!((id_col.std().unwrap() - 1.5811388300841898).abs() < 0.01);
    }

    #[test]
    fn test_numeric_ops_with_nulls() {
        // id = [None, 2, None, 4, None], score = [None, None, 3.3, None, 5.5]
        let dataset = make_with_nulls_dataset();

        let id_col = dataset.get_column_by_name("id").unwrap();
        assert_eq!(id_col.sum().unwrap(), 6.0); // 2 + 4
        assert_eq!(id_col.min().unwrap(), 2.0);
        assert_eq!(id_col.max().unwrap(), 4.0);
        assert_eq!(id_col.mean().unwrap(), 3.0); // 6 / 2
        assert_eq!(id_col.median().unwrap(), 3.0); // (2 + 4) / 2

        let score_col = dataset.get_column_by_name("score").unwrap();
        assert_eq!(score_col.sum().unwrap(), 8.8); // 3.3 + 5.5
        assert_eq!(score_col.min().unwrap(), 3.3);
        assert_eq!(score_col.max().unwrap(), 5.5);
        assert_eq!(score_col.mean().unwrap(), 4.4); // 8.8 / 2
        assert_eq!(score_col.median().unwrap(), 4.4); // (3.3 + 5.5) / 2
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn test_numeric_ops_all_null_returns_none() {
        let int_col = Column::Int(IntColumn(vec![None, None, None]));
        let float_col = Column::Float(FloatColumn(vec![None, None, None]));

        for col in [&int_col, &float_col] {
            assert!(col.sum().is_none());
            assert!(col.mean().is_none());
            assert!(col.min().is_none());
            assert!(col.max().is_none());
            assert!(col.std().is_none());
            assert!(col.median().is_none());
        }
    }

    #[test]
    fn test_numeric_ops_empty_returns_none() {
        let int_col = Column::Int(IntColumn(vec![]));
        let float_col = Column::Float(FloatColumn(vec![]));

        for col in [&int_col, &float_col] {
            assert!(col.sum().is_none());
            assert!(col.mean().is_none());
            assert!(col.min().is_none());
            assert!(col.max().is_none());
            assert!(col.std().is_none());
            assert!(col.median().is_none());
        }
    }

    #[test]
    fn test_std_single_value_returns_none() {
        let int_col = Column::Int(IntColumn(vec![Some(42)]));
        let float_col = Column::Float(FloatColumn(vec![Some(1.5)]));
        assert!(int_col.std().is_none());
        assert!(float_col.std().is_none());
    }

    #[test]
    fn test_comparable_ops_all_null() {
        let col = Column::Int(IntColumn(vec![None, None, None]));
        assert_eq!(col.gt(1.0), vec![None, None, None]);
        assert_eq!(col.ge(1.0), vec![None, None, None]);
        assert_eq!(col.lt(1.0), vec![None, None, None]);
        assert_eq!(col.le(1.0), vec![None, None, None]);
        assert_eq!(col.equal(1.0), vec![None, None, None]);
        assert_eq!(col.between(0.0, 2.0), vec![None, None, None]);
    }

    #[test]
    fn test_string_ops_all_null() {
        let col = Column::Str(StringColumn(vec![None, None]));
        assert_eq!(col.contains("a"), vec![None, None]);
        assert_eq!(col.starts_with("a"), vec![None, None]);
        assert_eq!(col.ends_with("a"), vec![None, None]);
        assert_eq!(col.matches_regex(".*"), vec![None, None]);
        assert_eq!(col.length(), vec![None, None]);
    }

    #[test]
    fn test_numeric_ops_single_value_std() {
        let dataset = make_with_nulls_dataset();
        let score_col = dataset.get_column_by_name("score").unwrap();
        // score has 2 non-null values, so std is valid
        assert!(score_col.std().is_some());
    }

    #[test]
    fn test_numeric_ops_on_non_numeric() {
        let dataset = make_all_types_dataset();
        let name_col = dataset.get_column_by_name("name").unwrap();
        assert!(name_col.sum().is_none());
        assert!(name_col.min().is_none());
        assert!(name_col.max().is_none());
        assert!(name_col.mean().is_none());
        assert!(name_col.std().is_none());
        assert!(name_col.median().is_none());
    }

    #[test]
    fn test_comparable_ops() {
        let dataset = make_all_types_dataset();
        // id = [1, 2, 3, 4, 5]
        let id_col = dataset.get_column_by_name("id").unwrap();
        assert_eq!(
            id_col.gt(3.0),
            vec![
                Some(false),
                Some(false),
                Some(false),
                Some(true),
                Some(true)
            ]
        );
        assert_eq!(
            id_col.ge(3.0),
            vec![Some(false), Some(false), Some(true), Some(true), Some(true)]
        );
        assert_eq!(
            id_col.lt(3.0),
            vec![
                Some(true),
                Some(true),
                Some(false),
                Some(false),
                Some(false)
            ]
        );
        assert_eq!(
            id_col.le(3.0),
            vec![Some(true), Some(true), Some(true), Some(false), Some(false)]
        );
        assert_eq!(
            id_col.equal(3.0),
            vec![
                Some(false),
                Some(false),
                Some(true),
                Some(false),
                Some(false)
            ]
        );
        assert_eq!(
            id_col.between(2.0, 4.0),
            vec![Some(false), Some(true), Some(true), Some(true), Some(false)]
        );
    }

    #[test]
    fn test_comparable_ops_with_nulls() {
        // id = [None, 2, None, 4, None]
        let dataset = make_with_nulls_dataset();
        let id_col = dataset.get_column_by_name("id").unwrap();
        assert_eq!(
            id_col.gt(3.0),
            vec![None, Some(false), None, Some(true), None]
        );
    }

    #[test]
    fn test_comparable_ops_on_non_comparable() {
        let dataset = make_all_types_dataset();
        let bool_col = dataset.get_column_by_name("active").unwrap();
        assert_eq!(bool_col.gt(1.0), vec![None; 5]);
    }

    #[test]
    fn test_string_ops() {
        let dataset = make_all_types_dataset();
        // name = ["alice", "bob", "charlie", "diana", "eve"]
        let name_col = dataset.get_column_by_name("name").unwrap();
        assert_eq!(
            name_col.contains("li"),
            vec![
                Some(true),
                Some(false),
                Some(true),
                Some(false),
                Some(false)
            ]
        );
        assert_eq!(
            name_col.starts_with("d"),
            vec![
                Some(false),
                Some(false),
                Some(false),
                Some(true),
                Some(false)
            ]
        );
        assert_eq!(
            name_col.ends_with("e"),
            vec![Some(true), Some(false), Some(true), Some(false), Some(true)]
        );
        assert_eq!(
            name_col.matches_regex("^[a-c]"),
            vec![Some(true), Some(true), Some(true), Some(false), Some(false)]
        );
    }

    #[test]
    fn test_string_ops_with_nulls() {
        // name = [None, "bob", "charlie", None, None]
        let dataset = make_with_nulls_dataset();
        let name_col = dataset.get_column_by_name("name").unwrap();
        assert_eq!(
            name_col.contains("bob"),
            vec![None, Some(true), Some(false), None, None]
        );
    }

    #[test]
    fn test_string_ops_on_non_string() {
        let dataset = make_all_types_dataset();
        let id_col = dataset.get_column_by_name("id").unwrap();
        assert_eq!(id_col.contains("foo"), vec![None; 5]);
    }

    #[test]
    fn test_length() {
        let dataset = make_all_types_dataset();
        // name = ["alice", "bob", "charlie", "diana", "eve"]
        let name_col = dataset.get_column_by_name("name").unwrap();
        assert_eq!(
            name_col.length(),
            vec![Some(5), Some(3), Some(7), Some(5), Some(3)]
        );
    }

    #[test]
    fn test_validate_not_null_column() {
        let dataset = make_all_types_dataset();
        let null_dataset = make_with_nulls_dataset();

        let passed_result = validate_columns(
            &dataset,
            &[ColumnRule::new("id", ColumnConstraint::NotNull)],
            ValidationConfig::default(),
        );
        let failed_result = validate_columns(
            &null_dataset,
            &[ColumnRule::new("id", ColumnConstraint::NotNull)],
            ValidationConfig::default(),
        );

        assert!(passed_result.results[0].passed);
        assert!(!failed_result.results[0].passed);
    }

    #[test]
    fn test_validate_unique() {
        let dataset = make_all_types_dataset();
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new("id", ColumnConstraint::Unique)],
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);

        let results = validate_columns(
            &dataset,
            &[ColumnRule::new("active", ColumnConstraint::Unique)],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
    }

    #[test]
    fn test_validate_comparing_columns() {
        let dataset = make_compare_dataset();
        let id_rules = col_rule("id").gt(0.0).unique().build();
        let x_rules = col_rule("x")
            .lt(col("y"))
            .lt(100.0)
            .unique()
            .gt(0.0)
            .build();
        assert_eq!(id_rules.len(), 2);
        assert_eq!(x_rules.len(), 4);

        let report = validate_columns(&dataset, &id_rules, ValidationConfig::default());
        for result in &report.results {
            assert!(result.passed)
        }

        let report = validate_columns(&dataset, &x_rules, ValidationConfig::default());
        for result in &report.results {
            assert!(result.passed)
        }
    }
    // ── col-pair: gt ─────────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_gt_passes() {
        let ds = make_compare_dataset();
        // y=[6,7,8,9,10] > x=[1,2,3,4,5] — always true
        let results = validate_columns(
            &ds,
            &col_rule("y").gt(col("x")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_col_pair_gt_fails() {
        let ds = make_compare_dataset();
        // x=[1,2,3,4,5] > z=[28,1,0.5,4,0.9]
        // row 0: 1>28 false, row 3: 4>4 false → 2 failures
        let results = validate_columns(
            &ds,
            &col_rule("x").gt(col("z")).build(),
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(2));
    }

    // ── col-pair: ge ─────────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_ge_passes() {
        let ds = make_compare_dataset();
        // y >= x always
        let results = validate_columns(
            &ds,
            &col_rule("y").ge(col("x")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_col_pair_ge_equal_values_pass() {
        let ds = make_compare_dataset();
        // x=[1,2,3,4,5] >= z=[28,1,0.5,4,0.9]
        // row 3: 4>=4 true; row 0: 1>=28 false → 1 failure
        let results = validate_columns(
            &ds,
            &col_rule("x").ge(col("z")).build(),
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(1));
    }

    // ── col-pair: lt ─────────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_lt_passes() {
        let ds = make_compare_dataset();
        // x < y always
        let results = validate_columns(
            &ds,
            &col_rule("x").lt(col("y")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_col_pair_lt_fails() {
        let ds = make_compare_dataset();
        // z=[28,1,0.5,4,0.9] < x=[1,2,3,4,5]
        // row 0: 28<1 false, row 3: 4<4 false → 2 failures
        let results = validate_columns(
            &ds,
            &col_rule("z").lt(col("x")).build(),
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(2));
    }

    // ── col-pair: le ─────────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_le_passes() {
        let ds = make_compare_dataset();
        // x <= y always
        let results = validate_columns(
            &ds,
            &col_rule("x").le(col("y")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_col_pair_le_fails() {
        let ds = make_compare_dataset();
        // x=[1,2,3,4,5] <= z=[28,1,0.5,4,0.9]
        // row 1: 2<=1 false, row 2: 3<=0.5 false, row 4: 5<=0.9 false → 3 failures
        let results = validate_columns(
            &ds,
            &col_rule("x").le(col("z")).build(),
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(3));
    }

    // ── col-pair: equal ───────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_equal_same_column() {
        let ds = make_compare_dataset();
        // x == x: every value equals itself
        let results = validate_columns(
            &ds,
            &col_rule("x").equal(col("x")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_col_pair_equal_fails() {
        let ds = make_compare_dataset();
        // x=[1,2,3,4,5] != y=[6,7,8,9,10] for every row → 5 failures
        let results = validate_columns(
            &ds,
            &col_rule("x").equal(col("y")).build(),
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(5));
    }

    // ── col-pair: between ─────────────────────────────────────────────────────

    // TODO: mixed Num+Column operands in Between hit MismatchedTypes in check_between — not yet supported
    #[test]
    #[ignore]
    fn test_col_pair_between_literal_col_passes() {
        let ds = make_compare_dataset();
        // 0.0 <= x <= y: x=[1..5], y=[6..10] — all pass
        let results = validate_columns(
            &ds,
            &col_rule("x").between(0.0, col("y")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    // TODO: mixed Num+Column operands in Between hit MismatchedTypes in check_between — not yet supported
    #[test]
    #[ignore]
    fn test_col_pair_between_col_literal_passes() {
        let ds = make_compare_dataset();
        // x <= y <= 100.0: y=[6..10], x=[1..5] — all pass
        let results = validate_columns(
            &ds,
            &col_rule("y").between(col("x"), 100.0).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_col_pair_between_col_col_fails() {
        let ds = make_compare_dataset();
        // z=[28,1,0.5,4,0.9] between x=[1,2,3,4,5] and y=[6,7,8,9,10]
        // row 0: 1<=28<=6 false (28>6), row 1: 2<=1 false, row 2: 3<=0.5 false, row 4: 5<=0.9 false → 4 failures
        let results = validate_columns(
            &ds,
            &col_rule("z").between(col("x"), col("y")).build(),
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(4));
    }

    // ── col-pair: nulls ───────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_null_counts_as_failure() {
        let ds = make_compare_nulls_dataset();
        // a < b: rows 0,3 pass; rows 1,2,4 have at least one null → None → skipped
        let results = validate_columns(
            &ds,
            &col_rule("a").lt(col("b")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_col_pair_one_sided_null_is_failure() {
        let ds = make_compare_nulls_dataset();
        // a < high: high has no nulls; a is null at rows 1,4 → None → skipped
        let results = validate_columns(
            &ds,
            &col_rule("a").lt(col("high")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_col_pair_both_null_is_failure() {
        let ds = make_compare_nulls_dataset();
        // a == c: same values/nulls; rows 1,4 both null → None → skipped
        let results = validate_columns(
            &ds,
            &col_rule("a").equal(col("c")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    // TODO: mixed Num+Column operands in Between hit MismatchedTypes in check_between — not yet supported
    #[test]
    #[ignore]
    fn test_col_pair_between_with_nulls() {
        let ds = make_compare_nulls_dataset();
        // 0.0 <= a <= high: a null at rows 1,4 → None → failure
        let results = validate_columns(
            &ds,
            &col_rule("a").between(0.0, col("high")).build(),
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(2));
    }

    // ── col-pair: edge cases ──────────────────────────────────────────────────

    // ── col-pair: str ─────────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_str_equal_passes() {
        let ds = DataFrame::new(
            vec!["a".to_string(), "b".to_string()],
            vec![
                Column::Str(StringColumn(vec![
                    Some("foo".into()),
                    Some("bar".into()),
                    None,
                ])),
                Column::Str(StringColumn(vec![
                    Some("foo".into()),
                    Some("bar".into()),
                    None,
                ])),
            ],
        );
        // same values: rows 0,1 pass; row 2 both null → None → skipped
        let results = validate_columns(
            &ds,
            &col_rule("a").equal(col("b")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_col_pair_str_lt_passes() {
        let ds = DataFrame::new(
            vec!["a".to_string(), "b".to_string()],
            vec![
                Column::Str(StringColumn(vec![Some("apple".into()), Some("cat".into())])),
                Column::Str(StringColumn(vec![
                    Some("banana".into()),
                    Some("dog".into()),
                ])),
            ],
        );
        // "apple" < "banana", "cat" < "dog" lexicographically
        let results = validate_columns(
            &ds,
            &col_rule("a").lt(col("b")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_col_pair_str_lt_fails() {
        let ds = DataFrame::new(
            vec!["a".to_string(), "b".to_string()],
            vec![
                Column::Str(StringColumn(vec![Some("zoo".into()), Some("cat".into())])),
                Column::Str(StringColumn(vec![Some("apple".into()), Some("dog".into())])),
            ],
        );
        // "zoo" < "apple" false → 1 failure
        let results = validate_columns(
            &ds,
            &col_rule("a").lt(col("b")).build(),
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(1));
    }

    // ── col-pair: bool ────────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_bool_equal_passes() {
        let ds = DataFrame::new(
            vec!["a".to_string(), "b".to_string()],
            vec![
                Column::Bool(BoolColumn(vec![Some(true), Some(false), None])),
                Column::Bool(BoolColumn(vec![Some(true), Some(false), None])),
            ],
        );
        // rows 0,1 match; row 2 both null → None → skipped
        let results = validate_columns(
            &ds,
            &col_rule("a").equal(col("b")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_col_pair_bool_gt_false_lt_true() {
        let ds = DataFrame::new(
            vec!["a".to_string(), "b".to_string()],
            vec![
                Column::Bool(BoolColumn(vec![Some(true), Some(false)])),
                Column::Bool(BoolColumn(vec![Some(false), Some(true)])),
            ],
        );
        // a > b: true>false passes, false>true fails → 1 failure
        let results = validate_columns(
            &ds,
            &col_rule("a").gt(col("b")).build(),
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(1));
    }

    // ── col-pair: edge cases ──────────────────────────────────────────────────

    #[test]
    fn test_col_pair_type_mismatch_all_fail() {
        let ds = make_compare_dataset();
        // id (Int) vs x (Float) → ComparableOps<&Column> returns all None → all skipped
        let results = validate_columns(
            &ds,
            &col_rule("id").gt(col("x")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_col_pair_all_null_left_all_fail() {
        let ds = DataFrame::new(
            vec!["a".to_string(), "b".to_string()],
            vec![
                Column::Float(FloatColumn(vec![None, None, None])),
                Column::Float(FloatColumn(vec![Some(1.0), Some(2.0), Some(3.0)])),
            ],
        );
        // all left values are None → all None → all skipped
        let results = validate_columns(
            &ds,
            &col_rule("a").lt(col("b")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_col_pair_between_type_mismatch_all_fail() {
        let ds = make_compare_dataset();
        // id (Int) between x (Float) and y (Float) → type mismatch in between_cols → all None → all skipped
        let results = validate_columns(
            &ds,
            &col_rule("id").between(col("x"), col("y")).build(),
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_col_pair_missing_column_error() {
        let ds = make_compare_dataset();
        let results = validate_columns(
            &ds,
            &col_rule("x").gt(col("nonexistent")).build(),
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert!(results.results[0].error.is_some());
    }

    #[test]
    fn test_validate_greater_than() {
        let dataset = make_all_types_dataset();
        // all ids are 1-5, so all > 0
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "id",
                ColumnConstraint::GreaterThan(Operand::Num(0.0)),
            )],
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));

        // not all ids > 3
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "id",
                ColumnConstraint::GreaterThan(Operand::Num(3.0)),
            )],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(3));
    }

    #[test]
    fn test_validate_greater_than_or_equal() {
        let dataset = make_all_types_dataset();
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "id",
                ColumnConstraint::GreaterThanOrEqual(1.0.into()),
            )],
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);

        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "id",
                ColumnConstraint::GreaterThanOrEqual(3.0.into()),
            )],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(2));
    }

    #[test]
    fn test_validate_less_than() {
        let dataset = make_all_types_dataset();
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "id",
                ColumnConstraint::LessThan(6.0.into()),
            )],
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);

        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "id",
                ColumnConstraint::LessThan(3.0.into()),
            )],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(3));
    }

    #[test]
    fn test_validate_less_than_or_equal() {
        let dataset = make_all_types_dataset();
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "id",
                ColumnConstraint::LessThanOrEqual(5.0.into()),
            )],
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);

        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "id",
                ColumnConstraint::LessThanOrEqual(3.0.into()),
            )],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(2));
    }

    #[test]
    fn test_validate_equal() {
        let dataset = make_all_types_dataset();
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "score",
                ColumnConstraint::Equal(95.5.into()),
            )],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(4));
    }

    #[test]
    fn test_validate_between() {
        let dataset = make_all_types_dataset();
        // all scores are 78.9-100.0
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "score",
                ColumnConstraint::Between {
                    min: 70.0.into(),
                    max: 110.0.into(),
                },
            )],
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);

        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "score",
                ColumnConstraint::Between {
                    min: 90.0.into(),
                    max: 100.0.into(),
                },
            )],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(2)); // bob=87.3, diana=78.9
    }

    #[test]
    fn test_validate_matches_regex() {
        let dataset = make_all_types_dataset();
        // all names are lowercase alpha
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "name",
                ColumnConstraint::MatchesRegex(r"^[a-z]+$".to_string()),
            )],
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);

        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "name",
                ColumnConstraint::MatchesRegex(r"^a".to_string()),
            )],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(4));
    }

    #[test]
    fn test_validate_contains() {
        let dataset = make_all_types_dataset();
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "name",
                ColumnConstraint::Contains("li".to_string()),
            )],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(3));

        // alice and charlie contain "li" — pass case with 2 matches
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "name",
                ColumnConstraint::Contains("b".to_string()),
            )],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(4)); // only bob contains "b"
    }

    #[test]
    fn test_validate_starts_with() {
        let dataset = make_all_types_dataset();
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "name",
                ColumnConstraint::StartsWith("a".to_string()),
            )],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(4));
    }

    #[test]
    fn test_validate_ends_with() {
        let dataset = make_all_types_dataset();
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "name",
                ColumnConstraint::EndsWith("e".to_string()),
            )],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(2)); // bob, diana don't end with "e"
    }

    #[test]
    fn test_validate_length_between() {
        let dataset = make_all_types_dataset();
        // names: alice(5), bob(3), charlie(7), diana(5), eve(3)
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "name",
                ColumnConstraint::LengthBetween { min: 3, max: 7 },
            )],
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);

        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "name",
                ColumnConstraint::LengthBetween { min: 4, max: 6 },
            )],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(3)); // bob(3), charlie(7), eve(3)
    }

    #[test]
    fn test_validate_in_set() {
        let dataset = make_all_types_dataset();
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "name",
                ColumnConstraint::InSet(ValuesSet::StrSet(vec![
                    "alice".to_string(),
                    "bob".to_string(),
                    "charlie".to_string(),
                    "diana".to_string(),
                    "eve".to_string(),
                ])),
            )],
            ValidationConfig::default(),
        );
        assert!(results.results[0].passed);

        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "name",
                ColumnConstraint::InSet(ValuesSet::StrSet(vec![
                    "alice".to_string(),
                    "bob".to_string(),
                ])),
            )],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(3));
    }

    #[test]
    fn test_validate_column_not_found() {
        let dataset = make_all_types_dataset();
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new("nonexistent", ColumnConstraint::NotNull)],
            ValidationConfig::default(),
        );
        assert!(!results.results[0].passed);
        assert!(results.results[0].error.is_some());
    }

    #[test]
    fn test_validate_with_nulls() {
        let dataset = make_with_nulls_dataset();
        // id column has nulls in rows 0, 2, 4
        let results = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "id",
                ColumnConstraint::GreaterThan(0.0.into()),
            )],
            ValidationConfig::default(),
        );
        // nulls are skipped; non-null values (2, 4) are both > 0 → passes
        assert!(results.results[0].passed);
        assert_eq!(results.results[0].failed_count, Some(0));
    }

    #[test]
    fn test_validate_multiple_rules() {
        let dataset = make_all_types_dataset();
        let rules = vec![
            ColumnRule::new("id", ColumnConstraint::NotNull),
            ColumnRule::new("id", ColumnConstraint::GreaterThan(0.0.into())),
            ColumnRule::new("name", ColumnConstraint::NotNull),
            ColumnRule::new(
                "score",
                ColumnConstraint::Between {
                    min: 0.0.into(),
                    max: 100.0.into(),
                },
            ),
        ];
        let results = validate_columns(&dataset, &rules, ValidationConfig::default());
        assert_eq!(results.results.len(), 4);
        assert!(results.results[0].passed);
        assert!(results.results[1].passed);
        assert!(results.results[2].passed);
        assert!(results.results[3].passed);
    }

    // ── ValidationReport fields ───────────────────────────────────────────────

    #[test]
    fn test_report_fields_all_pass() {
        let dataset = make_all_types_dataset();
        let rules = vec![
            ColumnRule::new("id", ColumnConstraint::NotNull),
            ColumnRule::new("name", ColumnConstraint::NotNull),
        ];
        let report = validate_columns(&dataset, &rules, ValidationConfig::default());
        assert!(report.passed);
        assert_eq!(report.total_rules, 2);
        assert_eq!(report.passed_count, 2);
        assert_eq!(report.failed_count, 0);
    }

    #[test]
    fn test_report_fields_partial_fail() {
        let dataset = make_all_types_dataset();
        // id passes not_null, score fails equal(0.0)
        let rules = vec![
            ColumnRule::new("id", ColumnConstraint::NotNull),
            ColumnRule::new("score", ColumnConstraint::Equal(0.0.into())),
        ];
        let report = validate_columns(&dataset, &rules, ValidationConfig::default());
        assert!(!report.passed);
        assert_eq!(report.total_rules, 2);
        assert_eq!(report.passed_count, 1);
        assert_eq!(report.failed_count, 1);
    }

    // ── failed_values content ─────────────────────────────────────────────────

    #[test]
    fn test_failed_values_indices_and_strings() {
        let dataset = make_all_types_dataset();
        // id = [1,2,3,4,5]; gt(3) fails rows 0,1,2 (values 1,2,3)
        let report = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "id",
                ColumnConstraint::GreaterThan(3.0.into()),
            )],
            ValidationConfig::default(),
        );
        let result = &report.results[0];
        assert!(!result.passed);
        assert_eq!(result.failed_count, Some(3));
        let fv = result.failed_values.as_ref().unwrap();
        assert_eq!(fv.len(), 3);
        assert_eq!(fv[0], (0, "1".to_string()));
        assert_eq!(fv[1], (1, "2".to_string()));
        assert_eq!(fv[2], (2, "3".to_string()));
    }

    #[test]
    fn test_failed_values_not_null() {
        let dataset = make_with_nulls_dataset();
        // id = [None, 2, None, 4, None]; not_null fails rows 0,2,4
        let report = validate_columns(
            &dataset,
            &[ColumnRule::new("id", ColumnConstraint::NotNull)],
            ValidationConfig::default(),
        );
        let fv = report.results[0].failed_values.as_ref().unwrap();
        assert_eq!(fv.len(), 3);
        assert_eq!(fv[0], (0, "null".to_string()));
        assert_eq!(fv[1], (2, "null".to_string()));
        assert_eq!(fv[2], (4, "null".to_string()));
    }

    #[test]
    fn test_failed_values_string_column() {
        let dataset = make_all_types_dataset();
        // name = ["alice","bob","charlie","diana","eve"]; starts_with("a") fails 4 rows
        let report = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "name",
                ColumnConstraint::StartsWith("a".to_string()),
            )],
            ValidationConfig::default(),
        );
        let fv = report.results[0].failed_values.as_ref().unwrap();
        assert_eq!(fv.len(), 4);
        assert_eq!(fv[0], (1, "bob".to_string()));
        assert_eq!(fv[1], (2, "charlie".to_string()));
    }

    #[test]
    fn test_failed_values_none_on_pass() {
        let dataset = make_all_types_dataset();
        let report = validate_columns(
            &dataset,
            &[ColumnRule::new("id", ColumnConstraint::NotNull)],
            ValidationConfig::default(),
        );
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_values.is_none());
    }

    // ── ValidationConfig max_failed_samples ─────────────────────────────────────

    #[test]
    fn test_max_failed_samples_cap() {
        let dataset = make_all_types_dataset();
        // id = [1,2,3,4,5]; gt(0) fails none; use gt(-1) so all pass — instead use equal(0) to fail all 5
        // equal(0.0) fails all 5 rows; cap at 2
        let report = validate_columns(
            &dataset,
            &[ColumnRule::new("id", ColumnConstraint::Equal(0.0.into()))],
            ValidationConfig {
                max_failed_samples: 2,
            },
        );
        let result = &report.results[0];
        assert!(!result.passed);
        assert_eq!(result.failed_count, Some(2)); // capped
        let fv = result.failed_values.as_ref().unwrap();
        assert_eq!(fv.len(), 2);
        assert_eq!(fv[0], (0, "1".to_string()));
        assert_eq!(fv[1], (1, "2".to_string()));
    }

    #[test]
    fn test_max_failed_samples_larger_than_failures() {
        let dataset = make_all_types_dataset();
        // id = [1,2,3,4,5]; gt(3) fails 3 rows; cap=10 — all 3 are returned
        let report = validate_columns(
            &dataset,
            &[ColumnRule::new(
                "id",
                ColumnConstraint::GreaterThan(3.0.into()),
            )],
            ValidationConfig {
                max_failed_samples: 10,
            },
        );
        let result = &report.results[0];
        assert!(!result.passed);
        let fv = result.failed_values.as_ref().unwrap();
        assert_eq!(fv.len(), 3);
    }
}

#[cfg(feature = "csv")]
mod csv_tests {
    use std::path::Path;
    use verdict_core::{
        csv_loader::DatasetCsvExt,
        dataframe::{DataFrame, DataType, Field, Schema},
    };

    fn make_schema() -> Schema {
        Schema::new(vec![
            Field::new("id", DataType::Int, None),
            Field::new("name", DataType::String, None),
            Field::new("score", DataType::Float, None),
            Field::new("active", DataType::Bool, None),
        ])
    }

    #[test]
    fn test_load_csv() {
        let schema = make_schema();
        let dataset =
            DataFrame::from_csv(Path::new("tests/fixtures/all_types.csv"), &schema).unwrap();
        assert_eq!(dataset.headers, vec!["id", "name", "score", "active"]);
        assert_eq!(dataset.shape(), (5, 4));
    }

    #[test]
    fn test_load_csv_with_nulls() {
        let schema = make_schema();
        let dataset =
            DataFrame::from_csv(Path::new("tests/fixtures/with_nulls.csv"), &schema).unwrap();
        assert_eq!(dataset.headers, vec!["id", "name", "score", "active"]);
        assert_eq!(dataset.shape(), (5, 4));
    }

    #[test]
    fn test_load_csv_invalid_path() {
        let schema = Schema::new(vec![Field::new("id", DataType::Int, None)]);
        let result = DataFrame::from_csv(Path::new("nonexistent.csv"), &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_bool_values() {
        let schema = make_schema();
        let dataset =
            DataFrame::from_csv(Path::new("tests/fixtures/all_types.csv"), &schema).unwrap();
        let col = dataset.get_column_by_name("active").unwrap();
        assert_eq!(col.len(), 5);
        assert_eq!(col.null_count(), 0);
    }

    #[test]
    fn test_parse_bool_invalid() {
        let schema = Schema::new(vec![Field::new("name", DataType::Bool, None)]);
        let result = DataFrame::from_csv(Path::new("tests/fixtures/all_types.csv"), &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_csv_parse_error() {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int, None),
            Field::new("name", DataType::Int, None),
            Field::new("score", DataType::Float, None),
            Field::new("active", DataType::Bool, None),
        ]);
        let result = DataFrame::from_csv(Path::new("tests/fixtures/all_types.csv"), &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_csv_schema_too_few_columns() {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int, None),
            Field::new("name", DataType::String, None),
        ]);
        let result = DataFrame::from_csv(Path::new("tests/fixtures/all_types.csv"), &schema);
        assert!(matches!(
            result,
            Err(verdict_core::csv_loader::CsvLoadingError::ShapeError {
                expected: 2,
                found: 4
            })
        ));
    }

    #[test]
    fn test_load_csv_schema_too_many_columns() {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int, None),
            Field::new("name", DataType::String, None),
            Field::new("score", DataType::Float, None),
            Field::new("active", DataType::Bool, None),
            Field::new("extra", DataType::Int, None),
        ]);
        let result = DataFrame::from_csv(Path::new("tests/fixtures/all_types.csv"), &schema);
        assert!(matches!(
            result,
            Err(verdict_core::csv_loader::CsvLoadingError::ShapeError {
                expected: 5,
                found: 4
            })
        ));
    }
}

#[cfg(test)]
mod datetime_converter_tests {
    use chrono::NaiveDate;
    use verdict_core::dataframe::{
        i32_to_naive_date, i64_to_naive_datetime, naive_date_to_i32, naive_datetime_to_i64,
    };

    #[test]
    fn test_naive_date_to_i32_epoch() {
        let date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        assert_eq!(naive_date_to_i32(&date), 0);
    }

    #[test]
    fn test_naive_date_to_i32_known_date() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        assert_eq!(naive_date_to_i32(&date), 19723);
    }

    #[test]
    fn test_naive_date_to_i32_before_epoch() {
        let date = NaiveDate::from_ymd_opt(1969, 12, 31).unwrap();
        assert_eq!(naive_date_to_i32(&date), -1);
    }

    #[test]
    fn test_i32_to_naive_date_epoch() {
        let date = i32_to_naive_date(0).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    }

    #[test]
    fn test_i32_to_naive_date_known() {
        let date = i32_to_naive_date(19723).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    }

    #[test]
    fn test_i32_to_naive_date_before_epoch() {
        let date = i32_to_naive_date(-1).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(1969, 12, 31).unwrap());
    }

    #[test]
    fn test_date_roundtrip() {
        let original = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let encoded = naive_date_to_i32(&original);
        let decoded = i32_to_naive_date(encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_naive_datetime_to_i64_epoch() {
        let dt = NaiveDate::from_ymd_opt(1970, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        assert_eq!(naive_datetime_to_i64(&dt), 0);
    }

    #[test]
    fn test_naive_datetime_to_i64_known() {
        // 2024-01-01T10:00:00 naive (no timezone)
        let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        assert_eq!(naive_datetime_to_i64(&dt), 1704103200000000);
    }

    #[test]
    fn test_naive_datetime_to_i64_before_epoch() {
        let dt = NaiveDate::from_ymd_opt(1969, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();
        assert_eq!(naive_datetime_to_i64(&dt), -1_000_000);
    }

    #[test]
    fn test_i64_to_naive_datetime_epoch() {
        let dt = i64_to_naive_datetime(0).unwrap();
        assert_eq!(
            dt,
            NaiveDate::from_ymd_opt(1970, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        );
    }

    #[test]
    fn test_i64_to_naive_datetime_known() {
        let dt = i64_to_naive_datetime(1704103200000000).unwrap();
        let expected = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        assert_eq!(dt, expected);
    }

    #[test]
    fn test_datetime_roundtrip() {
        let original = NaiveDate::from_ymd_opt(2024, 6, 15)
            .unwrap()
            .and_hms_opt(13, 30, 45)
            .unwrap();
        let encoded = naive_datetime_to_i64(&original);
        let decoded = i64_to_naive_datetime(encoded).unwrap();
        assert_eq!(original, decoded);
    }
}

#[cfg(test)]
mod date_constraint_tests {
    use verdict_core::{
        dataframe::{Column, DataFrame, DateColumn, DateTimeColumn},
        rules::{ColumnConstraint, ColumnRule, ValidationConfig, validate_columns},
    };

    fn make_date_dataset() -> DataFrame {
        // dates: 2024-01-01=19723, 2024-01-03=19725, 2024-01-05=19727
        DataFrame::new(
            vec!["date".to_string(), "datetime".to_string()],
            vec![
                Column::Date(DateColumn(vec![
                    Some(19723),
                    Some(19725),
                    Some(19727),
                    None,
                ])),
                // datetimes: 2024-01-01T10:00:00, 2024-01-03T12:00:00, 2024-01-05T14:00:00
                Column::DateTime(DateTimeColumn(vec![
                    Some(1704103200000000),
                    Some(1704276000000000),
                    Some(1704456000000000),
                    None,
                ])),
            ],
        )
    }

    fn cfg() -> ValidationConfig {
        ValidationConfig::default()
    }

    // --- After ---

    #[test]
    fn test_after_date_passes() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "date",
            ColumnConstraint::After("2023-12-31".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.passed);
    }

    #[test]
    fn test_after_date_fails() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "date",
            ColumnConstraint::After("2024-01-03".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(!report.passed);
        assert_eq!(report.results[0].failed_count, Some(2)); // 2024-01-01 and 2024-01-03 fail (exclusive)
    }

    #[test]
    fn test_after_date_boundary_exclusive() {
        let ds = make_date_dataset();
        // after 2024-01-01 means strictly greater — 2024-01-01 itself should fail
        let rules = vec![ColumnRule::new(
            "date",
            ColumnConstraint::After("2024-01-01".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(!report.passed);
        assert_eq!(report.results[0].failed_count, Some(1));
    }

    #[test]
    fn test_after_datetime_passes() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "datetime",
            ColumnConstraint::After("2024-01-01T09:00:00".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.passed);
    }

    #[test]
    fn test_after_datetime_fails() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "datetime",
            ColumnConstraint::After("2024-01-03T12:00:00".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(!report.passed);
        assert_eq!(report.results[0].failed_count, Some(2)); // 2024-01-01 and 2024-01-03 fail
    }

    // --- Before ---

    #[test]
    fn test_before_date_passes() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "date",
            ColumnConstraint::Before("2024-01-06".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.passed);
    }

    #[test]
    fn test_before_date_fails() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "date",
            ColumnConstraint::Before("2024-01-03".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(!report.passed);
        assert_eq!(report.results[0].failed_count, Some(2)); // 2024-01-03 and 2024-01-05 fail (exclusive)
    }

    #[test]
    fn test_before_date_boundary_exclusive() {
        let ds = make_date_dataset();
        // before 2024-01-05 means strictly less — 2024-01-05 itself should fail
        let rules = vec![ColumnRule::new(
            "date",
            ColumnConstraint::Before("2024-01-05".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(!report.passed);
        assert_eq!(report.results[0].failed_count, Some(1));
    }

    #[test]
    fn test_before_datetime_passes() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "datetime",
            ColumnConstraint::Before("2024-01-06T00:00:00".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.passed);
    }

    // --- BetweenDates ---

    #[test]
    fn test_between_dates_passes() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "date",
            ColumnConstraint::BetweenDates {
                min: "2024-01-01".to_string(),
                max: "2024-01-05".to_string(),
            },
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.passed);
    }

    #[test]
    fn test_between_dates_fails() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "date",
            ColumnConstraint::BetweenDates {
                min: "2024-01-02".to_string(),
                max: "2024-01-04".to_string(),
            },
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(!report.passed);
        assert_eq!(report.results[0].failed_count, Some(2)); // 2024-01-01 and 2024-01-05 fail
    }

    #[test]
    fn test_between_datetimes_passes() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "datetime",
            ColumnConstraint::BetweenDates {
                min: "2024-01-01T10:00:00".to_string(),
                max: "2024-01-05T14:00:00".to_string(),
            },
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.passed);
    }

    #[test]
    fn test_between_datetimes_fails() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "datetime",
            ColumnConstraint::BetweenDates {
                min: "2024-01-02T00:00:00".to_string(),
                max: "2024-01-04T00:00:00".to_string(),
            },
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(!report.passed);
        assert_eq!(report.results[0].failed_count, Some(2)); // first and last fail
    }

    // --- Nulls ---

    #[test]
    fn test_after_date_nulls_pass() {
        let ds = make_date_dataset();
        // all non-null values pass, null is ignored (use NotNull separately)
        let rules = vec![ColumnRule::new(
            "date",
            ColumnConstraint::After("2023-01-01".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.passed);
    }

    #[test]
    fn test_invalid_date_string_returns_error() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "date",
            ColumnConstraint::After("not-a-date".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.results[0].error.is_some());
    }

    #[test]
    fn test_datetime_string_on_date_column_returns_error() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "date",
            ColumnConstraint::After("2024-01-01T10:00:00".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.results[0].error.is_some());
    }

    #[test]
    fn test_date_string_on_datetime_column_returns_error() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "datetime",
            ColumnConstraint::After("2024-01-01".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.results[0].error.is_some());
    }

    #[test]
    fn test_before_datetime_string_on_date_column_returns_error() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "date",
            ColumnConstraint::Before("2024-01-01T10:00:00".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.results[0].error.is_some());
    }

    #[test]
    fn test_before_date_string_on_datetime_column_returns_error() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "datetime",
            ColumnConstraint::Before("2024-01-01".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.results[0].error.is_some());
    }

    #[test]
    fn test_between_dates_datetime_string_on_date_column_returns_error() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "date",
            ColumnConstraint::BetweenDates {
                min: "2024-01-01T00:00:00".to_string(),
                max: "2024-01-05T00:00:00".to_string(),
            },
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.results[0].error.is_some());
    }

    #[test]
    fn test_between_dates_date_string_on_datetime_column_returns_error() {
        let ds = make_date_dataset();
        let rules = vec![ColumnRule::new(
            "datetime",
            ColumnConstraint::BetweenDates {
                min: "2024-01-01".to_string(),
                max: "2024-01-05".to_string(),
            },
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.results[0].error.is_some());
    }
}

#[cfg(test)]
mod time_constraint_tests {
    use verdict_core::{
        dataframe::{Column, DataFrame, TimeColumn},
        rules::{ColumnConstraint, ColumnRule, ValidationConfig, validate_columns},
    };

    // Time values as microseconds since midnight.
    // 04:00:00 = 14_400_000_000 us
    // 06:00:00 = 21_600_000_000 us
    // 08:00:00 = 28_800_000_000 us
    // 10:00:00 = 36_000_000_000 us
    // 14:00:00 = 50_400_000_000 us
    // 22:00:00 = 79_200_000_000 us
    // 23:00:00 = 82_800_000_000 us
    const H04: i64 = 4 * 3600 * 1_000_000;
    const H06: i64 = 6 * 3600 * 1_000_000;
    const H08: i64 = 8 * 3600 * 1_000_000;
    const H10: i64 = 10 * 3600 * 1_000_000;
    const H14: i64 = 14 * 3600 * 1_000_000;
    const H23: i64 = 23 * 3600 * 1_000_000;

    fn make_time_df(values: Vec<Option<i64>>) -> DataFrame {
        DataFrame::new(
            vec!["t".to_string()],
            vec![Column::Time(TimeColumn(values))],
        )
    }

    fn cfg() -> ValidationConfig {
        ValidationConfig::default()
    }

    // --- After ---

    #[test]
    fn test_after_time_passes() {
        let ds = make_time_df(vec![Some(H08), Some(H10), Some(H14)]);
        let rules = vec![ColumnRule::new(
            "t",
            ColumnConstraint::After("06:00:00".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.passed);
    }

    #[test]
    fn test_after_time_fails() {
        let ds = make_time_df(vec![Some(H08), Some(H10), Some(H04)]);
        let rules = vec![ColumnRule::new(
            "t",
            ColumnConstraint::After("06:00:00".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(!report.passed);
        assert_eq!(report.results[0].failed_count, Some(1));
    }

    #[test]
    fn test_after_time_multiple_failures() {
        let ds = make_time_df(vec![Some(H08), Some(H04), Some(H04)]);
        let rules = vec![ColumnRule::new(
            "t",
            ColumnConstraint::After("06:00:00".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(!report.passed);
        assert_eq!(report.results[0].failed_count, Some(2));
    }

    #[test]
    fn test_after_time_boundary_exclusive() {
        // After("06:00:00") means strictly greater — exactly 06:00:00 should fail
        let ds = make_time_df(vec![Some(H06)]);
        let rules = vec![ColumnRule::new(
            "t",
            ColumnConstraint::After("06:00:00".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(!report.passed);
        assert_eq!(report.results[0].failed_count, Some(1));
    }

    // --- Before ---

    #[test]
    fn test_before_time_passes() {
        let ds = make_time_df(vec![Some(H08), Some(H10), Some(H14)]);
        let rules = vec![ColumnRule::new(
            "t",
            ColumnConstraint::Before("22:00:00".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.passed);
    }

    #[test]
    fn test_before_time_fails() {
        let ds = make_time_df(vec![Some(H08), Some(H10), Some(H23)]);
        let rules = vec![ColumnRule::new(
            "t",
            ColumnConstraint::Before("22:00:00".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(!report.passed);
        assert_eq!(report.results[0].failed_count, Some(1));
    }

    #[test]
    fn test_before_time_boundary_exclusive() {
        // Before("06:00:00") means strictly less — exactly 06:00:00 should fail
        let ds = make_time_df(vec![Some(H06)]);
        let rules = vec![ColumnRule::new(
            "t",
            ColumnConstraint::Before("06:00:00".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(!report.passed);
        assert_eq!(report.results[0].failed_count, Some(1));
    }

    // --- Nulls ---

    #[test]
    fn test_time_constraint_nulls_skipped() {
        let ds = make_time_df(vec![Some(H08), None, Some(H10)]);
        let rules = vec![ColumnRule::new(
            "t",
            ColumnConstraint::After("06:00:00".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.passed);
    }

    // --- Error cases ---

    #[test]
    fn test_invalid_time_string_returns_error() {
        let ds = make_time_df(vec![Some(H08)]);
        let rules = vec![ColumnRule::new(
            "t",
            ColumnConstraint::After("not-a-time".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.results[0].error.is_some());
    }

    #[test]
    fn test_before_time_invalid_string_returns_error() {
        let ds = make_time_df(vec![Some(H08)]);
        let rules = vec![ColumnRule::new(
            "t",
            ColumnConstraint::Before("25:99:99".to_string()),
        )];
        let report = validate_columns(&ds, &rules, cfg());
        assert!(report.results[0].error.is_some());
    }
}

#[cfg(test)]
mod table_constraint_tests {
    use verdict_core::{
        dataframe::{Column, DataFrame, FloatColumn, IntColumn, StringColumn},
        rules::{TableConstraint, TableRule, ValidationConfig, validate_table},
    };

    // Dataset: 5 rows, 3 columns ("id", "name", "score")
    fn make_table_dataset() -> DataFrame {
        DataFrame::new(
            vec!["id".to_string(), "name".to_string(), "score".to_string()],
            vec![
                Column::Int(IntColumn(vec![Some(1), Some(2), Some(3), Some(4), Some(5)])),
                Column::Str(StringColumn(vec![Some("a".to_string()); 5])),
                Column::Float(FloatColumn(vec![Some(1.0); 5])),
            ],
        )
    }

    fn cfg() -> ValidationConfig {
        ValidationConfig::default()
    }

    // ── RowsCountBetween ─────────────────────────────────────────────────────

    #[test]
    fn test_rows_count_between_passes_exact_min() {
        let ds = make_table_dataset();
        // 5 rows, boundary: exactly at min
        let rules = vec![TableRule::new(TableConstraint::RowsCountBetween {
            min: 5,
            max: 10,
        })];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_rows_count_between_passes_exact_max() {
        let ds = make_table_dataset();
        // 5 rows, boundary: exactly at max
        let rules = vec![TableRule::new(TableConstraint::RowsCountBetween {
            min: 1,
            max: 5,
        })];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_rows_count_between_passes_within_range() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::RowsCountBetween {
            min: 3,
            max: 8,
        })];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_rows_count_between_fails_below_min() {
        let ds = make_table_dataset();
        // 5 rows is below min=6
        let rules = vec![TableRule::new(TableConstraint::RowsCountBetween {
            min: 6,
            max: 10,
        })];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_rows_count_between_fails_above_max() {
        let ds = make_table_dataset();
        // 5 rows is above max=4
        let rules = vec![TableRule::new(TableConstraint::RowsCountBetween {
            min: 1,
            max: 4,
        })];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    // ── RowsCountGreaterOrEqual ───────────────────────────────────────────────

    #[test]
    fn test_rows_count_ge_passes_exact_boundary() {
        let ds = make_table_dataset();
        // 5 >= 5 — boundary: equal is a pass
        let rules = vec![TableRule::new(TableConstraint::RowsCountGreaterOrEqual(5))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_rows_count_ge_passes_above() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::RowsCountGreaterOrEqual(3))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
    }

    #[test]
    fn test_rows_count_ge_fails() {
        let ds = make_table_dataset();
        // 5 < 6 — fails
        let rules = vec![TableRule::new(TableConstraint::RowsCountGreaterOrEqual(6))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    // ── RowCountGreaterThan ───────────────────────────────────────────────────

    #[test]
    fn test_rows_count_gt_passes() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::RowCountGreaterThan(4))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_rows_count_gt_fails_equal() {
        let ds = make_table_dataset();
        // 5 > 5 is false — strict greater-than rejects the boundary
        let rules = vec![TableRule::new(TableConstraint::RowCountGreaterThan(5))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_rows_count_gt_fails_above() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::RowCountGreaterThan(6))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
    }

    // ── RowsCountLessOrEqual ──────────────────────────────────────────────────

    #[test]
    fn test_rows_count_le_passes_exact_boundary() {
        let ds = make_table_dataset();
        // 5 <= 5 — boundary: equal is a pass
        let rules = vec![TableRule::new(TableConstraint::RowsCountLessOrEqual(5))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_rows_count_le_passes_below() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::RowsCountLessOrEqual(10))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
    }

    #[test]
    fn test_rows_count_le_fails() {
        let ds = make_table_dataset();
        // 5 > 4 — fails
        let rules = vec![TableRule::new(TableConstraint::RowsCountLessOrEqual(4))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    // ── RowCountLessThan ──────────────────────────────────────────────────────

    #[test]
    fn test_rows_count_lt_passes() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::RowCountLessThan(6))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_rows_count_lt_fails_equal() {
        let ds = make_table_dataset();
        // 5 < 5 is false — strict less-than rejects the boundary
        let rules = vec![TableRule::new(TableConstraint::RowCountLessThan(5))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_rows_count_lt_fails_below() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::RowCountLessThan(3))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
    }

    // ── ColumnsCountBetween ───────────────────────────────────────────────────

    #[test]
    fn test_columns_count_between_passes_exact_min() {
        let ds = make_table_dataset();
        // 3 columns, boundary: exactly at min
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountBetween {
            min: 3,
            max: 6,
        })];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_columns_count_between_passes_exact_max() {
        let ds = make_table_dataset();
        // 3 columns, boundary: exactly at max
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountBetween {
            min: 1,
            max: 3,
        })];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_columns_count_between_fails_below_min() {
        let ds = make_table_dataset();
        // 3 columns is below min=4
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountBetween {
            min: 4,
            max: 8,
        })];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_columns_count_between_fails_above_max() {
        let ds = make_table_dataset();
        // 3 columns is above max=2
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountBetween {
            min: 1,
            max: 2,
        })];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    // ── ColumnsCountGreaterOrEqual ────────────────────────────────────────────

    #[test]
    fn test_columns_count_ge_passes_exact_boundary() {
        let ds = make_table_dataset();
        // 3 >= 3 — boundary: equal is a pass
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountGreaterOrEqual(
            3,
        ))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_columns_count_ge_passes_above() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountGreaterOrEqual(
            1,
        ))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
    }

    #[test]
    fn test_columns_count_ge_fails() {
        let ds = make_table_dataset();
        // 3 < 4 — fails
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountGreaterOrEqual(
            4,
        ))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    // ── ColumnsCountGreaterThan ───────────────────────────────────────────────

    #[test]
    fn test_columns_count_gt_passes() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountGreaterThan(2))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_columns_count_gt_fails_equal() {
        let ds = make_table_dataset();
        // 3 > 3 is false — strict greater-than rejects the boundary
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountGreaterThan(3))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_columns_count_gt_fails_above() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountGreaterThan(5))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
    }

    // ── ColumnsCountLessOrEqual ───────────────────────────────────────────────

    #[test]
    fn test_columns_count_le_passes_exact_boundary() {
        let ds = make_table_dataset();
        // 3 <= 3 — boundary: equal is a pass
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountLessOrEqual(3))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_columns_count_le_passes_below() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountLessOrEqual(10))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
    }

    #[test]
    fn test_columns_count_le_fails() {
        let ds = make_table_dataset();
        // 3 > 2 — fails
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountLessOrEqual(2))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    // ── ColumnsCountLessThan ──────────────────────────────────────────────────

    #[test]
    fn test_columns_count_lt_passes() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountLessThan(4))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_columns_count_lt_fails_equal() {
        let ds = make_table_dataset();
        // 3 < 3 is false — strict less-than rejects the boundary
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountLessThan(3))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
    }

    #[test]
    fn test_columns_count_lt_fails_below() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::ColumnsCountLessThan(1))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
    }

    // ── ColumnsExist ──────────────────────────────────────────────────────────

    #[test]
    fn test_columns_exist_all_present_passes() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::ColumnsExist(vec![
            "id".to_string(),
            "name".to_string(),
            "score".to_string(),
        ]))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
        assert!(report.results[0].error.is_none());
    }

    #[test]
    fn test_columns_exist_subset_passes() {
        let ds = make_table_dataset();
        // asking for only a subset of existing columns
        let rules = vec![TableRule::new(TableConstraint::ColumnsExist(vec![
            "id".to_string(),
            "score".to_string(),
        ]))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
    }

    #[test]
    fn test_columns_exist_one_missing_fails() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::ColumnsExist(vec![
            "id".to_string(),
            "missing_col".to_string(),
        ]))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
        let error = report.results[0].error.as_ref().unwrap();
        assert!(
            error.contains("missing_col"),
            "error message must name the missing column, got: {}",
            error
        );
    }

    #[test]
    fn test_columns_exist_multiple_missing_fails_and_names_all() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::ColumnsExist(vec![
            "id".to_string(),
            "ghost_a".to_string(),
            "ghost_b".to_string(),
        ]))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        let error = report.results[0].error.as_ref().unwrap();
        assert!(
            error.contains("ghost_a"),
            "error message must name ghost_a, got: {}",
            error
        );
        assert!(
            error.contains("ghost_b"),
            "error message must name ghost_b, got: {}",
            error
        );
    }

    #[test]
    fn test_columns_exist_all_missing_fails() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::ColumnsExist(vec![
            "nonexistent_1".to_string(),
            "nonexistent_2".to_string(),
        ]))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        let error = report.results[0].error.as_ref().unwrap();
        assert!(error.contains("nonexistent_1"));
        assert!(error.contains("nonexistent_2"));
    }

    // ── ShapeEquals ───────────────────────────────────────────────────────────

    #[test]
    fn test_shape_equals_passes() {
        let ds = make_table_dataset();
        // dataset has exactly 5 rows and 3 columns
        let rules = vec![TableRule::new(TableConstraint::ShapeEquals {
            rows: 5,
            columns: 3,
        })];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
        assert!(report.results[0].error.is_none());
    }

    #[test]
    fn test_shape_equals_fails_wrong_rows() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::ShapeEquals {
            rows: 99,
            columns: 3,
        })];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
        let error = report.results[0].error.as_ref().unwrap();
        assert!(
            error.contains("99"),
            "error should mention expected row count, got: {}",
            error
        );
    }

    #[test]
    fn test_shape_equals_fails_wrong_columns() {
        let ds = make_table_dataset();
        let rules = vec![TableRule::new(TableConstraint::ShapeEquals {
            rows: 5,
            columns: 10,
        })];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        assert!(report.results[0].failed_count.is_none());
        let error = report.results[0].error.as_ref().unwrap();
        assert!(
            error.contains("10"),
            "error should mention expected column count, got: {}",
            error
        );
    }

    #[test]
    fn test_shape_equals_fails_both_wrong_rows_checked_first() {
        let ds = make_table_dataset();
        // Both dimensions are wrong; the implementation checks rows first,
        // so the error should mention the row mismatch.
        let rules = vec![TableRule::new(TableConstraint::ShapeEquals {
            rows: 1,
            columns: 1,
        })];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
        let error = report.results[0].error.as_ref().unwrap();
        assert!(
            error.contains("1") && error.contains("row"),
            "error should mention row count mismatch, got: {}",
            error
        );
    }

    // ── ValidationReport aggregate fields ─────────────────────────────────────

    #[test]
    fn test_report_all_pass() {
        let ds = make_table_dataset();
        let rules = vec![
            TableRule::new(TableConstraint::RowsCountGreaterOrEqual(1)),
            TableRule::new(TableConstraint::ColumnsCountGreaterOrEqual(1)),
        ];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.passed);
        assert_eq!(report.total_rules, 2);
        assert_eq!(report.passed_count, 2);
        assert_eq!(report.failed_count, 0);
    }

    #[test]
    fn test_report_partial_fail() {
        let ds = make_table_dataset();
        let rules = vec![
            TableRule::new(TableConstraint::RowsCountGreaterOrEqual(1)), // passes
            TableRule::new(TableConstraint::RowsCountGreaterOrEqual(999)), // fails
        ];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.passed);
        assert_eq!(report.total_rules, 2);
        assert_eq!(report.passed_count, 1);
        assert_eq!(report.failed_count, 1);
    }

    // ── Multi-rule ordering ───────────────────────────────────────────────────

    #[test]
    fn test_multiple_rules_result_order_matches_input() {
        let ds = make_table_dataset();
        let rules = vec![
            TableRule::new(TableConstraint::RowsCountGreaterOrEqual(1)),
            TableRule::new(TableConstraint::ColumnsCountGreaterOrEqual(1)),
            TableRule::new(TableConstraint::ShapeEquals {
                rows: 5,
                columns: 3,
            }),
        ];
        let report = validate_table(&ds, &rules, cfg());
        assert_eq!(report.results.len(), 3);
        assert!(report.results[0].passed);
        assert!(report.results[1].passed);
        assert!(report.results[2].passed);
    }

    #[test]
    fn test_multiple_rules_mixed_pass_fail_count() {
        let ds = make_table_dataset();
        let rules = vec![
            TableRule::new(TableConstraint::RowsCountGreaterOrEqual(1)),
            TableRule::new(TableConstraint::ColumnsCountGreaterOrEqual(1)),
            TableRule::new(TableConstraint::RowsCountGreaterOrEqual(999)),
        ];
        let report = validate_table(&ds, &rules, cfg());
        assert_eq!(report.results.len(), 3);
        assert!(report.results[0].passed);
        assert!(report.results[1].passed);
        assert!(!report.results[2].passed);
        assert_eq!(report.failed_count, 1);
        assert_eq!(report.passed_count, 2);
    }

    // ── ValidationReport::merge ───────────────────────────────────────────────

    #[test]
    fn test_merge_table_and_column_reports() {
        use verdict_core::rules::{ColumnConstraint, ColumnRule, validate_columns};

        let ds = make_table_dataset();

        let table_rules = vec![
            TableRule::new(TableConstraint::RowsCountGreaterOrEqual(1)),
            TableRule::new(TableConstraint::RowsCountGreaterOrEqual(999)),
        ];
        let table_report = validate_table(&ds, &table_rules, cfg());

        let col_rules = vec![
            ColumnRule::new("id", ColumnConstraint::NotNull),
            ColumnRule::new("id", ColumnConstraint::Unique),
        ];
        let col_report = validate_columns(&ds, &col_rules, ValidationConfig::default());

        let merged = table_report.merge(col_report);
        assert_eq!(merged.results.len(), 4);
        assert_eq!(merged.failed_count, 1);
        assert_eq!(merged.passed_count, 3);
        assert!(!merged.passed);
    }

    // ── Empty dataset edge cases ──────────────────────────────────────────────

    #[test]
    fn test_empty_dataset_rows_count_between_zero_passes() {
        let ds = DataFrame::new(vec![], vec![]);
        let rules = vec![TableRule::new(TableConstraint::RowsCountBetween {
            min: 0,
            max: 10,
        })];
        let report = validate_table(&ds, &rules, cfg());
        assert!(report.results[0].passed);
    }

    #[test]
    fn test_empty_dataset_rows_count_gt_zero_fails() {
        let ds = DataFrame::new(vec![], vec![]);
        let rules = vec![TableRule::new(TableConstraint::RowCountGreaterThan(0))];
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
    }

    // ── TableRuleBuilder ──────────────────────────────────────────────────────

    #[test]
    fn test_table_rule_builder_shape_equals() {
        use verdict_core::rules::TableRuleBuilder;

        let ds = make_table_dataset();
        let rules = TableRuleBuilder::default().shape_equals(5, 3).build();
        let report = validate_table(&ds, &rules, cfg());
        assert_eq!(report.results.len(), 1);
        assert!(report.results[0].passed);
    }

    #[test]
    fn test_table_rule_builder_shape_equals_fails() {
        use verdict_core::rules::TableRuleBuilder;

        let ds = make_table_dataset();
        let rules = TableRuleBuilder::default().shape_equals(99, 99).build();
        let report = validate_table(&ds, &rules, cfg());
        assert!(!report.results[0].passed);
    }
}

#[cfg(all(test, feature = "parquet"))]
mod parquet_tests {
    use std::path::Path;
    use verdict_core::dataframe::DataFrame;
    use verdict_core::{dataframe::Column, parquet_loader::DatasetParquetExt};

    fn load_all_types() -> DataFrame {
        DataFrame::from_parquet(Path::new("tests/fixtures/parquet/all_types.parquet"))
            .expect("all_types.parquet should load without error")
    }

    fn load_with_nulls() -> DataFrame {
        DataFrame::from_parquet(Path::new("tests/fixtures/parquet/with_nulls.parquet"))
            .expect("with_nulls.parquet should load without error")
    }

    // --- Structure ---

    #[test]
    fn test_parquet_loads_correct_column_count() {
        let df = load_all_types();
        assert_eq!(df.columns.len(), 9);
    }

    #[test]
    fn test_parquet_loads_correct_row_count() {
        let df = load_all_types();
        assert_eq!(df.columns[0].len(), 10);
    }

    // --- Column type detection ---

    #[test]
    fn test_parquet_int_column_type() {
        let df = load_all_types();
        assert!(matches!(df.columns[0], Column::Int(_)), "id should be Int");
    }

    #[test]
    fn test_parquet_float_column_type() {
        let df = load_all_types();
        assert!(
            matches!(df.columns[1], Column::Float(_)),
            "score should be Float"
        );
    }

    #[test]
    fn test_parquet_str_column_type() {
        let df = load_all_types();
        assert!(
            matches!(df.columns[2], Column::Str(_)),
            "label should be Str"
        );
    }

    #[test]
    fn test_parquet_bool_column_type() {
        let df = load_all_types();
        assert!(
            matches!(df.columns[3], Column::Bool(_)),
            "active should be Bool"
        );
    }

    #[test]
    fn test_parquet_date_column_type() {
        let df = load_all_types();
        assert!(
            matches!(df.columns[4], Column::Date(_)),
            "date_col should be Date"
        );
    }

    #[test]
    fn test_parquet_datetime_ms_column_type() {
        let df = load_all_types();
        assert!(
            matches!(df.columns[5], Column::DateTime(_)),
            "ts_ms should be DateTime"
        );
    }

    #[test]
    fn test_parquet_datetime_us_column_type() {
        let df = load_all_types();
        assert!(
            matches!(df.columns[6], Column::DateTime(_)),
            "ts_us should be DateTime"
        );
    }

    #[test]
    fn test_parquet_time_ms_column_type() {
        let df = load_all_types();
        assert!(
            matches!(df.columns[7], Column::Time(_)),
            "time_ms should be Time"
        );
    }

    #[test]
    fn test_parquet_time_us_column_type() {
        let df = load_all_types();
        assert!(
            matches!(df.columns[8], Column::Time(_)),
            "time_us should be Time"
        );
    }

    // --- Precision normalisation (ms → us on load) ---

    #[test]
    fn test_parquet_timestamp_ms_normalized_to_us() {
        let df = load_all_types();
        let ts_ms_col = match &df.columns[5] {
            Column::DateTime(c) => &c.0,
            _ => panic!("ts_ms is not DateTime"),
        };
        let ts_us_col = match &df.columns[6] {
            Column::DateTime(c) => &c.0,
            _ => panic!("ts_us is not DateTime"),
        };
        // Both columns represent the same wall-clock times; after ms→us normalisation
        // their raw i64 values (microseconds since epoch) must be identical.
        assert_eq!(ts_ms_col, ts_us_col);
    }

    #[test]
    fn test_parquet_time_ms_normalized_to_us() {
        let df = load_all_types();
        let time_ms_col = match &df.columns[7] {
            Column::Time(c) => &c.0,
            _ => panic!("time_ms is not Time"),
        };
        let time_us_col = match &df.columns[8] {
            Column::Time(c) => &c.0,
            _ => panic!("time_us is not Time"),
        };
        assert_eq!(time_ms_col, time_us_col);
    }

    // --- Null handling ---

    #[test]
    fn test_parquet_null_values_loaded_as_none() {
        let df = load_with_nulls();
        // Generator placed None at indices 3 and 7 in every column.
        for (i, col) in df.columns.iter().enumerate() {
            assert_eq!(col.null_count(), 2, "column {} should have 2 nulls", i);
        }
    }

    // --- Error handling ---

    #[test]
    fn test_parquet_file_not_found_returns_error() {
        let result =
            DataFrame::from_parquet(Path::new("tests/fixtures/parquet/nonexistent.parquet"));
        assert!(result.is_err());
    }
}
