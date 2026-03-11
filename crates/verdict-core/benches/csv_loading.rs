use criterion::{Criterion, criterion_group, criterion_main};
use std::path::Path;
use verdict_core::{
    csv_loader::DatasetCsvExt,
    dataset::{DataType, Dataset, Field, Schema},
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
        Field::new("country", DataType::Str),
        Field::new("country_with_nulls", DataType::Str),
    ])
}

fn bench_csv_loading(c: &mut Criterion) {
    let schema = make_schema();

    let mut group = c.benchmark_group("csv_loading");

    group.bench_function("10k rows", |b| {
        b.iter(|| Dataset::from_csv(Path::new("../../fixtures/sample_10k.csv"), &schema).unwrap())
    });

    group.bench_function("100k rows", |b| {
        b.iter(|| Dataset::from_csv(Path::new("../../fixtures/sample_100k.csv"), &schema).unwrap())
    });

    group.bench_function("1m rows", |b| {
        b.iter(|| Dataset::from_csv(Path::new("../../fixtures/sample_1m.csv"), &schema).unwrap())
    });

    group.finish();
}

criterion_group!(benches, bench_csv_loading);
criterion_main!(benches);
