use std::{env, process};

use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_db::{DatabaseConfig, connect_database, repositories::KnotRepository};
use stellartrail_importer::read_knots3d_metadata;
use stellartrail_migration::Migrator;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("import-knots3d failed: {error:#}");
        process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let args = Args::parse()?;
    let knots = read_knots3d_metadata(&args.metadata)?;
    let config = DatabaseConfig::new(args.database_url)?;
    let db = connect_database(&config).await?;
    Migrator::up(&db, None).await?;
    let repository = KnotRepository::new(db, args.media_base_url);
    repository.replace_all_knots(&args.metadata, &knots).await?;
    println!("imported {} knots from {}", knots.len(), args.metadata);
    Ok(())
}

struct Args {
    metadata: String,
    database_url: String,
    media_base_url: String,
}

impl Args {
    fn parse() -> anyhow::Result<Self> {
        let mut metadata = env::var("KNOTS3D_METADATA_PATH").ok();
        let mut database_url = env::var("DATABASE_URL").ok();
        let mut media_base_url = env::var("MEDIA_BASE_URL").ok();
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--metadata" => metadata = args.next(),
                "--database-url" => database_url = args.next(),
                "--media-base-url" => media_base_url = args.next(),
                "--help" | "-h" => {
                    println!(
                        "usage: import-knots3d --metadata <path> --database-url <sqlite-url> [--media-base-url /assets]"
                    );
                    process::exit(0);
                }
                other => anyhow::bail!("unknown argument: {other}"),
            }
        }
        Ok(Self {
            metadata: metadata.ok_or_else(|| {
                anyhow::anyhow!("--metadata or KNOTS3D_METADATA_PATH is required")
            })?,
            database_url: database_url.unwrap_or_else(|| "sqlite://stellartrail.db".to_owned()),
            media_base_url: media_base_url.unwrap_or_else(|| "/assets".to_owned()),
        })
    }
}
