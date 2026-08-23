# Traits in Depth — What the Rust Book Skipped

The Rust Book teaches you what traits are and how to write simple ones. What it doesn't teach is the set of decisions you face when designing a real trait system: when to use a type parameter vs an associated type, what the three identical-looking `T`s in an impl block actually mean, or how to make one type implement the same trait fifteen different ways.

This chapter works through those patterns using a concrete running example: a column type system for a data validation library. You'll see each pattern appear as a real problem, not a contrived example.

---

## 0. The Running Example

The codebase stores tabular data as typed columns. Each column is a wrapper around a vector of nullable values:

```rust
pub struct TypedColumn<T>(pub Vec<Option<T>>);
```

The `Option<T>` represents nullable data — a cell can hold a value or be null. The generic `T` determines what kind of data the column stores.

We also have a type-erased enum that holds any column variant:

```rust
pub enum Column {
    Int(IntColumn),
    Float(FloatColumn),
    Str(StringColumn),
    Bool(BoolColumn),
    Date(DateColumn),
    DateTime(DateTimeColumn),
    Time(TimeColumn),
}
```

By the end of this chapter you'll understand the entire trait system built on top of these two types.

---

## 1. Type Aliases

The simplest thing you can do with `TypedColumn<T>` is give specific instantiations a name:

```rust
type IntColumn      = TypedColumn<i64>;
type FloatColumn    = TypedColumn<f64>;
type BoolColumn     = TypedColumn<bool>;
type StringColumn   = TypedColumn<String>;
type DateColumn     = TypedColumn<i32>;
```

A type alias is just a name — it creates no new type. `IntColumn` and `TypedColumn<i64>` are identical to the compiler. You can use them interchangeably, and you cannot write a new `impl` block for `IntColumn` that wouldn't also apply to `TypedColumn<i64>`.

### Type aliases vs newtypes

Now look at the date-like columns:

```rust
type DateTimeColumn = TypedColumn<i64>;
type TimeColumn     = TypedColumn<i64>;
```

Both store `i64` internally (microseconds since epoch for datetime, microseconds since midnight for time). As type aliases, `DateTimeColumn` and `TimeColumn` are the **same type**. The compiler sees them as identical. You cannot put them in separate enum variants, you cannot write separate `impl` blocks for them, and you cannot have a function that accepts one but not the other.

This is why the codebase uses **newtypes** for columns that share the same storage type:

```rust
pub struct DateTimeColumn(pub Vec<Option<i64>>);
pub struct TimeColumn(pub Vec<Option<i64>>);
```

A newtype is a separate struct. The compiler treats `DateTimeColumn` and `TimeColumn` as completely different types even though they contain identical data. Each gets its own `impl` blocks, its own trait implementations, and its own enum variant.

**Rule**: use a type alias when you want a shorter name. Use a newtype when you need type identity — when the same underlying storage must represent semantically different things.

---

## 2. Trait Type Parameters vs Associated Types

This is the most consequential decision when designing a trait. The Rust Book covers both forms but doesn't clearly explain when to use which.

### Associated types — `NumericOps`

```rust
pub trait NumericOps {
    type Item;
    fn sum(&self) -> Option<Self::Item>;
    fn min(&self) -> Option<Self::Item>;
    fn max(&self) -> Option<Self::Item>;
    fn mean(&self) -> Option<f64>;
    fn std(&self) -> Option<f64>;
    fn median(&self) -> Option<f64>;
}
```

`type Item` is an associated type. Each type that implements `NumericOps` sets `Item` to exactly one concrete type:

```rust
impl NumericOps for IntColumn {
    type Item = i64;

    fn sum(&self) -> Option<i64> { ... }
    fn min(&self) -> Option<i64> { ... }
    fn max(&self) -> Option<i64> { ... }
}

impl NumericOps for FloatColumn {
    type Item = f64;

    fn sum(&self) -> Option<f64> { ... }
    fn min(&self) -> Option<f64> { ... }
    fn max(&self) -> Option<f64> { ... }
}
```

`IntColumn` can only implement `NumericOps` once. Once you've written `type Item = i64`, that's what `sum` returns forever for `IntColumn`.

### Type parameters — `ComparableOps<T>`

```rust
pub trait ComparableOps<T> {
    fn gt(&self, compare: T) -> Vec<Option<bool>>;
    fn ge(&self, compare: T) -> Vec<Option<bool>>;
    fn lt(&self, compare: T) -> Vec<Option<bool>>;
    fn le(&self, compare: T) -> Vec<Option<bool>>;
    fn equal(&self, compare: T) -> Vec<Option<bool>>;
    fn between(&self, lower: T, upper: T) -> Vec<Option<bool>>;
}
```

