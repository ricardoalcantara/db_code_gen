use serde::Serialize;
use sql_builder::{baname, SqlBuilder, SqlName};

#[derive(Serialize, Clone, Debug)]
pub struct Key {
    pub column_name: String,
    pub constraint_name: String,
    pub constraint_type: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_auto_increment: bool,
    pub primary_key: Option<String>,
    pub unique: Option<String>,
    pub foreign_key: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct Table {
    pub name: String,
    pub is_auto_increment: bool,

    pub insert_sql: String,
    pub insert_sql_values: String,
    pub select_sql: String,
    pub delete_sql: String,
    pub where_pk: String,
    pub where_fk: String,
    pub columns: Vec<Column>,

    pub primary_key_columns: Vec<Column>,
    pub foreign_key_columns: Vec<Column>,
    pub unique_columns: Vec<Column>,
    pub ordinary_columns: Vec<Column>,
}

impl Table {
    pub fn new(
        name: String,
        is_auto_increment: bool,
        columns: Vec<Column>,
    ) -> anyhow::Result<Table> {
        let ordinary_columns: Vec<Column> = columns
            .iter()
            .filter(|c| !c.primary_key.is_some())
            .map(|c| c.clone())
            .collect();
        let primary_key_columns: Vec<Column> = columns
            .iter()
            .filter(|c| c.primary_key.is_some())
            .map(|c| c.clone())
            .collect();
        let foreign_key_columns: Vec<Column> = columns
            .iter()
            .filter(|c| c.foreign_key.is_some())
            .map(|c| c.clone())
            .collect();

        let unique_columns: Vec<Column> = columns
            .iter()
            .filter(|c| c.unique.is_some())
            .map(|c| c.clone())
            .collect();

        let mut insert_sql_builder = SqlBuilder::insert_into(baname!(&name));
        let mut select_sql_builder = SqlBuilder::select_from(baname!(&name));

        let mut field_count = 0;
        for field in &columns {
            if !(field.primary_key.is_some() && field.is_auto_increment) {
                field_count += 1;
                insert_sql_builder.field(baname!(&field.name));
            }
            if field.data_type == "binary(16)" {
                select_sql_builder
                    .field(format!("`{name}` as \"{name}:Uuid\"", name = &field.name));
            } else if field.data_type == "tinyint(1)" {
                select_sql_builder
                    .field(format!("`{name}` as \"{name}:bool\"", name = &field.name));
            } else {
                select_sql_builder.field(baname!(&field.name));
            }
        }

        let values = &std::iter::repeat("?")
            .take(field_count)
            .collect::<Vec<&str>>()[..];
        let insert_sql_values = format!("({})", values.join(", "));
        insert_sql_builder.values(&values);

        let insert_sql = insert_sql_builder.sql()?;
        let insert_sql = insert_sql[..insert_sql.rfind(" VALUES").unwrap()].to_owned();

        let mut select_sql = select_sql_builder.sql()?;
        select_sql.pop();

        let mut delete_sql_builder = SqlBuilder::delete_from(baname!(&name));

        for field in &primary_key_columns {
            delete_sql_builder.and_where_eq(&field.name, "?");
        }

        let mut delete_sql = delete_sql_builder.sql()?;
        delete_sql.pop();

        let where_pk = primary_key_columns
            .iter()
            .map(|c| format!("{} = ?", baname!(&c.name)))
            .collect::<Vec<String>>()
            .join(" AND ");
        let where_fk = primary_key_columns
            .iter()
            .map(|c| format!("{} = ?", baname!(&c.name)))
            .collect::<Vec<String>>()
            .join(" AND ");

        Ok(Table {
            name,
            is_auto_increment,
            insert_sql,
            insert_sql_values,
            select_sql,
            delete_sql,
            where_pk,
            where_fk,
            columns,
            primary_key_columns,
            foreign_key_columns,
            unique_columns,
            ordinary_columns,
        })
    }
}
