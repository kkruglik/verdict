# Phase 10: Parquet Support — Implementation Guide

---

## Step 1: Understand what you're mirroring

Before writing a single line, read `csv_loader.rs` end to end and answer these questions for yourself:

- What is the contract of `DatasetCsvExt`? Why is it a trait on `DataFrame` rather than a free function?
- What does `ColBuilder` do and why does it exist? What problem would arise without it?
- What does the loader do when it encounters an empty string? Is that the right behavior for Parquet?

You're about to mirror this module. You need to understand every decision in it before you can make the equivalent ones for a different format.

---

## Step 2: Learn the format you're loading

Parquet is not CSV. Before touching code, understand:

- How is a Parquet file structured on disk? What are row groups?
- How does Parquet represent nulls? How is that different from an empty CSV cell?
- What are physical types vs logical types? Why does this distinction matter for mapping to verdict's `Column` types?
- How are dates and timestamps stored in Parquet? What epoch and unit do they use?

You'll hit confusing bugs during implementation if you skip this.

---

## Step 3: Make the dependency decision

Go look at both options — the `parquet` crate from arrow-rs and `parquet2`. Check their last release dates, open issues, and whether they're still actively maintained.

Then think about what "static binary" means for this project and how much binary size increase is acceptable. Make a decision and be able to defend it.

---

## Step 4: Design the interface before writing any code

What should `DatasetParquetExt` look like? Write just the trait signature — no implementation.

Questions to challenge you:
- Should it be identical to `DatasetCsvExt`, or does Parquet's self-describing nature suggest a different signature?
- Parquet files contain schema information. Should your loader use it, ignore it, or validate the file schema against verdict's schema?
- What variants should `ParquetLoadingError` have? Think about everything that can go wrong that's specific to Parquet vs what's shared with CSV errors.

Write the trait and error type first. Show them to yourself and ask: does this feel right?

---

## Step 5: Build the type mapping table

Before writing the loader body, here is the full mapping based on verdict's type system analysis:

| Parquet physical type | Parquet logical type | verdict `Column` | Notes |
|---|---|---|---|
| `BOOLEAN` | — | `BoolColumn` | direct |
| `INT32` | none | `IntColumn(i64)` | widen i32→i64 |
| `INT64` | none | `IntColumn(i64)` | direct |
| `FLOAT` | — | `FloatColumn(f64)` | widen f32→f64 |
| `DOUBLE` | — | `FloatColumn(f64)` | direct |
| `BYTE_ARRAY` | `UTF8` / `STRING` | `StringColumn` | direct |
| `INT32` | `DATE` | `DateColumn(i32)` | perfect match — both use days since Unix epoch |
| `INT64` | `TIMESTAMP(millis)` | `DateTimeColumn(i64)` | multiply × 1000 to get micros |
| `INT64` | `TIMESTAMP(micros)` | `DateTimeColumn(i64)` | direct |
| `INT64` | `TIMESTAMP(nanos)` | `DateTimeColumn(i64)` | divide ÷ 1000 — sub-microsecond precision lost |

**Types with no verdict equivalent — must return a clear error:**

| Parquet type | Why no mapping |
|---|---|
| `INT96` | Deprecated 96-bit timestamp (Julian day + nanos). Old Spark/Hive files. Complex to decode. |
| `DECIMAL` | No decimal column type in verdict. Could map to `FloatColumn` but lossy. |
| `FIXED_LEN_BYTE_ARRAY` | Used for UUIDs, decimals. No verdict equivalent unless treated as string. |
| `LIST`, `MAP`, `STRUCT` | Nested types. Verdict is flat columnar only. |
| `TIME` | Time-of-day without date. No verdict equivalent. |

For each unsupported type, ask yourself: should the loader error on the whole file, skip the column silently, or error only if that column appears in the verdict schema? What would a user expect?

---

## Step 6: Implement and handle the hard case

Now write the loader. The straightforward types will be easy. The hard case is dates and timestamps — Parquet stores them as integers with a specific epoch and precision (milliseconds? microseconds? nanoseconds?).

Look at how `csv_loader.rs` uses `naive_date_to_i32` and `naive_datetime_to_i64`. Understand what those functions expect. Then figure out how to get Parquet's integer representation into the same format.

---

## Step 7: Wire up the CLI

The CLI currently calls `DataFrame::from_csv` unconditionally. Your job is to make it choose the right loader based on the file.

Think about: where should the branching logic live? What should happen if someone passes a `.parquet` file but the binary was compiled without the `parquet` feature?

---

## Step 8: Write tests that would catch real bugs

Don't write tests that just confirm the happy path. Think about:

- A Parquet file with nulls in every column type
- A Parquet file whose schema doesn't match the verdict schema passed in
- Dates at edge values (epoch, far future)
- A column with a Parquet logical type that has no verdict equivalent

Generate a small `.parquet` fixture programmatically in the test (or using a script) — don't check in a binary file you don't control.

---

## Step 9: Reflect before calling it done

When it works, read your own code with fresh eyes and ask:

- Is there any duplication between `csv_loader.rs` and `parquet_loader.rs` that belongs in a shared utility?
- Are your error messages as useful as the CSV ones?
- Would a user who has never seen verdict's source understand what went wrong from the error output alone?
