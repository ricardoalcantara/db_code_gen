use anyhow::Result;
use serde::Serialize;
use sql_builder::SqlBuilder;
use sqlx::{ConnectOptions, MySqlConnection};
use std::str::FromStr;

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

pub async fn load_database(database_url: String, table_schema: &str) -> Result<Vec<Table>> {
    let options = sqlx::mysql::MySqlConnectOptions::from_str(&database_url)?;

    let mut conn = options.connect().await?;

    let mut tables = Vec::new();

    for table_name in load_table_names(&mut conn, table_schema).await? {
        let keys = get_keys(&mut conn, table_schema, &table_name).await?;
        let columns = load_table_columns(&mut conn, table_schema, &table_name).await?;
        let mut ordinary_columns = Vec::new();
        let mut primary_keys = Vec::new();
        let mut foreign_keys = Vec::new();
        for c in columns.clone().drain(..) {
            if let Some(key) = keys.iter().find(|k| k.0 == c.name) {
                if key.2 == "PRIMARY KEY" {
                    primary_keys.push(c);
                } else if key.2 == "FOREIGN KEY" {
                    foreign_keys.push(c.clone());
                    ordinary_columns.push(c);
                } else {
                    ordinary_columns.push(c);
                }
            } else {
                ordinary_columns.push(c);
            }
        }

        let mut insert_sql_builder = SqlBuilder::insert_into(&table_name);
        let mut select_sql_builder = SqlBuilder::select_from(&table_name);

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

        let mut delete_sql_builder = SqlBuilder::delete_from(&table_name);

        for field in &primary_keys {
            delete_sql_builder.and_where_eq(&field.name, "?");
        }

        let mut delete_sql = delete_sql_builder.sql()?;
        delete_sql.pop();

        let table = Table {
            name: table_name,
            insert_sql,
            select_sql,
            delete_sql,
            primary_keys,
            foreign_keys,
            ordinary_columns,
            columns,
        };

        tables.push(table);
    }

    Ok(tables)
}

async fn load_table_names(conn: &mut MySqlConnection, table_schema: &str) -> Result<Vec<String>> {
    let result = sqlx::query!(
        r"SELECT TABLE_NAME
        FROM INFORMATION_SCHEMA.tables
        WHERE TABLE_SCHEMA = ?",
        table_schema
    )
    .fetch_all(conn)
    .await?;

    Ok(result.into_iter().map(|r| r.TABLE_NAME).collect())
}

async fn load_table_columns(
    conn: &mut MySqlConnection,
    table_schema: &str,
    table_name: &str,
) -> Result<Vec<Column>> {
    let result = sqlx::query!(
        r"SELECT COLUMN_NAME, IS_NULLABLE, ORDINAL_POSITION, COLUMN_TYPE, COLUMN_COMMENT
        FROM INFORMATION_SCHEMA.columns
        WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
        ORDER BY ORDINAL_POSITION ",
        table_schema,
        table_name
    )
    .fetch_all(conn)
    .await?;

    Ok(result
        .into_iter()
        .map(|r| Column {
            name: r
                .COLUMN_NAME
                .expect("I don't know what to do with COLUMN_NAME null"),
            data_type: r.COLUMN_TYPE,
            is_nullable: r.IS_NULLABLE == "YES",
        })
        .collect())
}

async fn get_keys(
    conn: &mut MySqlConnection,
    table_schema: &str,
    table_name: &str,
) -> Result<Vec<(String, String, String)>> {
    let table_constraints = sqlx::query!(
        r"SELECT CONSTRAINT_SCHEMA, CONSTRAINT_NAME, CONSTRAINT_TYPE
        FROM INFORMATION_SCHEMA.table_constraints
        WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? ",
        table_schema,
        table_name
    )
    .fetch_all(&mut *conn)
    .await?;

    let key_column_usage = sqlx::query!(
        r"SELECT COLUMN_NAME, CONSTRAINT_SCHEMA, CONSTRAINT_NAME, ORDINAL_POSITION
        FROM INFORMATION_SCHEMA.key_column_usage
        WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? ",
        table_schema,
        table_name
    )
    .fetch_all(&mut *conn)
    .await?;

    let result = key_column_usage
        .into_iter()
        .map(|c| {
            let constraint_type = table_constraints
                .iter()
                .find(|tc| tc.CONSTRAINT_NAME == c.CONSTRAINT_NAME)
                .expect("I'm sure it shouldn't happen")
                .CONSTRAINT_TYPE
                .clone();

            (
                c.COLUMN_NAME.unwrap(),
                c.CONSTRAINT_NAME.unwrap(),
                constraint_type,
            )
        })
        .collect();

    Ok(result)
}
