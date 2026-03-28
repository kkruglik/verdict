use criterion::{Criterion, criterion_group, criterion_main};
use std::path::Path;
use verdict_core::{
    csv_loader::DatasetCsvExt,
    dataframe::{DataFrame, DataType, Field, Schema},
    rules::{ColumnConstraint, ColumnRule, Operand},
};

fn make_schema() -> Schema {
    Schema::new(vec![
        Field::new("user_id", DataType::Int),
        Field::new("score", DataType::Float),
        Field::new("score_with_nulls", DataType::Float),
        Field::new("age", DataType::Int),
        Field::new("age_with_nulls", DataType::Int),
        Field::new("is_active", DataType::Bool),
        Field::new("is_active_with_nulls", DataType::Bool),
        Field::new("country", DataType::String),
        Field::new("country_with_nulls", DataType::String),
    ])
}

fn make_rules() -> Vec<ColumnRule> {
    vec![
        ColumnRule::new("user_id", ColumnConstraint::NotNull),
        ColumnRule::new("user_id", ColumnConstraint::Unique),
        ColumnRule::new(
            "score",
            ColumnConstraint::Between {
                min: Operand::Num(0.0),
                max: Operand::Num(100.0),
            },
        ),
        ColumnRule::new("age", ColumnConstraint::GreaterThan(Operand::Num(0.0))),
        ColumnRule::new("age", ColumnConstraint::LessThan(Operand::Num(120.0))),
        ColumnRule::new("country", ColumnConstraint::NotNull),
    ]
}

fn bench_csv_loading(c: &mut Criterion) {
    let schema = make_schema();

    let mut group = c.benchmark_group("csv_loading");

    group.bench_function("10k rows", |b| {
        b.iter(|| DataFrame::from_csv(Path::new("../../fixtures/sample_10k.csv"), &schema).unwrap())
    });

    group.bench_function("100k rows", |b| {
        b.iter(|| {
            DataFrame::from_csv(Path::new("../../fixtures/sample_100k.csv"), &schema).unwrap()
        })
    });

    group.bench_function("1m rows", |b| {
        b.iter(|| DataFrame::from_csv(Path::new("../../fixtures/sample_1m.csv"), &schema).unwrap())
    });

    group.finish();
}

fn bench_validation(c: &mut Criterion) {
    let schema = make_schema();
    let rules = make_rules();
    let config = ValidateConfig::default();

    let dataset_100k =
        DataFrame::from_csv(Path::new("../../fixtures/sample_100k.csv"), &schema).unwrap();
    let dataset_1m =
        DataFrame::from_csv(Path::new("../../fixtures/sample_1m.csv"), &schema).unwrap();

    let mut group = c.benchmark_group("validation");

    group.bench_function("100k rows", |b| {
        b.iter(|| {
            validate(
                &dataset_100k,
                &rules,
                ValidateConfig {
                    max_failed_samples: config.max_failed_samples,
                },
            )
        })
    });

    group.bench_function("1m rows", |b| {
        b.iter(|| {
            validate(
                &dataset_1m,
                &rules,
                ValidateConfig {
                    max_failed_samples: config.max_failed_samples,
                },
            )
        })
    });

    group.finish();
}

criterion_group!(benches, bench_csv_loading, bench_validation);
criterion_main!(benches);
