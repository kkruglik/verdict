# Session: Parquet Loading & TimeColumn Implementation

## 1. Initial Goal
Implement `DataFrame::from_parquet` — load a Parquet file into the existing `DataFrame` type.

---

## 2. Schema Type Mapping

### Attempt
Used `get_physical_type()` to map parquet columns to `DataType`.

### Issue
Physical types are too coarse. `BYTE_ARRAY` could be a string, JSON, UUID, date, decimal — the physical type alone doesn't tell you which. `INT32` could be an integer or a date.

### Fix
Added `get_basic_info().logical_type_ref()` as the primary type source, falling back to physical type when no logical annotation exists. One loop, logical type first, physical type as else branch.

---

## 3. Two-Pass Architecture (Schema + Data)

### Attempt
Original code had two separate loops: one to build `col_types_mapping: Vec<DataType>`, another to build `columns_data: Vec<ColBuilder>` from that mapping.

### Issue
Redundant intermediate step — `DataType` was only used to create `ColBuilder`. Two passes over the same schema data.

### Fix
Collapsed into one loop. Iterate `fields` once, produce `ColBuilder` directly. `DataType` removed from parquet loader entirely. Used `Vec::with_capacity(num_rows)` for preallocation since `file_metadata().num_rows()` is available before reading data.

---

## 4. Row Iteration

### Attempt
```rust
for row_iter in reader.get_row_iter(None)? {
    for row in row_iter {
```

### Issue
`get_row_iter(None)?` returns `RowIter` — unwrapping the `Result` of creating the iterator. Then `for row in row_iter` iterates rows. But `RowIter::Item = Row`, not `Result<Row>`. The double loop was wrong — `row_iter` was already a `Row`.

### Confusion
The `RowIter` actually does yield `Row` directly (Item = Row), so the outer loop was iterating over `RowIter` which yields rows. The inner `for row in row_iter` tried to iterate a `Row` which is not an iterator.

### Fix
```rust
for row in reader.get_row_iter(None)? {
    let row = row?;
```
`get_row_iter` returns `Result<RowIter>` — the `?` unwraps that. Each iteration yields `Row` directly. The separate `let row = row?` is because `RowIter` actually yields `Result<Row>` per item (each row read can independently fail on corrupted data).

---

## 5. Field Dereferencing

### Attempt
```rust
(ColBuilder::Int(v), Field::Int(val)) => v.push(Some(val as i64)),
```

### Issue
`get_column_iter()` yields `(&str, &Field)` — field is a reference. When matching `Field::Int(val)` through a reference, `val` binds as `&i32` for non-Copy types and `i32` for Copy types depending on match ergonomics. Casts like `val as i64` on `&i32` fail, and `*val` was needed in some cases.

### Fix
Used `*val` consistently for all primitive dereferences.

---

## 6. Semicolon Bug

### Issue
Both `match` blocks in the `if let Some(l_type)` / `else` branches had a semicolon after the closing `}`:
```rust
match l_type { ... };  // <-- semicolon turns expression into statement returning ()
```
This made `mapped_type` have type `()` instead of `DataType`.

### Fix
Removed the semicolons. `match` is an expression in Rust — adding `;` discards the value.

---

## 7. Unused Imports and Variables

### Issues found during compilation
- `std::collections::HashMap` — unused after refactor
- `BoolColumn`, `Schema` — unused imports
- `Row` — unused import  
- `ColumnReader` — unused import
- `pos` in `for (pos, col)` — pos never used
- `schema` variable from `schema_descr()` — never used
- `Option<String>` second field in `Date`/`DateTime` `ColBuilder` variants — never read

### Fix
Removed all of them iteratively as compiler warnings surfaced.

---

## 8. PathBuf::ends_with Trap

### Attempt
```rust
if cli.filename.ends_with(".csv") {
```

### Issue
`PathBuf::ends_with` checks path *components*, not string suffixes. `"sample.parquet".ends_with(".parquet")` returns `false` because `.parquet` is not a full path component.

### Fix
```rust
cli.filename.extension().is_some_and(|e| e == "csv")
```
`extension()` returns `Option<&OsStr>` without the dot. `OsStr` implements `PartialEq<str>` so the comparison works directly.

