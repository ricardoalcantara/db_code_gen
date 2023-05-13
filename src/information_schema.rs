use anyhow::Result;
use sqlx::{ConnectOptions, MySqlConnection};
use std::str::FromStr;

use crate::table::{Column, Table};

struct Key {
    column_name: String,
    constraint_name: String,
    constraint_type: String,
}

pub async fn load_database(database_url: String, table_schema: &str) -> Result<Vec<Table>> {
    let options = sqlx::mysql::MySqlConnectOptions::from_str(&database_url)?;

    let mut conn = options.connect().await?;

    let mut tables = Vec::new();

    for table_name in load_table_names(&mut conn, table_schema).await? {
        let keys = get_keys(&mut conn, table_schema, &table_name).await?;
        let mut columns = load_table_columns(&mut conn, table_schema, &table_name).await?;

        for key in keys {
            if let Some(mut column) = columns.iter_mut().find(|c| c.name == key.column_name) {
                if key.constraint_type == "PRIMARY KEY" {
                    column.is_primary_key = true;
                } else if key.constraint_type == "FOREIGN KEY" {
                    column.is_foreign_key = true;
                }
            }
        }

        let table = Table::new(table_name, columns)?;

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
            is_primary_key: false,
            is_foreign_key: false,
        })
        .collect())
}

async fn get_keys(
    conn: &mut MySqlConnection,
    table_schema: &str,
    table_name: &str,
) -> Result<Vec<Key>> {
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

            Key {
                column_name: c.COLUMN_NAME.unwrap(),
                constraint_name: c.CONSTRAINT_NAME.unwrap(),
                constraint_type,
            }
        })
        .collect();

    Ok(result)
}
