#[cfg(test)]
mod tests {
    use verdict_core::{
        dataset::{
            BoolColumn, Column, Dataset, FloatColumn, InSetValues, IntColumn, StrColumn,
            ops::{ComparableOps, StringOps},
        },
        rules::{Constraint, Operand, Rule, col, rule, validate},
    };

    fn make_all_types_dataset() -> Dataset {
        Dataset::new(
            vec![
                "id".to_string(),
                "name".to_string(),
                "score".to_string(),
                "active".to_string(),
            ],
            vec![
                Column::Int(IntColumn(vec![Some(1), Some(2), Some(3), Some(4), Some(5)])),
                Column::Str(StrColumn(vec![
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
            ],
        )
    }

    fn make_compare_dataset() -> Dataset {
        Dataset::new(
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
    fn make_compare_nulls_dataset() -> Dataset {
        Dataset::new(
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

    fn make_with_nulls_dataset() -> Dataset {
        Dataset::new(
            vec![
                "id".to_string(),
                "name".to_string(),
                "score".to_string(),
                "active".to_string(),
            ],
            vec![
                Column::Int(IntColumn(vec![None, Some(2), None, Some(4), None])),
                Column::Str(StrColumn(vec![
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
        let col = Column::Str(StrColumn(vec![None, None]));
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

        let passed_result = validate(&dataset, &[Rule::new("id", Constraint::NotNull)]);
        let failed_result = validate(&null_dataset, &[Rule::new("id", Constraint::NotNull)]);

        assert!(passed_result[0].passed);
        assert!(!failed_result[0].passed);
    }

    #[test]
    fn test_validate_unique() {
        let dataset = make_all_types_dataset();
        let results = validate(&dataset, &[Rule::new("id", Constraint::Unique)]);
        assert!(results[0].passed);

        let results = validate(&dataset, &[Rule::new("active", Constraint::Unique)]);
        assert!(!results[0].passed);
    }

    #[test]
    fn test_validate_comparing_columns() {
        let dataset = make_compare_dataset();
        let id_rules = rule("id").gt(0.0).unique().build();
        let x_rules = rule("x").lt(col("y")).lt(100.0).unique().gt(0.0).build();
        assert_eq!(id_rules.len(), 2);
        assert_eq!(x_rules.len(), 4);

        for result in &validate(&dataset, &id_rules) {
            assert!(result.passed)
        }

        for result in &validate(&dataset, &x_rules) {
            assert!(result.passed)
        }
    }
    // ── col-pair: gt ─────────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_gt_passes() {
        let ds = make_compare_dataset();
        // y=[6,7,8,9,10] > x=[1,2,3,4,5] — always true
        let results = validate(&ds, &rule("y").gt(col("x")).build());
        assert!(results[0].passed);
        assert_eq!(results[0].failed_count, 0);
    }

    #[test]
    fn test_col_pair_gt_fails() {
        let ds = make_compare_dataset();
        // x=[1,2,3,4,5] > z=[28,1,0.5,4,0.9]
        // row 0: 1>28 false, row 3: 4>4 false → 2 failures
        let results = validate(&ds, &rule("x").gt(col("z")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 2);
    }

    // ── col-pair: ge ─────────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_ge_passes() {
        let ds = make_compare_dataset();
        // y >= x always
        let results = validate(&ds, &rule("y").ge(col("x")).build());
        assert!(results[0].passed);
        assert_eq!(results[0].failed_count, 0);
    }

    #[test]
    fn test_col_pair_ge_equal_values_pass() {
        let ds = make_compare_dataset();
        // x=[1,2,3,4,5] >= z=[28,1,0.5,4,0.9]
        // row 3: 4>=4 true; row 0: 1>=28 false → 1 failure
        let results = validate(&ds, &rule("x").ge(col("z")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 1);
    }

    // ── col-pair: lt ─────────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_lt_passes() {
        let ds = make_compare_dataset();
        // x < y always
        let results = validate(&ds, &rule("x").lt(col("y")).build());
        assert!(results[0].passed);
        assert_eq!(results[0].failed_count, 0);
    }

    #[test]
    fn test_col_pair_lt_fails() {
        let ds = make_compare_dataset();
        // z=[28,1,0.5,4,0.9] < x=[1,2,3,4,5]
        // row 0: 28<1 false, row 3: 4<4 false → 2 failures
        let results = validate(&ds, &rule("z").lt(col("x")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 2);
    }

    // ── col-pair: le ─────────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_le_passes() {
        let ds = make_compare_dataset();
        // x <= y always
        let results = validate(&ds, &rule("x").le(col("y")).build());
        assert!(results[0].passed);
        assert_eq!(results[0].failed_count, 0);
    }

    #[test]
    fn test_col_pair_le_fails() {
        let ds = make_compare_dataset();
        // x=[1,2,3,4,5] <= z=[28,1,0.5,4,0.9]
        // row 1: 2<=1 false, row 2: 3<=0.5 false, row 4: 5<=0.9 false → 3 failures
        let results = validate(&ds, &rule("x").le(col("z")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 3);
    }

    // ── col-pair: equal ───────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_equal_same_column() {
        let ds = make_compare_dataset();
        // x == x: every value equals itself
        let results = validate(&ds, &rule("x").equal(col("x")).build());
        assert!(results[0].passed);
        assert_eq!(results[0].failed_count, 0);
    }

    #[test]
    fn test_col_pair_equal_fails() {
        let ds = make_compare_dataset();
        // x=[1,2,3,4,5] != y=[6,7,8,9,10] for every row → 5 failures
        let results = validate(&ds, &rule("x").equal(col("y")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 5);
    }

    // ── col-pair: between ─────────────────────────────────────────────────────

    // TODO: mixed Num+Column operands in Between hit MismatchedTypes in check_between — not yet supported
    #[test]
    #[ignore]
    fn test_col_pair_between_literal_col_passes() {
        let ds = make_compare_dataset();
        // 0.0 <= x <= y: x=[1..5], y=[6..10] — all pass
        let results = validate(&ds, &rule("x").between(0.0, col("y")).build());
        assert!(results[0].passed);
        assert_eq!(results[0].failed_count, 0);
    }

    // TODO: mixed Num+Column operands in Between hit MismatchedTypes in check_between — not yet supported
    #[test]
    #[ignore]
    fn test_col_pair_between_col_literal_passes() {
        let ds = make_compare_dataset();
        // x <= y <= 100.0: y=[6..10], x=[1..5] — all pass
        let results = validate(&ds, &rule("y").between(col("x"), 100.0).build());
        assert!(results[0].passed);
        assert_eq!(results[0].failed_count, 0);
    }

    #[test]
    fn test_col_pair_between_col_col_fails() {
        let ds = make_compare_dataset();
        // z=[28,1,0.5,4,0.9] between x=[1,2,3,4,5] and y=[6,7,8,9,10]
        // row 0: 1<=28<=6 false (28>6), row 1: 2<=1 false, row 2: 3<=0.5 false, row 4: 5<=0.9 false → 4 failures
        let results = validate(&ds, &rule("z").between(col("x"), col("y")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 4);
    }

    // ── col-pair: nulls ───────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_null_counts_as_failure() {
        let ds = make_compare_nulls_dataset();
        // a < b: rows 0,3 pass; rows 1,2,4 have at least one null → None → failure
        let results = validate(&ds, &rule("a").lt(col("b")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 3);
    }

    #[test]
    fn test_col_pair_one_sided_null_is_failure() {
        let ds = make_compare_nulls_dataset();
        // a < high: high has no nulls; a is null at rows 1,4 → None → failure
        let results = validate(&ds, &rule("a").lt(col("high")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 2);
    }

    #[test]
    fn test_col_pair_both_null_is_failure() {
        let ds = make_compare_nulls_dataset();
        // a == c: same values/nulls; rows 1,4 both null → None → failure
        let results = validate(&ds, &rule("a").equal(col("c")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 2);
    }

    // TODO: mixed Num+Column operands in Between hit MismatchedTypes in check_between — not yet supported
    #[test]
    #[ignore]
    fn test_col_pair_between_with_nulls() {
        let ds = make_compare_nulls_dataset();
        // 0.0 <= a <= high: a null at rows 1,4 → None → failure
        let results = validate(&ds, &rule("a").between(0.0, col("high")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 2);
    }

    // ── col-pair: edge cases ──────────────────────────────────────────────────

    // ── col-pair: str ─────────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_str_equal_passes() {
        let ds = Dataset::new(
            vec!["a".to_string(), "b".to_string()],
            vec![
                Column::Str(StrColumn(vec![
                    Some("foo".into()),
                    Some("bar".into()),
                    None,
                ])),
                Column::Str(StrColumn(vec![
                    Some("foo".into()),
                    Some("bar".into()),
                    None,
                ])),
            ],
        );
        // same values: rows 0,1 pass; row 2 both null → None → failure
        let results = validate(&ds, &rule("a").equal(col("b")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 1);
    }

    #[test]
    fn test_col_pair_str_lt_passes() {
        let ds = Dataset::new(
            vec!["a".to_string(), "b".to_string()],
            vec![
                Column::Str(StrColumn(vec![Some("apple".into()), Some("cat".into())])),
                Column::Str(StrColumn(vec![Some("banana".into()), Some("dog".into())])),
            ],
        );
        // "apple" < "banana", "cat" < "dog" lexicographically
        let results = validate(&ds, &rule("a").lt(col("b")).build());
        assert!(results[0].passed);
        assert_eq!(results[0].failed_count, 0);
    }

    #[test]
    fn test_col_pair_str_lt_fails() {
        let ds = Dataset::new(
            vec!["a".to_string(), "b".to_string()],
            vec![
                Column::Str(StrColumn(vec![Some("zoo".into()), Some("cat".into())])),
                Column::Str(StrColumn(vec![Some("apple".into()), Some("dog".into())])),
            ],
        );
        // "zoo" < "apple" false → 1 failure
        let results = validate(&ds, &rule("a").lt(col("b")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 1);
    }

    // ── col-pair: bool ────────────────────────────────────────────────────────

    #[test]
    fn test_col_pair_bool_equal_passes() {
        let ds = Dataset::new(
            vec!["a".to_string(), "b".to_string()],
            vec![
                Column::Bool(BoolColumn(vec![Some(true), Some(false), None])),
                Column::Bool(BoolColumn(vec![Some(true), Some(false), None])),
            ],
        );
        // rows 0,1 match; row 2 both null → None → failure
        let results = validate(&ds, &rule("a").equal(col("b")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 1);
    }

    #[test]
    fn test_col_pair_bool_gt_false_lt_true() {
        let ds = Dataset::new(
            vec!["a".to_string(), "b".to_string()],
            vec![
                Column::Bool(BoolColumn(vec![Some(true), Some(false)])),
                Column::Bool(BoolColumn(vec![Some(false), Some(true)])),
            ],
        );
        // a > b: true>false passes, false>true fails → 1 failure
        let results = validate(&ds, &rule("a").gt(col("b")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 1);
    }

    // ── col-pair: edge cases ──────────────────────────────────────────────────

    #[test]
    fn test_col_pair_type_mismatch_all_fail() {
        let ds = make_compare_dataset();
        // id (Int) vs x (Float) → ComparableOps<&Column> returns all None → all fail
        let results = validate(&ds, &rule("id").gt(col("x")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 5);
    }

    #[test]
    fn test_col_pair_all_null_left_all_fail() {
        let ds = Dataset::new(
            vec!["a".to_string(), "b".to_string()],
            vec![
                Column::Float(FloatColumn(vec![None, None, None])),
                Column::Float(FloatColumn(vec![Some(1.0), Some(2.0), Some(3.0)])),
            ],
        );
        // all left values are None → all None → all fail
        let results = validate(&ds, &rule("a").lt(col("b")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 3);
    }

    #[test]
    fn test_col_pair_between_type_mismatch_all_fail() {
        let ds = make_compare_dataset();
        // id (Int) between x (Float) and y (Float) → type mismatch in between_cols → all None
        let results = validate(&ds, &rule("id").between(col("x"), col("y")).build());
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 5);
    }

    #[test]
    fn test_col_pair_missing_column_error() {
        let ds = make_compare_dataset();
        let results = validate(&ds, &rule("x").gt(col("nonexistent")).build());
        assert!(!results[0].passed);
        assert!(results[0].error.is_some());
    }

    #[test]
    fn test_validate_greater_than() {
        let dataset = make_all_types_dataset();
        // all ids are 1-5, so all > 0
        let results = validate(
            &dataset,
            &[Rule::new("id", Constraint::GreaterThan(Operand::Num(0.0)))],
        );
        assert!(results[0].passed);
        assert_eq!(results[0].failed_count, 0);

        // not all ids > 3
        let results = validate(
            &dataset,
            &[Rule::new("id", Constraint::GreaterThan(Operand::Num(3.0)))],
        );
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 3);
    }

    #[test]
    fn test_validate_greater_than_or_equal() {
        let dataset = make_all_types_dataset();
        let results = validate(
            &dataset,
            &[Rule::new("id", Constraint::GreaterThanOrEqual(1.0.into()))],
        );
        assert!(results[0].passed);

        let results = validate(
            &dataset,
            &[Rule::new("id", Constraint::GreaterThanOrEqual(3.0.into()))],
        );
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 2);
    }

    #[test]
    fn test_validate_less_than() {
        let dataset = make_all_types_dataset();
        let results = validate(
            &dataset,
            &[Rule::new("id", Constraint::LessThan(6.0.into()))],
        );
        assert!(results[0].passed);

        let results = validate(
            &dataset,
            &[Rule::new("id", Constraint::LessThan(3.0.into()))],
        );
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 3);
    }

    #[test]
    fn test_validate_less_than_or_equal() {
        let dataset = make_all_types_dataset();
        let results = validate(
            &dataset,
            &[Rule::new("id", Constraint::LessThanOrEqual(5.0.into()))],
        );
        assert!(results[0].passed);

        let results = validate(
            &dataset,
            &[Rule::new("id", Constraint::LessThanOrEqual(3.0.into()))],
        );
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 2);
    }

    #[test]
    fn test_validate_equal() {
        let dataset = make_all_types_dataset();
        let results = validate(
            &dataset,
            &[Rule::new("score", Constraint::Equal(95.5.into()))],
        );
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 4);
    }

    #[test]
    fn test_validate_between() {
        let dataset = make_all_types_dataset();
        // all scores are 78.9-100.0
        let results = validate(
            &dataset,
            &[Rule::new(
                "score",
                Constraint::Between {
                    min: 70.0.into(),
                    max: 110.0.into(),
                },
            )],
        );
        assert!(results[0].passed);

        let results = validate(
            &dataset,
            &[Rule::new(
                "score",
                Constraint::Between {
                    min: 90.0.into(),
                    max: 100.0.into(),
                },
            )],
        );
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 2); // bob=87.3, diana=78.9
    }

    #[test]
    fn test_validate_matches_regex() {
        let dataset = make_all_types_dataset();
        // all names are lowercase alpha
        let results = validate(
            &dataset,
            &[Rule::new(
                "name",
                Constraint::MatchesRegex(r"^[a-z]+$".to_string()),
            )],
        );
        assert!(results[0].passed);

        let results = validate(
            &dataset,
            &[Rule::new(
                "name",
                Constraint::MatchesRegex(r"^a".to_string()),
            )],
        );
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 4);
    }

    #[test]
    fn test_validate_contains() {
        let dataset = make_all_types_dataset();
        let results = validate(
            &dataset,
            &[Rule::new("name", Constraint::Contains("li".to_string()))],
        );
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 3);

        // alice and charlie contain "li" — pass case with 2 matches
        let results = validate(
            &dataset,
            &[Rule::new("name", Constraint::Contains("b".to_string()))],
        );
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 4); // only bob contains "b"
    }

    #[test]
    fn test_validate_starts_with() {
        let dataset = make_all_types_dataset();
        let results = validate(
            &dataset,
            &[Rule::new("name", Constraint::StartsWith("a".to_string()))],
        );
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 4);
    }

    #[test]
    fn test_validate_ends_with() {
        let dataset = make_all_types_dataset();
        let results = validate(
            &dataset,
            &[Rule::new("name", Constraint::EndsWith("e".to_string()))],
        );
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 2); // bob, diana don't end with "e"
    }

    #[test]
    fn test_validate_length_between() {
        let dataset = make_all_types_dataset();
        // names: alice(5), bob(3), charlie(7), diana(5), eve(3)
        let results = validate(
            &dataset,
            &[Rule::new(
                "name",
                Constraint::LengthBetween { min: 3, max: 7 },
            )],
        );
        assert!(results[0].passed);

        let results = validate(
            &dataset,
            &[Rule::new(
                "name",
                Constraint::LengthBetween { min: 4, max: 6 },
            )],
        );
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 3); // bob(3), charlie(7), eve(3)
    }

    #[test]
    fn test_validate_in_set() {
        let dataset = make_all_types_dataset();
        let results = validate(
            &dataset,
            &[Rule::new(
                "name",
                Constraint::InSet(InSetValues::StrSet(vec![
                    "alice".to_string(),
                    "bob".to_string(),
                    "charlie".to_string(),
                    "diana".to_string(),
                    "eve".to_string(),
                ])),
            )],
        );
        assert!(results[0].passed);

        let results = validate(
            &dataset,
            &[Rule::new(
                "name",
                Constraint::InSet(InSetValues::StrSet(vec![
                    "alice".to_string(),
                    "bob".to_string(),
                ])),
            )],
        );
        assert!(!results[0].passed);
        assert_eq!(results[0].failed_count, 3);
    }

    #[test]
    fn test_validate_column_not_found() {
        let dataset = make_all_types_dataset();
        let results = validate(&dataset, &[Rule::new("nonexistent", Constraint::NotNull)]);
        assert!(!results[0].passed);
        assert!(results[0].error.is_some());
    }

    #[test]
    fn test_validate_with_nulls() {
        let dataset = make_with_nulls_dataset();
        // id column has nulls in rows 0, 2, 4
        let results = validate(
            &dataset,
            &[Rule::new("id", Constraint::GreaterThan(0.0.into()))],
        );
        assert!(!results[0].passed);
        // nulls count as failures
        assert!(results[0].failed_count > 0);
    }

    #[test]
    fn test_validate_multiple_rules() {
        let dataset = make_all_types_dataset();
        let rules = vec![
            Rule::new("id", Constraint::NotNull),
            Rule::new("id", Constraint::GreaterThan(0.0.into())),
            Rule::new("name", Constraint::NotNull),
            Rule::new(
                "score",
                Constraint::Between {
                    min: 0.0.into(),
                    max: 100.0.into(),
                },
            ),
        ];
        let results = validate(&dataset, &rules);
        assert_eq!(results.len(), 4);
        assert!(results[0].passed);
        assert!(results[1].passed);
        assert!(results[2].passed);
        assert!(results[3].passed);
    }
}

#[cfg(feature = "csv")]
mod csv_tests {
    use verdict_core::{
        csv_loader::DatasetCsvExt,
        dataset::{DataType, Dataset, Field, Schema},
    };

    fn make_schema() -> Schema {
        Schema::new(vec![
            Field::new("id", DataType::Int),
            Field::new("name", DataType::Str),
            Field::new("score", DataType::Float),
            Field::new("active", DataType::Bool),
        ])
    }

    #[test]
    fn test_load_csv() {
        let schema = make_schema();
        let dataset = Dataset::from_csv("tests/fixtures/all_types.csv", &schema).unwrap();
        assert_eq!(dataset.headers, vec!["id", "name", "score", "active"]);
        assert_eq!(dataset.shape(), (5, 4));
    }

    #[test]
    fn test_load_csv_with_nulls() {
        let schema = make_schema();
        let dataset = Dataset::from_csv("tests/fixtures/with_nulls.csv", &schema).unwrap();
        assert_eq!(dataset.headers, vec!["id", "name", "score", "active"]);
        assert_eq!(dataset.shape(), (5, 4));
    }

    #[test]
    fn test_load_csv_invalid_path() {
        let schema = Schema::new(vec![Field::new("id", DataType::Int)]);
        let result = Dataset::from_csv("nonexistent.csv", &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_bool_values() {
        let schema = make_schema();
        let dataset = Dataset::from_csv("tests/fixtures/all_types.csv", &schema).unwrap();
        let col = dataset.get_column_by_name("active").unwrap();
        assert_eq!(col.len(), 5);
        assert_eq!(col.null_count(), 0);
    }

    #[test]
    fn test_parse_bool_invalid() {
        let schema = Schema::new(vec![Field::new("name", DataType::Bool)]);
        let result = Dataset::from_csv("tests/fixtures/all_types.csv", &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_csv_parse_error() {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int),
            Field::new("name", DataType::Int),
            Field::new("score", DataType::Float),
            Field::new("active", DataType::Bool),
        ]);
        let result = Dataset::from_csv("tests/fixtures/all_types.csv", &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_csv_schema_too_few_columns() {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int),
            Field::new("name", DataType::Str),
        ]);
        let result = Dataset::from_csv("tests/fixtures/all_types.csv", &schema);
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
            Field::new("id", DataType::Int),
            Field::new("name", DataType::Str),
            Field::new("score", DataType::Float),
            Field::new("active", DataType::Bool),
            Field::new("extra", DataType::Int),
        ]);
        let result = Dataset::from_csv("tests/fixtures/all_types.csv", &schema);
        assert!(matches!(
            result,
            Err(verdict_core::csv_loader::CsvLoadingError::ShapeError {
                expected: 5,
                found: 4
            })
        ));
    }
}
