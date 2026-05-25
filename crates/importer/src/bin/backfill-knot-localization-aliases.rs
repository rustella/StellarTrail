//! Backfills locale-scoped knot aliases from existing Knots3D raw metadata or a local metadata file.

use std::{env, path::PathBuf, process::ExitCode};

use stellartrail_importer::knot_alias_backfill::{
    AliasBackfillExpectations, AliasBackfillOptions, AliasBackfillSource,
    backfill_knot_localization_aliases,
};

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("backfill-knot-localization-aliases failed: {error:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> anyhow::Result<()> {
    let options = Args::parse()?.into_options();
    let report = backfill_knot_localization_aliases(options).await?;
    for line in report.lines() {
        println!("{line}");
    }
    Ok(())
}

struct Args {
    database_url: String,
    source: AliasBackfillSource,
    dry_run: bool,
    expectations: AliasBackfillExpectations,
}

impl Args {
    fn parse() -> anyhow::Result<Self> {
        let mut database_url = env::var("DATABASE_URL").ok();
        let mut from_raw_db = false;
        let mut metadata = None;
        let mut dry_run = false;
        let mut expectations = AliasBackfillExpectations::default();
        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--database-url" => database_url = args.next(),
                "--from-raw-db" => from_raw_db = true,
                "--metadata" => metadata = args.next().map(PathBuf::from),
                "--dry-run" => dry_run = true,
                "--expected-items" => {
                    expectations.items = parse_i64_arg("--expected-items", args.next())?
                }
                "--expected-localizations" => {
                    expectations.localizations =
                        parse_i64_arg("--expected-localizations", args.next())?
                }
                "--expected-media-resources" => {
                    expectations.media_resources =
                        parse_i64_arg("--expected-media-resources", args.next())?
                }
                "--expected-knot-media-resources" => {
                    expectations.knot_media_resources =
                        parse_i64_arg("--expected-knot-media-resources", args.next())?
                }
                "--help" | "-h" => anyhow::bail!(
                    "usage: backfill-knot-localization-aliases --database-url <url> (--from-raw-db | --metadata <path>) [--dry-run] [--expected-items <n>] [--expected-localizations <n>] [--expected-media-resources <n>] [--expected-knot-media-resources <n>]"
                ),
                other => anyhow::bail!("unknown argument: {other}"),
            }
        }

        let source = match (from_raw_db, metadata) {
            (true, None) => AliasBackfillSource::RawDb,
            (false, Some(path)) => AliasBackfillSource::Metadata(path),
            (true, Some(_)) => anyhow::bail!("choose only one source: --from-raw-db or --metadata"),
            (false, None) => anyhow::bail!("one source is required: --from-raw-db or --metadata"),
        };

        Ok(Self {
            database_url: database_url
                .ok_or_else(|| anyhow::anyhow!("--database-url or DATABASE_URL is required"))?,
            source,
            dry_run,
            expectations,
        })
    }

    fn into_options(self) -> AliasBackfillOptions {
        AliasBackfillOptions {
            database_url: self.database_url,
            source: self.source,
            dry_run: self.dry_run,
            expectations: self.expectations,
        }
    }
}

fn parse_i64_arg(flag: &str, value: Option<String>) -> anyhow::Result<i64> {
    let value = value.ok_or_else(|| anyhow::anyhow!("{flag} requires a value"))?;
    value
        .parse::<i64>()
        .map_err(|err| anyhow::anyhow!("{flag} must be an integer: {err}"))
}
