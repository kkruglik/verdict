# Phase 8: Table-level Constraints — Technical Requirements

## Context & Goal

Column-level constraints (Phase 2) validate *values inside* a column. Table-level constraints validate the *dataset itself* — its shape, column set, row count. This is the first check Great Expectations and Soda Core run before they even look at values. Without it, verdict can silently pass validation on a completely wrong file (wrong schema, truncated export, missing columns).

---

## Scope

Three constraints, all operating on `Dataset`:

| Constraint | Meaning |
|---|---|
| `RowCountBetween { min, max }` | Dataset has between `min` and `max` rows (inclusive) |
| `ColumnCountEquals(n)` | Dataset has exactly `n` columns |
| `ColumnsExist(names)` | All listed column names are present in the dataset |

---

## Naming Cleanup

Complete these renames before writing new code. If it compiles and tests pass, the rename is complete.

| Current | Rename to | Why |
|---|---|---|
| `Dataset` | `DataFrame` | Industry standard; `Dataset` is too generic |
| `StrColumn` | `StringColumn` | Breaks pattern with all other typed columns |
| `DataType::Str` | `DataType::String` | Matches Rust type name and full-word convention |
| `DatasetRule` | `TableRule` | `Dataset` leaks impl type into rules layer; `Column`/`Table` is the right axis |
| `DatasetRuleBuilder` | `TableRuleBuilder` | Follows from above |
| `InSetValues` | `ValueSet` | Constraint name (`InSet`) leaked into a data type name |
| `InSetValues::Int64Set` | `ValueSet::Int` | Exposes storage internals (`i64`) to user |
| `InSetValues::Int32Set` | `ValueSet::Date` | Exposes storage internals (`i32`) to user |
| `Keep` | `KeepStrategy` | Too terse, no meaning without context |
| `ValidateConfig` | `ValidationConfig` | Only verb-prefixed name; breaks `Validation*` pattern |

---

## Deliverables

### 0. Rename `Constraint` → `ColumnConstraint` and `Rule` → `ColumnRule`

Before touching any new code, rename the existing types across `verdict-core`, `verdict-py`, and `verdict-cli`. Do this as a **separate commit**.

> **Why now?** Once `TableConstraint` and `TableRule` exist, the old names become ambiguous. Renaming after the fact touches even more files. The existing test suite guarantees you caught every reference — if it compiles and tests pass, the rename is complete.

### 1. `TableConstraint` enum in `verdict-core`

A new enum, **separate** from `ColumnConstraint`. Do not add these variants to the existing enum.

> **Why separate?** `ColumnConstraint` variants are meaningless without a target column. Putting `RowCountBetween` there would let you write `ColumnRule { column: "whatever", constraint: ColumnConstraint::RowCountBetween { .. } }` — nonsense that compiles. A separate enum makes that unrepresentable at the type level.

### 2. `TableRule` struct

Analogous to `ColumnRule`, but holds a `TableConstraint` with no column field.

Think carefully about what fields this struct needs. It has no column name — what does it need instead?

> **Hint:** `ColumnRule` carries a column name because validation errors need to say *which column* failed. `TableRule` still needs to produce a `ValidationResult`. What will you put in the `column` field of `ValidationResult` for a table-level check? Make a deliberate choice and stay consistent.

### 3. Validation logic

A function (or functions) that takes `&Dataset` and `&[TableRule]` and returns results. Each check is simple — `Dataset::shape()` gives you `(rows, cols)`, `Dataset::get_column_by_name()` tells you if a column exists.

> **Hint:** The existing `validate()` (now operating on `&[ColumnRule]`) can stay untouched. Your new function mirrors its shape but for table rules.

The interesting question is: what does `ColumnsExist` report when multiple columns are missing? One result with all missing names, or one result per missing column?

> **Hint:** Think about what's most useful to a user debugging a pipeline. If 5 required columns are missing, do they want 5 separate results or one summary? Look at how `failed_values` works in column-level results for inspiration. There's no single right answer — but pick one and document your reasoning in a comment.

### 4. Wire into the public API

The dev plan leaves open whether table constraints go into `validate()` or a new `validate_table()`. This is the most important design decision in this phase.

Arguments for a separate `validate_table()`:
- Clean separation of concerns
- Different input type (`&[TableRule]` vs `&[Rule]`)
- Callers can run them independently

Arguments for merging into `validate()`:
- Single call site for users
- One `ValidationReport` to check
- Less API surface

> **Hint:** Think about the Python and CLI users. If a CLI user runs verdict on a CSV with the wrong schema, does it make sense to run column-level validation at all if `ColumnsExist` already failed? A separate `validate_table()` lets callers short-circuit. A merged API hides that control. Neither is wrong — pick deliberately.

### 5. Expose in Python bindings (`verdict-py`)

- Rename `Constraint` → `ColumnConstraint` and `Rule` → `ColumnRule` in the Python API
- `TableConstraint` with constructors: `row_count_between(min, max)`, `column_count_equals(n)`, `columns_exist([names])`
- `TableRule(constraint)` — no column argument
- Wire into whatever function you chose in step 4

Follow the existing PyO3 pattern — `#[pyclass(name = "...")]`, static constructors as `#[staticmethod]`.

### 6. Expose in CLI schema format

Add a `table_constraints` array to the JSON schema (alongside `columns`):

```json
{
  "columns": [...],
  "table_constraints": [
    { "type": "RowCountBetween", "min": 1000, "max": 5000 },
    { "type": "ColumnCountEquals", "value": 7 },
    { "type": "ColumnsExist", "names": ["id", "email", "created_at"] }
  ]
}
```

Add a `TableConstraintConfig` struct (mirroring how `ConstraintConfig` works), deserialize with serde, wire into the CLI runner.

---

## Acceptance Criteria

- [ ] `Constraint` → `ColumnConstraint`, `Rule` → `ColumnRule` renamed across all crates in a separate commit
- [ ] All three `TableConstraint` variants implemented and tested in `verdict-core`
- [ ] Unit tests cover: passing case, failing case, and boundary values for each constraint
- [ ] `ColumnsExist` test covers both "all present" and "some missing" cases
- [ ] Python smoke test (like `explore.py`) demonstrates table constraint usage
- [ ] CLI accepts `table_constraints` in JSON schema and outputs results correctly
- [ ] `cargo test --all-features` passes

---

## What's Out of Scope

- `validate_table()` does **not** need `ValidateConfig` / `max_failed_samples` — table constraints don't fail per-row, they fail per-dataset. No sample tracking needed.
- No YAML CLI support yet (that's Phase 9).
- No `severity` field yet (Phase 9).

---

## Start Here

Before writing a line of implementation, sketch the type signatures on paper (or in a comment block):

```
ColumnConstraint / ColumnRule — just a rename, no logic change
TableConstraint — what variants, what data each carries
TableRule — what fields
validate_table (or merged) — signature
```

Get those right first. In Rust, if your types compile, your logic is usually correct. If you find yourself fighting the borrow checker or adding ugly casts, it's usually a sign the type design needs adjustment — step back and rethink before pushing forward.
