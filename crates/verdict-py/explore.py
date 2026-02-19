from verdict_py import Dataset, Column, Constraint, Rule, py_validate

dataset = Dataset(
    headers=["id", "name", "share", "age"],
    columns=[
        Column.integer([1, 2, 3, 4]),
        Column.string(["ann", "clark", "lana", "lex"]),
        Column.floating([20.3, 2.1, 3.9, 40.0]),
        Column.integer([20, None, 30, 40]),
    ],
)


def section(title):
    print(f"\n{'='*50}")
    print(f"  {title}")
    print(f"{'='*50}")


def explore_dataset():
    section("Dataset")
    print(f"repr:              {dataset}")
    print(f"shape:             {dataset.shape()}")
    print(f"get_by_name(id):   {dataset.get_column_by_name('id')}")
    print(f"get_by_name(name): {dataset.get_column_by_name('name')}")
    print(f"get_by_index(0):   {dataset.get_column_by_index(0)}")
    print(f"get_index(id):     {dataset.get_column_index('id')}")
    print(f"missing column:    {dataset.get_column_by_name('nonexistent')}")


def explore_basic_ops():
    section("Column basic ops (age — int with nulls)")
    age = dataset.get_column_by_name("age")
    print(f"repr:              {age}")
    print(f"len:               {age.len()}")
    print(f"is_empty:          {age.is_empty()}")
    print(f"is_null:           {age.is_null()}")
    print(f"null_count:        {age.null_count()}")
    print(f"not_null_count:    {age.not_null_count()}")
    print(f"unique_count:      {age.unique_count()}")
    print(f"duplicates_count:  {age.duplicates_count()}")


def explore_numeric_ops():
    section("Numeric ops (age — int)")
    age = dataset.get_column_by_name("age")
    print(f"sum:               {age.sum()}")
    print(f"mean:              {age.mean()}")
    print(f"min:               {age.min()}")
    print(f"max:               {age.max()}")
    print(f"std:               {age.std()}")
    print(f"median:            {age.median()}")

    section("Numeric ops (share — float)")
    share = dataset.get_column_by_name("share")
    print(f"sum:               {share.sum()}")
    print(f"mean:              {share.mean()}")
    print(f"min:               {share.min()}")
    print(f"max:               {share.max()}")
    print(f"std:               {share.std()}")
    print(f"median:            {share.median()}")


def explore_comparison_ops():
    section("Comparison ops (age — int)")
    age = dataset.get_column_by_name("age")
    print(f"gt(25):            {age.gt(25.0)}")
    print(f"ge(30):            {age.ge(30.0)}")
    print(f"lt(35):            {age.lt(35.0)}")
    print(f"le(30):            {age.le(30.0)}")
    print(f"equal(30):         {age.equal(30.0)}")
    print(f"between(20, 35):   {age.between(20.0, 35.0)}")


def explore_string_ops():
    section("String ops (name — str)")
    name = dataset.get_column_by_name("name")
    print(f"repr:              {name}")
    print(f"equal('clark'):    {name.equal('clark')}")
    print(f"contains('an'):    {name.contains('an')}")
    print(f"starts_with('l'):  {name.starts_with('l')}")
    print(f"ends_with('x'):    {name.ends_with('x')}")
    print(f"matches_regex:     {name.matches_regex('^[a-c]')}")
    print(f"str_length:        {name.str_length()}")
    print(f"is_in:             {name.is_in(['clark', 'ann'])}")


def explore_validation():
    section("Validation")
    rules = [
        Rule("id", Constraint.not_null()),
        Rule("id", Constraint.unique()),
        Rule("age", Constraint.not_null()),
        Rule("age", Constraint.gt(0.0)),
        Rule("age", Constraint.between(18.0, 99.0)),
        Rule("name", Constraint.not_null()),
        Rule("name", Constraint.contains("a")),
        Rule("share", Constraint.gt(0.0)),
        Rule("share", Constraint.between(1.0, 50.0)),
        Rule("name", Constraint.is_in(["ann", "clark", "lana", "lex"])),
        Rule("name", Constraint.starts_with("a")),
        Rule("name", Constraint.matches_regex("^[a-z]+")),
        Rule("name", Constraint.length_between(2, 10)),
    ]
    results = py_validate(dataset, rules)
    for r in results:
        print(r)


if __name__ == "__main__":
    explore_dataset()
    explore_basic_ops()
    explore_numeric_ops()
    explore_comparison_ops()
    explore_string_ops()
    explore_validation()
