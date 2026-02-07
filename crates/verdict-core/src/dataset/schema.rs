pub struct Field {
    pub name: String,
    pub dtype: DataType,
}

impl Field {
    pub fn new(name: impl Into<String>, dtype: DataType) -> Self {
        Field {
            name: name.into(),
            dtype,
        }
    }
}

pub enum DataType {
    Int,
    Str,
    Float,
    Bool,
}

pub struct Schema {
    pub fields: Vec<Field>,
}

impl Schema {
    pub fn new(fields: Vec<Field>) -> Self {
        Schema { fields }
    }
}