`T` is a type parameter on the trait itself. This changes everything: a single type can implement `ComparableOps<T>` for **many different values of `T`**:

```rust
impl ComparableOps<i64>        for IntColumn { ... }  // compare against an integer
impl ComparableOps<f64>        for IntColumn { ... }  // compare against a float
impl ComparableOps<&IntColumn> for IntColumn { ... }  // compare against another column
```

`IntColumn` implements the same trait three times with three different type parameters. This would be impossible with an associated type — you'd be defining `type T` three times for the same `Self`.

### The decision rule

| Question | Use |
|---|---|
| Can `Self` meaningfully implement this trait multiple times? | type parameter |
| Is there only one sensible implementation per `Self`? | associated type |
| Does the return type vary across implementations? | associated type (`type Output`) |

Ask: "Is there any reason someone would want two different implementations of this trait for the same type?" If yes, use a type parameter. If no, use an associated type.

For `NumericOps`: there is only one way to compute the sum of an `IntColumn` — it's always `i64`. Associated type.

For `ComparableOps`: an `IntColumn` can reasonably compare against `i64`, `f64`, or another column. Type parameter.

### Why `mean` returns `f64` and not `Self::Item`

Notice that `mean`, `std`, and `median` return `f64` directly, not `Option<Self::Item>`:

```rust
fn mean(&self) -> Option<f64>;
fn std(&self)  -> Option<f64>;
fn median(&self) -> Option<f64>;
```

These operations are mathematically incapable of staying within integer types. The mean of `[1, 2]` is `1.5`. Forcing the result into `i64` would silently truncate. So the return type is hardcoded to `f64` regardless of `Self::Item`. For `FloatColumn` where `Item = f64` this happens to be the same type — the distinction matters only for `IntColumn`.

---

## 3. The Three Ts in `impl`

```rust
impl<T> ComparableOps<T> for TypedColumn<T>
where
    T: Copy + PartialOrd,
```

Three appearances of `T`. Many Rust learners find this confusing. Each one plays a different role:

- **`impl<T>`** — *declares* `T` as a free type variable. This says "the following `impl` is generic over some type `T`." Without this declaration, the compiler wouldn't know what `T` refers to.

- **`ComparableOps<T>`** — *which trait* you're implementing. Since `ComparableOps` takes a type parameter, you must say which instantiation you're implementing. Writing `T` here means "the `T`-flavored version of `ComparableOps`."

- **`TypedColumn<T>`** — *which type* you're implementing the trait for. Again, since `TypedColumn` is generic, you specify which instantiation.

All three are the same `T`, which links them: "for any type `T` that satisfies the bounds, implement `ComparableOps<T>` for `TypedColumn<T>`."

### They don't have to be the same

Separate type variables unlock more expressive impls:

```rust
impl<T, Rhs> ComparableOps<Rhs> for TypedColumn<T>
where
    T: Into<Rhs> + Copy,
    Rhs: PartialOrd + Copy,
{
    fn gt(&self, compare: Rhs) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x.into() > compare)).collect()
    }
}
```

`T` is the storage type, `Rhs` is the comparison type. They can be different — as long as `T` can be converted into `Rhs`. This would let `TypedColumn<i32>` compare against `i64` via `.into()`.

**Practical caveat**: `i64: Into<f64>` does not hold in Rust — the conversion is lossy for large values. So this pattern can't replace the manual `ComparableOps<f64> for IntColumn` impl, which explicitly casts with `x as f64`. The approach works when the conversion is lossless (e.g., `i32` into `i64`).

---

## 4. Blanket Implementations

The `impl<T> ComparableOps<T> for TypedColumn<T>` we saw earlier is a **blanket implementation** — a single `impl` block that covers an entire family of types at once:

```rust
impl<T> ComparableOps<T> for TypedColumn<T>
where
    T: Copy + PartialOrd,
{
    fn gt(&self, compare: T) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x > compare)).collect()
    }

    fn ge(&self, compare: T) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x >= compare)).collect()
    }

    fn lt(&self, compare: T) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x < compare)).collect()
    }

    fn le(&self, compare: T) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x <= compare)).collect()
    }

    fn equal(&self, compare: T) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x == compare)).collect()
    }

    fn between(&self, lower: T, upper: T) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.map(|x| x >= lower && x <= upper))
            .collect()
    }
}
```

This single block gives `ComparableOps<i64>` to `TypedColumn<i64>`, `ComparableOps<f64>` to `TypedColumn<f64>`, `ComparableOps<bool>` to `TypedColumn<bool>`, and so on — for any `T` that is `Copy + PartialOrd`.

