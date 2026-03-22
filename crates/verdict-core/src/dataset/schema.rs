pub struct Field {
    pub name: String,
    pub dtype: DataType,
    pub format: Option<String>,
}

impl Field {
    pub fn new(name: impl Into<String>, dtype: DataType, format: Option<&str>) -> Self {
        Field {
            name: name.into(),
            dtype,
            format: format.map(String::from),
        }
    }
}

#[derive(Clone)]
pub enum DataType {
    Int,
    Str,
    Float,
    Bool,
    Date,
    DateTime,
}

pub struct Schema {
    pub fields: Vec<Field>,
}

impl Schema {
    pub fn new(fields: Vec<Field>) -> Self {
        Schema { fields }
    }
}
