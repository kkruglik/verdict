#[cfg(test)]
mod tests {
    use verdict_core::dataset::{DataType, Dataset, Field, Schema};

    use super::*;

    #[test]
    fn test_load_csv() {
        let fields = vec![
            Field::new("id", DataType::Int),
            Field::new("name", DataType::Str),
            Field::new("score", DataType::Float),
            Field::new("active", DataType::Bool),
        ];
        let schema = Schema::new(fields);
        let dataset = Dataset::from_csv("./tests/fixtures/all_types.csv", &schema).unwrap();

        assert_eq!(dataset.headers, vec!["id", "name", "score", "active"]);
        assert_eq!(dataset.shape(), (5, 4))
    }

    #[test]
    fn test_load_csv_with_nulls() {
        let fields = vec![
            Field::new("id", DataType::Int),
            Field::new("name", DataType::Str),
            Field::new("score", DataType::Float),
            Field::new("active", DataType::Bool),
        ];
        let schema = Schema::new(fields);
        let dataset = Dataset::from_csv("./tests/fixtures/with_nulls.csv", &schema).unwrap();

        assert_eq!(dataset.headers, vec!["id", "name", "score", "active"]);
        assert_eq!(dataset.shape(), (5, 4))
    }
}
