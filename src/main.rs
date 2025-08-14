use anyhow::Result;
use clap::{Parser, ValueEnum};
use morph_test::backend::{ExternalBackend, DEFAULT_TIMEOUT};
use morph_test::engine::run_suites;
use morph_test::report::print_human;
use morph_test::spec::{load_specs, BackendChoice};
use std::path::PathBuf;
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum BackendOpt {
    Auto,
    Hfst,
    Foma,
}
impl From<BackendOpt> for BackendChoice {
    fn from(v: BackendOpt) -> Self {
        match v {
            BackendOpt::Auto => BackendChoice::Auto,
            BackendOpt::Hfst => BackendChoice::Hfst,
            BackendOpt::Foma => BackendChoice::Foma,
        }
    }
}
#[derive(Parser, Debug)]
#[command(version, author, about = "Morphological generator test runner (Rust)")]
struct Cli {
    #[arg(value_name = "TEST_PATHS", required = true)]
    tests: Vec<PathBuf>,
    // Standard: HFST med hfst-optimised-lookup
    #[arg(
        long,
        value_enum,
        default_value = "hfst",
        help = "Vel backend (hfst eller foma)"
    )]
    backend: BackendOpt,
    #[arg(
        long,
        alias = "gen",
        value_name = "FILE",
        help = "Overstyr generator-FST (.hfstol for HFST, .foma for Foma)"
    )]
    generator: Option<String>,
    #[arg(long, aliases = ["analyzer", "morph"], value_name = "FILE", help = "Overstyr analyser-FST (.hfstol for HFST, .foma for Foma)")]
    analyser: Option<String>,
}
fn main() -> Result<()> {
    // Rayon brukar all CPU-kjernar som standard (maks parallellitet).
    let cli = Cli::parse();
    let suites_with_cfg = load_specs(&cli.tests, cli.backend.into())?;
    let mut aggregate = morph_test::types::Summary::default();
    for swc in suites_with_cfg {
        // Overstyr generator/analyser frå CLI dersom oppgitt
        let effective_gen = if let Some(gen) = &cli.generator {
            gen.clone()
        } else {
            swc.gen_fst.clone()
        };
        let effective_morph = if let Some(morph) = &cli.analyser {
            Some(morph.clone())
        } else {
            swc.morph_fst.clone()
        };
        let backend = ExternalBackend {
            lookup_cmd: swc.lookup_cmd.clone(),
            generator_fst: Some(effective_gen),
            analyzer_fst: effective_morph,
            timeout: Some(DEFAULT_TIMEOUT),
        };
        let summary = run_suites(&backend, &[swc.suite]);
        // Print per-fil for kontekst
        print_human(&summary);
        aggregate.total += summary.total;
        aggregate.passed += summary.passed;
        aggregate.failed += summary.failed;
        aggregate.cases.extend(summary.cases);
    }
    // Exit-kode: 0 når alt OK, elles ≠ 0
    if aggregate.failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}
