//! Imports Knots3D bilingual metadata JSON into the configured database.

use std::{env, path::PathBuf, process::ExitCode};

use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_db::{DatabaseConfig, connect_database, repositories::KnotRepository};
use stellartrail_importer::read_knots3d_metadata;
use stellartrail_migration::Migrator;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("import-knots3d failed: {error:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> anyhow::Result<()> {
    let args = Args::parse()?;
    let seeds = read_knots3d_metadata(&args.metadata)?;
    let db = connect_database(&DatabaseConfig::new(args.database_url)?).await?;
    Migrator::up(&db, None).await?;
    KnotRepository::new(db)
        .replace_all_knots("knots3d", &seeds)
        .await?;
    println!("imported {} Knots3D records", seeds.len());
    Ok(())
}

struct Args {
    metadata: PathBuf,
    database_url: String,
}

impl Args {
    fn parse() -> anyhow::Result<Self> {
        let mut metadata = env::var("KNOTS3D_METADATA").ok().map(PathBuf::from);
        let mut database_url = env::var("DATABASE_URL").ok();
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--metadata" => metadata = args.next().map(PathBuf::from),
                "--database-url" => database_url = args.next(),
                "--help" | "-h" => {
                    anyhow::bail!(
                        "usage: import-knots3d --metadata <path> --database-url <sqlite-url>"
                    )
                }
                other => anyhow::bail!("unknown argument: {other}"),
            }
        }
        Ok(Self {
            metadata: metadata.ok_or_else(|| anyhow::anyhow!("--metadata is required"))?,
            database_url: database_url
                .ok_or_else(|| anyhow::anyhow!("--database-url is required"))?,
        })
    }
}
