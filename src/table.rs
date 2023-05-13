use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
}

#[derive(Serialize)]
pub struct Table {
    pub name: String,
    pub insert_sql: String,
    pub select_sql: String,
    pub delete_sql: String,
    pub primary_keys: Vec<Column>,
    pub foreign_keys: Vec<Column>,
    pub ordinary_columns: Vec<Column>,
    pub columns: Vec<Column>,
}

impl Table {
    pub fn new(name: String, columns: Vec<Column>) -> Table {
        todo!();
    }
}
