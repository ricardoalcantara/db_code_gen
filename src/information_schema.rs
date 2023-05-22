use anyhow::Result;
use sqlx::{ConnectOptions, FromRow, MySqlConnection};
use std::str::FromStr;

use crate::table::{Column, Key, Table};

pub async fn load_database(database_url: &str, table_schema: &str) -> Result<Vec<Table>> {
    let options = sqlx::mysql::MySqlConnectOptions::from_str(database_url)?;

    let mut conn = options.connect().await?;

    let mut tables = Vec::new();

    for table in load_table_names(&mut conn, table_schema).await? {
        let keys = get_keys(&mut conn, table_schema, &table.TABLE_NAME).await?;
        let mut columns = load_table_columns(&mut conn, table_schema, &table.TABLE_NAME).await?;

        for key in keys {
            if let Some(mut column) = columns.iter_mut().find(|c| c.name == key.column_name) {
                if key.constraint_type == "PRIMARY KEY" {
                    column.primary_key = Some(key.constraint_name);
                } else if key.constraint_type == "FOREIGN KEY" {
                    column.foreign_key = Some(key.constraint_name);
                } else if key.constraint_type == "UNIQUE" {
                    column.unique = Some(key.constraint_name);
                }
            }
        }

        let model_table = Table::new(
            table.TABLE_NAME,
            table.AUTO_INCREMENT.unwrap_or_default(),
            columns,
        )?;
        tables.push(model_table);
    }

    Ok(tables)
}

#[derive(FromRow)]
struct Tables {
    TABLE_NAME: String,
    AUTO_INCREMENT: Option<bool>,
}

#[derive(FromRow)]
struct Columns {
    COLUMN_NAME: String,
    COLUMN_TYPE: String,
    IS_NULLABLE: String,
    EXTRA: String,
    // ordinal_position: String,
    // column_comment: String,
}

#[derive(FromRow)]
struct TableConstraints {
    CONSTRAINT_SCHEMA: String,
    CONSTRAINT_NAME: String,
    CONSTRAINT_TYPE: String,
}

#[derive(FromRow)]
struct KeyColumnUsage {
    COLUMN_NAME: String,
    CONSTRAINT_SCHEMA: String,
    CONSTRAINT_NAME: String,
    ORDINAL_POSITION: u32,
}

async fn load_table_names(conn: &mut MySqlConnection, table_schema: &str) -> Result<Vec<Tables>> {
    let result: Vec<Tables> = sqlx::query_as(
        r"SELECT TABLE_NAME, AUTO_INCREMENT
        FROM INFORMATION_SCHEMA.tables
        WHERE TABLE_SCHEMA = ?",
    )
    .bind(table_schema)
    .fetch_all(conn)
    .await?;

    Ok(result)
}

async fn load_table_columns(
    conn: &mut MySqlConnection,
    table_schema: &str,
    table_name: &str,
) -> Result<Vec<Column>> {
    let result: Vec<Columns> = sqlx::query_as(
        r"SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, ORDINAL_POSITION, EXTRA, COLUMN_COMMENT
        FROM INFORMATION_SCHEMA.columns
        WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
        ORDER BY ORDINAL_POSITION",
    )
    .bind(table_schema)
    .bind(table_name)
    .fetch_all(conn)
    .await?;

    Ok(result
        .into_iter()
        .map(|r| Column {
            name: r.COLUMN_NAME,
            data_type: r.COLUMN_TYPE,
            is_auto_increment: r.EXTRA == "auto_increment",
            is_nullable: r.IS_NULLABLE == "YES",
            primary_key: None,
            foreign_key: None,
            unique: None,
        })
        .collect())
}

async fn get_keys(
    conn: &mut MySqlConnection,
    table_schema: &str,
    table_name: &str,
) -> Result<Vec<Key>> {
    let table_constraints: Vec<TableConstraints> = sqlx::query_as(
        r"SELECT CONSTRAINT_SCHEMA, CONSTRAINT_NAME, CONSTRAINT_TYPE
        FROM INFORMATION_SCHEMA.table_constraints
        WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? ",
    )
    .bind(table_schema)
    .bind(table_name)
    .fetch_all(&mut *conn)
    .await?;

    let key_column_usage: Vec<KeyColumnUsage> = sqlx::query_as(
        r"SELECT COLUMN_NAME, CONSTRAINT_SCHEMA, CONSTRAINT_NAME, ORDINAL_POSITION
        FROM INFORMATION_SCHEMA.key_column_usage
        WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? ",
    )
    .bind(table_schema)
    .bind(table_name)
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

            Key {
                column_name: c.COLUMN_NAME,
                constraint_name: c.CONSTRAINT_NAME,
                constraint_type,
            }
        })
        .collect();

    Ok(result)
}