The standard library uses blanket impls extensively. `impl<T: Display> Display for &T` means that any reference to a displayable type is itself displayable. `impl<T: Clone> Clone for Vec<T>` means `Vec<i64>` is cloneable because `i64` is cloneable.

### What blanket impls can't cover

Blanket impls work when the implementation logic is **identical** across all `T`. The moment you need type-specific behavior, you need separate impls.

For example, comparing an `IntColumn` against a `&NaiveDate` requires converting the stored `i32` (days since epoch) to a `NaiveDate` and then comparing. That logic is specific to `DateColumn` and can't be expressed as a bound on `T`. It needs its own impl:

```rust
impl ComparableOps<&NaiveDate> for DateColumn {
    fn gt(&self, compare: &NaiveDate) -> Vec<Option<bool>> {
        let cmp = compare.num_days_from_ce();
        self.0.iter().map(|v| v.map(|x| x > cmp)).collect()
    }
    // ...
}
```

### The orphan rule

You can only write a blanket impl if you own either the trait or the type (or both). You cannot write `impl<T> SomeExternalTrait for T` because that would affect every type in the universe, including types from other crates. Rust enforces this to prevent conflicting impls.

---

## 5. Same Type, Multiple Trait Impls

Because `ComparableOps<T>` has `T` as a type parameter, `IntColumn` can implement it any number of times:

```rust
// Compare against the native storage type
impl ComparableOps<i64> for IntColumn {
    fn gt(&self, compare: i64) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x > compare)).collect()
    }
    // ...
}

// Compare against float — needs an explicit cast
impl ComparableOps<f64> for IntColumn {
    fn gt(&self, compare: f64) -> Vec<Option<bool>> {
        self.0
            .iter()
            .map(|v| v.map(|x| (x as f64) > compare))
            .collect()
    }
    // ...
}

// Compare element-wise against another column
impl ComparableOps<&IntColumn> for IntColumn {
    fn gt(&self, compare: &IntColumn) -> Vec<Option<bool>> {
        self.0
            .iter()
            .zip(compare.0.iter())
            .map(|(x, y)| match (x, y) {
                (Some(x), Some(y)) => Some(x > y),
                _ => None,
            })
            .collect()
    }
    // ...
}
```

Three implementations of the same trait for the same type — each handling a different comparison scenario with different logic.

This is **the definitive answer** to the question "when should I put the type parameter on the trait?" When you need the same `Self` to implement the trait multiple ways. If you had used an associated type instead (`type Rhs;`), Rust would only allow one implementation per `Self`, and all three of the above would conflict.

---

## 6. Associated Types for Varying Return Types

So far, every method in `ComparableOps` returns `Vec<Option<bool>>`. But consider string comparisons: when you compare a `DateColumn` against `"2024-01-01"`, the string must be parsed into a date. That parsing can fail — the string might not be a valid date.

The current return type `Vec<Option<bool>>` has no way to signal a parse error. The failure disappears silently: `None` values in the output are indistinguishable from null data rows.

The fix, inspired by Polars' `ChunkCompare` trait, is an associated `Output` type:

```rust
pub trait ComparableOps<T> {
    type Output;
    fn gt(&self, compare: T) -> Self::Output;
    fn ge(&self, compare: T) -> Self::Output;
    fn lt(&self, compare: T) -> Self::Output;
    fn le(&self, compare: T) -> Self::Output;
    fn equal(&self, compare: T) -> Self::Output;
    fn between(&self, lower: T, upper: T) -> Self::Output;
}
```

Now different implementations can declare different output types:

```rust
// Typed impl — comparing against a native type, always succeeds
impl<T: ColumnType> ComparableOps<T::Native> for TypedColumn<T> {
    type Output = Vec<Option<bool>>;

    fn gt(&self, compare: T::Native) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x > compare)).collect()
    }
}

// Type-erased impl — comparing against a string, parsing can fail
impl ComparableOps<&str> for Column {
    type Output = Result<Vec<Option<bool>>, ValidationError>;

    fn gt(&self, compare: &str) -> Result<Vec<Option<bool>>, ValidationError> {
        match self {
            Column::Date(col) => Ok(col.gt(&NaiveDate::from_str(compare)?)),
            Column::DateTime(col) => Ok(col.gt(&NaiveDateTime::from_str(compare)?)),
            Column::Time(col) => Ok(col.gt(&NaiveTime::from_str(compare)?)),
            Column::Str(col) => Ok(col.gt(compare)),
            _ => Ok(vec![None; self.len()]),
        }
    }
}
```

