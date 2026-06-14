use super::*;
use serde_json::json;

#[test]
fn parse_operand_returns_num_for_integer() {
    let value = json!(42);
    let result = parse_operand(&value).unwrap();
    assert!(matches!(result, Operand::Num(v) if v == 42.0));
}

#[test]
fn parse_operand_returns_num_for_float() {
    let value = json!(1.5);
    let result = parse_operand(&value).unwrap();
    assert!(matches!(result, Operand::Num(v) if v == 1.5));
}

#[test]
fn parse_operand_returns_str_for_string() {
    let value = json!("hello");
    let result = parse_operand(&value).unwrap();
    assert!(matches!(result, Operand::Str(s) if s == "hello"));
}

#[test]
fn parse_operand_returns_column_for_col_object() {
    let value = json!({"col": "user_id"});
    let result = parse_operand(&value).unwrap();
    assert!(matches!(result, Operand::Column(s) if s == "user_id"));
}

#[test]
fn parse_operand_errors_on_boolean() {
    let value = json!(true);
    let result = parse_operand(&value);
    assert!(result.is_err());
}

#[test]
fn parse_operand_errors_on_array() {
    let value = json!([1, 2]);
    let result = parse_operand(&value);
    assert!(result.is_err());
}

#[test]
fn parse_is_in_returns_int_set() {
    let value = json!([1, 2, 3]);
    let result = parse_is_in(&value, &DtypeConfig::Int).unwrap();
    assert!(matches!(result, ValuesSet::Int64Set(v) if v == vec![1, 2, 3]));
}

#[test]
fn parse_is_in_returns_float_set() {
    let value = json!([1.1, 2.2, 3.3]);
    let result = parse_is_in(&value, &DtypeConfig::Float).unwrap();
    assert!(matches!(result, ValuesSet::FloatSet(_)));
}

#[test]
fn parse_is_in_returns_str_set() {
    let value = json!(["a", "b", "c"]);
    let result = parse_is_in(&value, &DtypeConfig::Str).unwrap();
    assert!(
        matches!(result, ValuesSet::StrSet(v) if v == vec!["a".to_string(), "b".to_string(), "c".to_string()])
    );
}

#[test]
fn parse_is_in_errors_on_mixed_types() {
    let value = json!([1, "two", 3.0]);
    let result = parse_is_in(&value, &DtypeConfig::Int);
    assert!(result.is_err());
}

#[test]
fn parse_is_in_errors_on_non_array() {
    let value = json!(42);
    let result = parse_is_in(&value, &DtypeConfig::Int);
    assert!(result.is_err());
}

#[test]
fn parse_length_between_returns_min_max() {
    let value = json!([2, 10]);
    let (min, max) = parse_length_between(&value).unwrap();
    assert_eq!(min, 2);
    assert_eq!(max, 10);
}

#[test]
fn parse_length_between_accepts_zero_min() {
    let value = json!([0, 5]);
    let (min, _) = parse_length_between(&value).unwrap();
    assert_eq!(min, 0);
}

#[test]
fn parse_length_between_errors_on_wrong_length() {
    let value = json!([1, 2, 3]);
    let result = parse_length_between(&value);
    assert!(result.is_err());
}

#[test]
fn parse_length_between_errors_on_float_values() {
    let value = json!([1.5, 10.0]);
    let result = parse_length_between(&value);
    assert!(result.is_err());
}

#[test]
fn parse_length_between_errors_on_non_array() {
    let value = json!("not an array");
    let result = parse_length_between(&value);
    assert!(result.is_err());
}

#[test]
fn parse_constraint_not_null() {
    let result = parse_column_constraint("not_null", &json!(true), &DtypeConfig::Int).unwrap();
    assert!(matches!(result, ColumnConstraint::NotNull));
}

#[test]
fn parse_constraint_unique() {
    let result = parse_column_constraint("unique", &json!(true), &DtypeConfig::Int).unwrap();
    assert!(matches!(result, ColumnConstraint::Unique));
}

#[test]
fn parse_constraint_gt_with_number() {
    let result = parse_column_constraint("gt", &json!(5), &DtypeConfig::Int).unwrap();
    assert!(matches!(result, ColumnConstraint::GreaterThan(Operand::Num(v)) if v == 5.0));
}

#[test]
fn parse_constraint_ge_with_column_ref() {
    let result =
        parse_column_constraint("ge", &json!({"col": "other"}), &DtypeConfig::Int).unwrap();
    assert!(
        matches!(result, ColumnConstraint::GreaterThanOrEqual(Operand::Column(s)) if s == "other")
    );
}

#[test]
fn parse_constraint_between_two_numbers() {
    let result = parse_column_constraint("between", &json!([0, 100]), &DtypeConfig::Int).unwrap();
    assert!(matches!(
        result,
        ColumnConstraint::Between {
            min: Operand::Num(_),
            max: Operand::Num(_),
        }
    ));
}

#[test]
fn parse_constraint_is_in_integers() {
    let result = parse_column_constraint("is_in", &json!([1, 2, 3]), &DtypeConfig::Int).unwrap();
    assert!(matches!(
        result,
        ColumnConstraint::InSet(ValuesSet::Int64Set(_))
    ));
}

#[test]
fn parse_constraint_contains_string() {
    let result = parse_column_constraint("contains", &json!("foo"), &DtypeConfig::Str).unwrap();
    assert!(matches!(result, ColumnConstraint::Contains(s) if s == "foo"));
}

#[test]
fn parse_constraint_length_between() {
    let result =
        parse_column_constraint("length_between", &json!([1, 50]), &DtypeConfig::Str).unwrap();
    assert!(matches!(
        result,
        ColumnConstraint::LengthBetween { min: 1, max: 50 }
    ));
}

#[test]
fn parse_constraint_errors_on_unknown_name() {
    let result = parse_column_constraint("nonexistent", &json!(null), &DtypeConfig::Int);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("unsupported constraint")
    );
}
