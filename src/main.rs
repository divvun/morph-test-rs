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
    Xerox,
}
impl From<BackendOpt> for BackendChoice {
    fn from(v: BackendOpt) -> Self {
        match v {
            BackendOpt::Auto => BackendChoice::Auto,
            BackendOpt::Hfst => BackendChoice::Hfst,
            BackendOpt::Xerox => BackendChoice::Xerox,
        }
    }
}
#[derive(Parser, Debug)]
#[command(version, author, about = "Morphological generator test runner (Rust)")]
struct Cli {
    #[arg(value_name = "TEST_PATHS", required = true)]
    tests: Vec<PathBuf>,
    #[arg(
        long,
        value_enum,
        default_value = "auto",
        help = "Vel backend når begge finst i Config"
    )]
    backend: BackendOpt,
}
fn main() -> Result<()> {
    // Rayon brukar all CPU-kjernar som standard (maks parallellitet).
    let cli = Cli::parse();
    let suites_with_cfg = load_specs(&cli.tests, cli.backend.into())?;
    let mut aggregate = morph_test::types::Summary::default();
    for swc in suites_with_cfg {
        let backend = ExternalBackend {
            lookup_cmd: swc.lookup_cmd.clone(),
            generator_fst: Some(swc.gen_fst.clone()),
            analyzer_fst: swc.morph_fst.clone(),
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
