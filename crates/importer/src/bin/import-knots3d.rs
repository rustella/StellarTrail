use std::{env, process};

use stellartrail_db::{DatabaseConfig, KnotRepository};
use stellartrail_importer::read_knots3d_metadata;

fn main() {
    if let Err(error) = run() {
        eprintln!("import-knots3d failed: {error:#}");
        process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let args = Args::parse()?;
    let knots = read_knots3d_metadata(&args.metadata)?;
    let config = DatabaseConfig::new(args.database_url)?;
    let repository = KnotRepository::connect(&config)?.migrate()?;
    repository.replace_all_knots(&args.metadata, &knots)?;
    println!("imported {} knots from {}", knots.len(), args.metadata);
    Ok(())
}

struct Args {
    metadata: String,
    database_url: String,
}

impl Args {
    fn parse() -> anyhow::Result<Self> {
        let mut metadata = env::var("KNOTS3D_METADATA_PATH").ok();
        let mut database_url = env::var("DATABASE_URL").ok();
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--metadata" => metadata = args.next(),
                "--database-url" => database_url = args.next(),
                "--help" | "-h" => {
                    println!("usage: import-knots3d --metadata <path> --database-url <sqlite-url>");
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
        })
    }
}
