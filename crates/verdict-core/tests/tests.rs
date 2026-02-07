#[cfg(test)]
mod tests {
    use verdict_core::dataset::{DataType, Dataset, Field, Schema};

    fn make_dataset(filename: &str) -> Dataset {
        let fields = vec![
            Field::new("id", DataType::Int),
            Field::new("name", DataType::Str),
            Field::new("score", DataType::Float),
            Field::new("active", DataType::Bool),
        ];
        let schema = Schema::new(fields);
        Dataset::from_csv(filename, &schema).unwrap()
    }

    #[test]
    fn test_load_csv() {
        let dataset = make_dataset("./tests/fixtures/all_types.csv");
        assert_eq!(dataset.headers, vec!["id", "name", "score", "active"]);
        assert_eq!(dataset.shape(), (5, 4));
    }

    #[test]
    fn test_load_csv_with_nulls() {
        let dataset = make_dataset("./tests/fixtures/with_nulls.csv");
        assert_eq!(dataset.headers, vec!["id", "name", "score", "active"]);
        assert_eq!(dataset.shape(), (5, 4));
    }

    #[test]
    fn test_load_csv_invalid_path() {
        let fields = vec![Field::new("id", DataType::Int)];
        let schema = Schema::new(fields);
        let result = Dataset::from_csv("./tests/fixtures/nonexistent.csv", &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_csv_parse_error() {
        let fields = vec![
            Field::new("id", DataType::Int),
            Field::new("name", DataType::Int), // name is not an int
            Field::new("score", DataType::Float),
            Field::new("active", DataType::Bool),
        ];
        let schema = Schema::new(fields);
        let result = Dataset::from_csv("./tests/fixtures/all_types.csv", &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_column_by_name() {
        let dataset = make_dataset("./tests/fixtures/all_types.csv");
        assert_eq!(dataset.get_column_by_name("id").unwrap().len(), 5);
    }

    #[test]
    fn test_get_column_by_name_missing() {
        let dataset = make_dataset("./tests/fixtures/all_types.csv");
        assert!(dataset.get_column_by_name("nonexistent").is_none());
    }

    #[test]
    fn test_get_column_by_index() {
        let dataset = make_dataset("./tests/fixtures/all_types.csv");
        assert_eq!(dataset.get_column_by_index(0).unwrap().len(), 5);
    }

    #[test]
    fn test_get_column_by_index_out_of_bounds() {
        let dataset = make_dataset("./tests/fixtures/all_types.csv");
        assert!(dataset.get_column_by_index(99).is_none());
    }

    #[test]
    fn test_get_column_index() {
        let dataset = make_dataset("./tests/fixtures/all_types.csv");
        assert_eq!(dataset.get_column_index("score"), Some(2));
        assert_eq!(dataset.get_column_index("nonexistent"), None);
    }

    #[test]
    fn test_column_len() {
        let dataset = make_dataset("./tests/fixtures/all_types.csv");
        let col = dataset.get_column_by_name("id").unwrap();
        assert_eq!(col.len(), 5);
        assert!(!col.is_empty());
    }

    #[test]
    fn test_null_count_no_nulls() {
        let dataset = make_dataset("./tests/fixtures/all_types.csv");
        let col = dataset.get_column_by_name("id").unwrap();
        assert_eq!(col.null_count(), 0);
        assert_eq!(col.not_null_count(), 5);
    }

    #[test]
    fn test_null_count_with_nulls() {
        let dataset = make_dataset("./tests/fixtures/with_nulls.csv");
        let id_col = dataset.get_column_by_name("id").unwrap();
        assert_eq!(id_col.null_count(), 3);
        assert_eq!(id_col.not_null_count(), 2);
    }

    #[test]
    fn test_is_null_mask() {
        let dataset = make_dataset("./tests/fixtures/with_nulls.csv");
        let id_col = dataset.get_column_by_name("id").unwrap();
        let mask = id_col.is_null();
        assert_eq!(mask, vec![true, false, true, false, true]);
    }
}
