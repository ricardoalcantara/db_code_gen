use clap::Parser;
use filters::register_filter;
use information_schema::load_database;
use table::Table;
use tera::{Context, Tera};

pub mod cli;
pub mod filters;
pub mod information_schema;
pub mod table;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::ERROR.into())
                .from_env_lossy(),
        )
        .init();

    let args = cli::Cli::parse();
    let config = match args.command {
        cli::Commands::File { path } => {
            let data = std::fs::read_to_string(&path)?;

            if path.ends_with(".json") {
                serde_json::from_str(&data).unwrap()
            } else {
                serde_yaml::from_str(&data).unwrap()
            }
        }
        cli::Commands::Inline(config) => config,
    };

    if let Some(env) = config.dotenv {
        dotenvy::from_path(env).ok();
    }

    let database_url = config
        .database_url
        .or(std::env::var("DATABASE_URL").ok())
        .expect("DATABASE_URL must be set");

    let table_schema = database_url[(database_url.rfind("/").unwrap() + 1)..].to_owned();

    let tables: Vec<Table> = load_database(&database_url, &table_schema).await?;

    let tera = match Tera::new(&format!("{}/**/*", config.template_directory)) {
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

        let mut context = Context::new();
        context.insert("table", table);

        for template in &config.templates {
            let mut split = template.split(":");
            let template_name = split.next().unwrap();
            let suffix = split.next().unwrap_or_default();

            let rendered = tera.render(template_name, &context)?;

            let output_file = if config.render_folder {
                let folder = template_name[..template_name.rfind(".").unwrap()].to_owned();
                format!("{}/{folder}/{}{suffix}", config.output, table.name)
            } else {
                format!("{}/{}{suffix}", config.output, table.name)
            };
            std::fs::write(output_file, rendered).unwrap();
        }
    }

    Ok(())
}
