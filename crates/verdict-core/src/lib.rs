pub mod dataframe;
pub mod errors;
pub mod rules;

#[cfg(feature = "csv")]
pub mod csv_loader;

#[cfg(feature = "parquet")]
pub mod parquet_loader;