The `?` operator propagates the parse error up. Callers of the string impl know they're getting a `Result` and must handle it. Callers of the typed impl get a plain `Vec<Option<bool>>` with no error handling needed.

### The trade-off

The API is inconsistent: the same method name returns different types depending on which impl you're using. Callers need to know which one they're calling. This is deliberate — Polars accepts exactly this trade-off in its `ChunkCompare` trait, because typed usage and type-erased usage are always distinct callsites.

If you find this uncomfortable, the simpler alternative is to make **all** impls return `Result`, wrapping infallible operations in `Ok(...)`. Consistent API, slight verbosity everywhere.

---

## 7. Marker Types (Phantom Types)

We saw that `DateTimeColumn` and `TimeColumn` both store `i64`. We solved the immediate problem by making them separate newtypes. But newtypes bring back all the boilerplate we were trying to avoid — you lose the blanket impl.

There is a third option: **marker types**.

The idea: give `TypedColumn` a second type parameter that carries no data, only type identity.

```rust
use std::marker::PhantomData;

pub struct TypedColumn<T, Marker>(pub Vec<Option<T>>, PhantomData<Marker>);
```

`PhantomData<Marker>` takes zero bytes at runtime. It exists only to make the type system aware of `Marker`. Now define zero-sized marker structs:

```rust
struct IntMarker;
struct DateTimeMarker;
struct TimeMarker;
struct DateMarker;
struct FloatMarker;
```

And create type aliases using both parameters:

```rust
type IntColumn      = TypedColumn<i64, IntMarker>;
type DateTimeColumn = TypedColumn<i64, DateTimeMarker>;
type TimeColumn     = TypedColumn<i64, TimeMarker>;
type DateColumn     = TypedColumn<i32, DateMarker>;
type FloatColumn    = TypedColumn<f64, FloatMarker>;
```

Now `DateTimeColumn` and `TimeColumn` are distinct types at compile time — `TypedColumn<i64, DateTimeMarker>` vs `TypedColumn<i64, TimeMarker>` — even though they store identical data. They can live in separate enum variants, have separate impl blocks, and be passed to functions that only accept one kind.

The blanket impl becomes:

```rust
impl<T, M> ComparableOps<T> for TypedColumn<T, M>
where
    T: Copy + PartialOrd,
{
    fn gt(&self, compare: T) -> Vec<Option<bool>> {
        self.0.iter().map(|v| v.map(|x| x > compare)).collect()
    }
    // ...
}
```

This covers all column types automatically, regardless of their marker. Adding a new column type costs one zero-sized struct and one type alias — the blanket impls apply immediately.

### How Polars does it

This is precisely the Polars design. Their equivalent of `TypedColumn` is `ChunkedArray<T>`, where `T` is a marker type implementing the `PolarsDataType` trait:

```rust
// In Polars (simplified):
pub struct Int64Type;
pub struct Float64Type;
pub struct Time64MicrosecondType;
pub struct Date32Type;

// All implement PolarsDataType which specifies the underlying native storage
trait PolarsDataType {
    type Native;
}

impl PolarsDataType for Int64Type              { type Native = i64; }
impl PolarsDataType for Time64MicrosecondType  { type Native = i64; }  // same storage
impl PolarsDataType for Date32Type             { type Native = i32; }

// ChunkedArray<Int64Type> and ChunkedArray<Time64MicrosecondType> are distinct types
// even though both store Vec<Option<i64>>
```

The `PolarsDataType` associated type `Native` connects each marker to its storage type. Blanket impls on `ChunkedArray<T: PolarsDataType>` apply to all column types at once, and adding a new column type is a matter of declaring a new marker and implementing `PolarsDataType` for it.

### The cost

The marker type approach is more complex up front. PhantomData is unfamiliar syntax. The connection between a marker and its storage is indirect. For a small system with few types, newtypes may be cleaner. For a system that needs to grow — where adding a new column type should be cheap — the marker pattern pays for itself.

---

## Summary

| Pattern | Use when |
|---|---|
| Type alias | You want a shorter name for a generic type instantiation |
| Newtype | The same storage must represent distinct types with separate impls |
| Associated type | One implementation per `Self`; the return type is fixed |
| Trait type parameter | Multiple implementations per `Self` with different logic |
| Blanket impl | Logic is identical across all `T` meeting the bounds |
| Multiple impls for same type | Naturally follows from trait type parameters |
| `type Output` associated type | Return type legitimately varies across impls |
| Marker types + PhantomData | Same storage, distinct type identity; blanket impls should still apply |
