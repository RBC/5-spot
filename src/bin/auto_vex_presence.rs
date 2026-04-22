// Copyright (c) 2025 Erick Bourgeois, firestoned
// SPDX-License-Identifier: Apache-2.0
//! # Presence-based auto-VEX generator (CI tool, roadmap Phase 2)
//!
//! Reads a Grype JSON report and one or more CycloneDX SBOMs, cross-checks
//! the set of hand-authored statements in `.vex/*.json`, and emits an
//! OpenVEX document containing `not_affected + component_not_present`
//! statements for every Grype finding whose affected `purl` is absent from
//! every SBOM and whose CVE identifier is not already triaged.
//!
//! ## Usage
//!
//! ```bash
//! cargo run --bin auto-vex-presence -- \
//!     --grype-json grype.json \
//!     --sbom docker-sbom-Chainguard.json \
//!     --sbom docker-sbom-Distroless.json \
//!     --vex-dir .vex \
//!     --product-purl pkg:oci/5-spot \
//!     --id "https://github.com/finos/5-spot/actions/runs/123/auto-vex-presence" \
//!     --author auto-vex-presence \
//!     --output vex.auto-presence.json
//! ```
//!
//! All logic lives in [`five_spot::auto_vex_presence`]; this binary is
//! strictly a clap-driven CLI + file I/O wrapper.

use std::collections::HashSet;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use five_spot::auto_vex_presence::{
    build_document, compute_presence_vex, load_triaged_from_vex_dir, GrypeReport, Sbom,
};

#[derive(Parser, Debug)]
#[command(
    name = "auto-vex-presence",
    about = "Emit OpenVEX component_not_present statements for Grype findings absent from SBOMs"
)]
struct Cli {
    /// Grype JSON report (produced by `grype --output json`).
    #[arg(long)]
    grype_json: PathBuf,

    /// One or more CycloneDX SBOM JSON files. A purl present in ANY SBOM
    /// counts as "component present" and excludes the finding.
    #[arg(long, required = true)]
    sbom: Vec<PathBuf>,

    /// Directory of hand-authored `.vex/*.json` statements; CVEs already
    /// covered by one of these are skipped. Missing directory is OK.
    #[arg(long)]
    vex_dir: PathBuf,

    /// Product purl to attach to every emitted statement (e.g.
    /// `pkg:oci/5-spot`).
    #[arg(long)]
    product_purl: String,

    /// Canonical `@id` for the document (typically a URL of the CI run).
    #[arg(long)]
    id: String,

    /// Document-level author (typically `auto-vex-presence`).
    #[arg(long, default_value = "auto-vex-presence")]
    author: String,

    /// RFC-3339 UTC timestamp. Defaults to now() at process start.
    #[arg(long)]
    timestamp: Option<String>,

    /// Output path. Defaults to stdout.
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("auto-vex-presence: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let grype_bytes = std::fs::read(&cli.grype_json).map_err(|e| {
        anyhow::anyhow!(
            "failed to read --grype-json {}: {}",
            cli.grype_json.display(),
            e
        )
    })?;
    let grype: GrypeReport = serde_json::from_slice(&grype_bytes).map_err(|e| {
        anyhow::anyhow!(
            "failed to parse --grype-json {}: {}",
            cli.grype_json.display(),
            e
        )
    })?;

    let mut sboms: Vec<Sbom> = Vec::with_capacity(cli.sbom.len());
    for path in &cli.sbom {
        let bytes = std::fs::read(path)
            .map_err(|e| anyhow::anyhow!("failed to read --sbom {}: {}", path.display(), e))?;
        let sbom: Sbom = serde_json::from_slice(&bytes)
            .map_err(|e| anyhow::anyhow!("failed to parse --sbom {}: {}", path.display(), e))?;
        sboms.push(sbom);
    }

    let triaged: HashSet<String> = load_triaged_from_vex_dir(&cli.vex_dir).map_err(|e| {
        anyhow::anyhow!("failed to load --vex-dir {}: {}", cli.vex_dir.display(), e)
    })?;

    let timestamp = cli
        .timestamp
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());

    let statements = compute_presence_vex(&grype, &sboms, &triaged, &cli.product_purl, &timestamp);
    let doc = build_document(statements, &cli.id, &cli.author, &timestamp);
    let rendered = serde_json::to_string_pretty(&doc)? + "\n";

    match &cli.output {
        Some(path) => std::fs::write(path, &rendered)
            .map_err(|e| anyhow::anyhow!("failed to write --output {}: {}", path.display(), e))?,
        None => print!("{rendered}"),
    }

    eprintln!(
        "auto-vex-presence: {} grype match(es) -> {} statement(s) emitted ({} already triaged in {})",
        grype.matches.len(),
        doc.statements.len(),
        triaged.len(),
        cli.vex_dir.display(),
    );
    Ok(())
}
