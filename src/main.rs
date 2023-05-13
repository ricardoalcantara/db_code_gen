use dotenvy::dotenv;
use filters::register_filter;
use information_schema::load_database;
use table::Table;
use tera::{Context, Tera};

pub mod filters;
pub mod information_schema;
pub mod table;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::ERROR.into())
                .from_env_lossy(),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let table_schema = "platform_application";

    let tables: Vec<Table> = load_database(database_url, table_schema).await?;

    let tera = match Tera::new("templates/**/*") {
        Ok(mut tera) => {
            register_filter(&mut tera);
            tera
        }
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    for table in tables.iter() {
        if table.name == "_sqlx_migrations" {
            continue;
        }
        println!("Table: {}", table.name);

        let mut context = Context::new();
        context.insert("table", table);

        let output = tera.render("model.tera", &context)?;

        std::fs::write(
            format!(
                "E:/Projects/plataforma-rs/platform-application/lib/repository/src/models/{}_gen.rs",
                table.name
            ),
            output,
        )
        .unwrap();

        let output = tera.render("repository.tera", &context)?;

        std::fs::write(
            format!(
                "E:/Projects/plataforma-rs/platform-application/lib/repository/src/repositories/{}_repository_gen.rs",
                table.name
            ),
            output,
        )
        .unwrap();
    }

    Ok(())
}

// async fn load_table_names() {
//     information_schema.tables (table_schema, table_name) {
//             table_schema -> VarChar,
//             table_name -> VarChar,
//             table_comment -> VarChar,
//         }
// }

// async fn load_table_data(conn: &mut MysqlConnection, table: &TableName,) {

//     information_schema.columns (table_schema, table_name, column_name) {
//             table_schema -> VarChar,
//             table_name -> VarChar,
//             column_name -> VarChar,
//             #[sql_name = "is_nullable"]
//             __is_nullable -> VarChar,
//             ordinal_position -> BigInt,
//             udt_name -> VarChar,
//             udt_schema -> VarChar,
//             column_type -> VarChar,
//             column_comment -> VarChar,
//         }
// }

// async fn get_primary_keys(
//     conn: &mut InferConnection,
//     table: &TableName,
// ) {
//     table! {
//         information_schema.key_column_usage (table_schema, table_name, column_name, constraint_name) {
//             table_schema -> VarChar,
//             table_name -> VarChar,
//             column_name -> VarChar,
//             constraint_schema -> VarChar,
//             constraint_name -> VarChar,
//             ordinal_position -> BigInt,
//         }
//     }
// }

// async fn load_fk() {
//     table! {
//         information_schema.table_constraints (constraint_schema, constraint_name) {
//             table_schema -> VarChar,
//             constraint_schema -> VarChar,
//             constraint_name -> VarChar,
//             constraint_type -> VarChar,
//         }
//     }
// }
