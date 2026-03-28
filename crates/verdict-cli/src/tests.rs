use super::*;
use serde_json::json;

#[test]
fn parse_operand_returns_num_for_integer() {
    // Arrange
    let value = json!(42);
    // Act
    let result = parse_operand(&value).unwrap();
    // Assert
    assert!(matches!(result, Operand::Num(v) if v == 42.0));
}

#[test]
fn parse_operand_returns_num_for_float() {
    // Arrange
    let value = json!(3.14);
    // Act
    let result = parse_operand(&value).unwrap();
    // Assert
    assert!(matches!(result, Operand::Num(v) if (v - 3.14).abs() < f64::EPSILON));
}

#[test]
fn parse_operand_returns_str_for_string() {
    // Arrange
    let value = json!("hello");
    // Act
    let result = parse_operand(&value).unwrap();
    // Assert
    assert!(matches!(result, Operand::Str(s) if s == "hello"));
}

#[test]
fn parse_operand_returns_column_for_col_object() {
    // Arrange
    let value = json!({"col": "user_id"});
    // Act
    let result = parse_operand(&value).unwrap();
    // Assert
    assert!(matches!(result, Operand::Column(s) if s == "user_id"));
}

#[test]
fn parse_operand_errors_on_boolean() {
    // Arrange
    let value = json!(true);
    // Act
    let result = parse_operand(&value);
    // Assert
    assert!(result.is_err());
}

#[test]
fn parse_operand_errors_on_array() {
    // Arrange
    let value = json!([1, 2]);
    // Act
    let result = parse_operand(&value);
    // Assert
    assert!(result.is_err());
}

// --- parse_is_in ---

#[test]
fn parse_is_in_returns_int_set() {
    // Arrange
    let value = json!([1, 2, 3]);
    // Act
    let result = parse_is_in(&value).unwrap();
    // Assert
    assert!(matches!(result, ValuesSet::Int64Set(v) if v == vec![1, 2, 3]));
}

#[test]
fn parse_is_in_returns_float_set() {
    // Arrange
    let value = json!([1.1, 2.2, 3.3]);
    // Act
    let result = parse_is_in(&value).unwrap();
    // Assert
    assert!(matches!(result, ValuesSet::FloatSet(_)));
}

#[test]
fn parse_is_in_returns_str_set() {
    // Arrange
    let value = json!(["a", "b", "c"]);
    // Act
    let result = parse_is_in(&value).unwrap();
    // Assert
    assert!(
        matches!(result, ValuesSet::StrSet(v) if v == vec!["a".to_string(), "b".to_string(), "c".to_string()])
    );
}

#[test]
fn parse_is_in_errors_on_mixed_types() {
    // Arrange
    let value = json!([1, "two", 3.0]);
    // Act
    let result = parse_is_in(&value);
    // Assert
    assert!(result.is_err());
}

#[test]
fn parse_is_in_errors_on_non_array() {
    // Arrange
    let value = json!(42);
    // Act
    let result = parse_is_in(&value);
    // Assert
    assert!(result.is_err());
}

// --- parse_length_between ---

#[test]
fn parse_length_between_returns_min_max() {
    // Arrange
    let value = json!([2, 10]);
    // Act
    let (min, max) = parse_length_between(&value).unwrap();
    // Assert
    assert_eq!(min, 2);
    assert_eq!(max, 10);
}

#[test]
fn parse_length_between_accepts_zero_min() {
    // Arrange
    let value = json!([0, 5]);
    // Act
    let (min, _) = parse_length_between(&value).unwrap();
    // Assert
    assert_eq!(min, 0);
}

#[test]
fn parse_length_between_errors_on_wrong_length() {
    // Arrange
    let value = json!([1, 2, 3]);
    // Act
    let result = parse_length_between(&value);
    // Assert
    assert!(result.is_err());
}

#[test]
fn parse_length_between_errors_on_float_values() {
    // Arrange
    let value = json!([1.5, 10.0]);
    // Act
    let result = parse_length_between(&value);
    // Assert
    assert!(result.is_err());
}

#[test]
fn parse_length_between_errors_on_non_array() {
    // Arrange
    let value = json!("not an array");
    // Act
    let result = parse_length_between(&value);
    // Assert
    assert!(result.is_err());
}

// --- parse_constraint ---

#[test]
fn parse_constraint_not_null() {
    let result = parse_constraint("not_null", &json!(true)).unwrap();
    assert!(matches!(result, ColumnConstraint::NotNull));
}

#[test]
fn parse_constraint_unique() {
    let result = parse_constraint("unique", &json!(true)).unwrap();
    assert!(matches!(result, ColumnConstraint::Unique));
}

#[test]
fn parse_constraint_gt_with_number() {
    let result = parse_constraint("gt", &json!(5)).unwrap();
    assert!(matches!(result, ColumnConstraint::GreaterThan(Operand::Num(v)) if v == 5.0));
}

#[test]
fn parse_constraint_ge_with_column_ref() {
    let result = parse_constraint("ge", &json!({"col": "other"})).unwrap();
    assert!(
        matches!(result, ColumnConstraint::GreaterThanOrEqual(Operand::Column(s)) if s == "other")
    );
}

#[test]
fn parse_constraint_between_two_numbers() {
    let result = parse_constraint("between", &json!([0, 100])).unwrap();
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
    let result = parse_constraint("is_in", &json!([1, 2, 3])).unwrap();
    assert!(matches!(
        result,
        ColumnConstraint::InSet(ValuesSet::Int64Set(_))
    ));
}

#[test]
fn parse_constraint_contains_string() {
    let result = parse_constraint("contains", &json!("foo")).unwrap();
    assert!(matches!(result, ColumnConstraint::Contains(s) if s == "foo"));
}

#[test]
fn parse_constraint_length_between() {
    let result = parse_constraint("length_between", &json!([1, 50])).unwrap();
    assert!(matches!(
        result,
        ColumnConstraint::LengthBetween { min: 1, max: 50 }
    ));
}

#[test]
fn parse_constraint_errors_on_unknown_name() {
    let result = parse_constraint("nonexistent", &json!(null));
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("unsupported constraint")
    );
}
