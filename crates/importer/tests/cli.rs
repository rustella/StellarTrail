use std::process::Command;

#[test]
fn write_subcommand_requires_explicit_write_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_import-gear-atlas-all"))
        .args([
            "write",
            "--url",
            "https://packwizard.com/gear/tent/example",
            "--database-url",
            "sqlite::memory:",
            "--submitter-user-id",
            "import-user",
            "--batch-id",
            "batch-test",
            "--translation-provider",
            "test",
        ])
        .output()
        .expect("run importer");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--write"), "{stderr}");
}

#[test]
fn backfill_localizations_requires_explicit_write_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_import-gear-atlas-all"))
        .args([
            "backfill-localizations",
            "--database-url",
            "sqlite::memory:",
            "--translation-provider",
            "test",
        ])
        .output()
        .expect("run importer");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--write"), "{stderr}");
}

#[test]
fn discover_8264_full_scan_requires_explicit_authorization_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_import-gear-atlas-all"))
        .args([
            "discover",
            "--source",
            "8264",
            "--full-scan",
            "--max-items-per-source",
            "0",
        ])
        .output()
        .expect("run importer");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--ignore-robots 8264"), "{stderr}");
}

#[test]
fn ignore_robots_is_restricted_to_8264() {
    let output = Command::new(env!("CARGO_BIN_EXE_import-gear-atlas-all"))
        .args([
            "dry-run",
            "--url",
            "https://packwizard.com/gear/tent/example",
            "--ignore-robots",
            "packwizard",
            "--max-items-per-source",
            "0",
        ])
        .output()
        .expect("run importer");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("only supported for 8264"), "{stderr}");
}

#[test]
fn discover_8264_list_flags_are_accepted_with_authorization() {
    let output = Command::new(env!("CARGO_BIN_EXE_import-gear-atlas-all"))
        .args([
            "discover",
            "--source",
            "8264",
            "--full-scan",
            "--ignore-robots",
            "8264",
            "--8264-skip-id-fallback",
            "--8264-list-max-pages-per-scope",
            "1",
            "--max-items-per-source",
            "0",
        ])
        .output()
        .expect("run importer");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"source\": \"8264\""), "{stdout}");
}

#[test]
fn dry_run_accepts_discovery_jsonl_without_fetching_when_limit_is_zero() {
    let dir = tempfile::tempdir().expect("tempdir");
    let discovery = dir.path().join("discovery.jsonl");
    std::fs::write(
        &discovery,
        r#"{"source":"packwizard","url":"https://packwizard.com/gear/tent/example","discovery_method":"fixture","discovered_at":"2026-06-10T00:00:00Z"}"#,
    )
    .expect("write discovery");

    let output = Command::new(env!("CARGO_BIN_EXE_import-gear-atlas-all"))
        .args([
            "dry-run",
            "--from-discovery-file",
            discovery.to_str().expect("path"),
            "--max-items-per-source",
            "0",
        ])
        .output()
        .expect("run importer");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "[]");
}