---

## 9. Parquet Date Fixtures

### Attempt
Converted CSV fixtures to Parquet using pandas `df.to_parquet()`.

### Issue
Pandas does not auto-parse date/datetime columns from CSV. `created_date` and `created_at` were stored as `BYTE_ARRAY/STRING` in the Parquet file. When the validator applied `After`/`Before` constraints, the column was `Column::Str` instead of `Column::Date`/`Column::DateTime`, hitting `unreachable!()` and panicking.

### Fix
Used pyarrow directly to cast columns to proper types before writing:
```python
table = pa.Table.from_pandas(df)
# cast date columns to date32, datetime columns stay as timestamp[us]
table = table.cast(schema_with_date32)
pq.write_table(table, out)
```

---

## 10. Unsupported Parquet Types

### Issue
`_ => panic!("Unsupported type!")` in both the schema pass and the data pass. Panics in a library are bad — callers can't recover.

### Fix
Added two error variants to `ParquetLoadingError`:
- `UnsupportedType { column, type_name }` — returned from schema pass when a logical type is not supported
- `TypeMismatch { column, row, expected, got }` — returned from data pass when `Field` variant doesn't match `ColBuilder`

No panics remain in the loader.

---

## 11. Field Variant Coverage

### Issue
Many `Field` variants were not handled: `Byte`, `Short`, `UByte`, `UShort`, `UInt`, `ULong`, `Float16`, `Decimal`, `TimeMillis`, `TimeMicros`, `ListInternal`, `MapInternal`, `Bytes`, `Group`.

### Decisions per type
- `Byte/Short/UByte/UShort/UInt/ULong` → cast to `i64`, stored in `Int`
- `Float16` → `f32::from(half) as f64`
- `Decimal` → byte fold to reconstruct unscaled integer, divide by `10^scale` to get `f64`. Note: `Decimal` does not implement `Display`, so `to_string()` doesn't work.
- `TimeMillis/TimeMicros` → initially routed to `ColBuilder::Int`, later moved to `ColBuilder::Time` after `TimeColumn` was added
- `ListInternal` → elements joined with `,` as a string
- `MapInternal` → entries formatted as `"key, value"` joined with `,`
- `Bytes` → `val.as_utf8()?` to get UTF-8 string
- `Group` → `None` pushed (nested structs not supported)

---

## 12. TimeColumn

### Goal
Add proper `Time` column type instead of storing time-of-day values as `Int`.

### Storage unit decision
Milliseconds since midnight as `i32`. Max value `86_399_999` fits in `i32`. Consistent with Parquet's `TimeMillis` representation.

### Issues found during implementation
- `ColBuilder::Time` added but `Column::Time` arms missing from 14 places: `len`, `is_null`, `null_count`, `not_null_count`, `unique_count`, `duplicated`, `is_in`, 8 match blocks in `column_checks.rs` for failed value stringification
- `TimeColumn` not re-exported from `dataframe/mod.rs`
- `ComparableOps<i32> for Column` didn't dispatch to `Column::Time` — silently returned all `None`
- `Column::is_in` had no `Time` arm — silently returned all `None`
- `naive_time_to_i32` was returning seconds, not milliseconds — unit mismatch with parquet `TimeMillis`
- CSV loader had wrong default format `"%Y-%m-%d"` and used `NaiveDate` instead of `NaiveTime`
- `After`/`Before` constraints had no `Column::Time` arm — hit `unreachable!()` and panicked
- `Field::TimeMillis`/`Field::TimeMicros` in parquet loader were routing to `ColBuilder::Int` instead of `ColBuilder::Time`

### Key insight on unit consistency
For CSV-only path: fully consistent regardless of unit, because both stored values and constraint thresholds go through `naive_time_to_i32`. Unit only matters when mixing CSV and Parquet sources.

### Not implemented (future work)
- `BetweenTimes` constraint variant
- Cross-column time comparisons (`ComparableOps<&Column>` for `Time`)
- `i32_to_naive_time` inverse for human-readable error messages
