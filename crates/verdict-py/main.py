from verdict_py import PyDataset, PyColumn


dataset = PyDataset(
    headers=["id", "name", "share", "age"],
    columns=[
        PyColumn.integer([1, 2, 3, 4]),
        PyColumn.string(["ann", "clark", "lana", "lex"]),
        PyColumn.floating([20.3, 2.1, 3.9, 40.0]),
        PyColumn.integer([20, None, 30, 40]),
    ],
)
print(dataset)

print(dataset.shape())

print(dataset.get_column_by_name("id"))
print(dataset.get_column_by_name("name"))
print(dataset.get_column_by_index(0))
print(dataset.get_column_index("id"))


print(dataset.get_column_by_name("age"))
print(dataset.get_column_by_name("age").is_null())

# Column basic ops
age = dataset.get_column_by_name("age")
print(f"\n=== Basic ops on age column ===")
print(f"len: {age.len()}")
print(f"is_empty: {age.is_empty()}")
print(f"null_count: {age.null_count()}")
print(f"not_null_count: {age.not_null_count()}")
print(f"unique_count: {age.unique_count()}")
print(f"duplicates_count: {age.duplicates_count()}")

# Numeric ops
print(f"\n=== Numeric ops on age column ===")
print(f"sum: {age.sum()}")
print(f"mean: {age.mean()}")
print(f"min: {age.min()}")
print(f"max: {age.max()}")
print(f"std: {age.std()}")
print(f"median: {age.median()}")

# Comparison ops
print(f"\n=== Comparison ops on age column ===")
print(f"gt(25): {age.gt(25.0)}")
print(f"ge(30): {age.ge(30.0)}")
print(f"lt(35): {age.lt(35.0)}")
print(f"le(30): {age.le(30.0)}")
print(f"equal(30): {age.equal(30.0)}")
print(f"between(20, 35): {age.between(20.0, 35.0)}")

# Float ops
share = dataset.get_column_by_name("share")
print(f"\n=== Numeric ops on share (float) column ===")
print(f"sum: {share.sum()}")
print(f"mean: {share.mean()}")
print(f"min: {share.min()}")
print(f"max: {share.max()}")

# String ops
name = dataset.get_column_by_name("name")
print(f"\n=== String ops on name column ===")
print(f"equal_str('clark'): {name.equal_str('clark')}")
print(f"contains('an'): {name.contains('an')}")
print(f"starts_with('l'): {name.starts_with('l')}")
print(f"ends_with('x'): {name.ends_with('x')}")
print(f"matches_regex('^[a-c]'): {name.matches_regex('^[a-c]')}")
print(f"str_length: {name.str_length()}")


print(name.is_in(["clark", "ann"]))
