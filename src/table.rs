use serde::Serialize;
use sql_builder::SqlBuilder;

#[derive(Serialize, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
}

#[derive(Serialize)]
pub struct Table {
    pub name: String,

    pub insert_sql: String,
    pub select_sql: String,
    pub delete_sql: String,
    pub where_pk: String,
    pub where_fk: String,
    pub columns: Vec<Column>,

    pub primary_key_columns: Vec<Column>,
    pub foreign_key_columns: Vec<Column>,
    pub ordinary_columns: Vec<Column>,
}

impl Table {
    pub fn new(name: String, columns: Vec<Column>) -> anyhow::Result<Table> {
        let ordinary_columns: Vec<Column> = columns
            .iter()
            .filter(|c| !c.is_primary_key)
            .map(|c| c.clone())
            .collect();
        let primary_key_columns: Vec<Column> = columns
            .iter()
            .filter(|c| c.is_primary_key)
            .map(|c| c.clone())
            .collect();
        let foreign_key_columns: Vec<Column> = columns
            .iter()
            .filter(|c| c.is_foreign_key)
            .map(|c| c.clone())
            .collect();

        let mut insert_sql_builder = SqlBuilder::insert_into(&name);
        let mut select_sql_builder = SqlBuilder::select_from(&name);

        for field in &columns {
            insert_sql_builder.field(&field.name);
            if field.data_type == "binary(16)" {
                select_sql_builder.field(format!("{name} as \"{name}:Uuid\"", name = field.name));
            } else if field.data_type == "tinyint(1)" {
                select_sql_builder.field(format!("{name} as \"{name}:bool\"", name = field.name));
            } else {
                select_sql_builder.field(&field.name);
            }
        }

        let values = &std::iter::repeat("?")
            .take(columns.len())
            .collect::<Vec<&str>>()[..];
        insert_sql_builder.values(values);

        let mut insert_sql = insert_sql_builder.sql()?;
        insert_sql.pop();
        let mut select_sql = select_sql_builder.sql()?;
        select_sql.pop();

        let mut delete_sql_builder = SqlBuilder::delete_from(&name);

        for field in &primary_key_columns {
            delete_sql_builder.and_where_eq(&field.name, "?");
        }

        let mut delete_sql = delete_sql_builder.sql()?;
        delete_sql.pop();

        let where_pk = primary_key_columns
            .iter()
            .map(|c| format!("{} = ?", c.name))
            .collect::<Vec<String>>()
            .join(" AND ");
        let where_fk = primary_key_columns
            .iter()
            .map(|c| format!("{} = ?", c.name))
            .collect::<Vec<String>>()
            .join(" AND ");

        Ok(Table {
            name,
            insert_sql,
            select_sql,
            delete_sql,
            where_pk,
            where_fk,
            columns,
            primary_key_columns,
            foreign_key_columns,
            ordinary_columns,
        })
    }
}
