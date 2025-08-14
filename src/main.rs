use anyhow::Result;
use clap::{Parser, ValueEnum};
use morph_test::backend::{ExternalBackend, DEFAULT_TIMEOUT};
use morph_test::engine::run_suites;
use morph_test::report::print_human;
use morph_test::spec::{load_specs, BackendChoice};
use std::path::{Path, PathBuf};
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
    // --generator og synonymet --gen
    #[arg(
        long,
        value_name = "FILE",
        visible_alias = "gen",
        help = "Overstyr generator-FST (.hfstol for HFST, .foma for Foma) [alias: --gen]"
    )]
    generator: Option<String>,
    // --analyser og synonyma --morph og --analyzer
    #[arg(
        long,
        value_name = "FILE",
        visible_aliases = ["morph", "analyzer"],
        help = "Overstyr analyser-FST (.hfstol for HFST, .foma for Foma) [alias: --morph, --analyzer]"
    )]
    analyser: Option<String>,
    // Stille-modus
    #[arg(
        short = 'q',
        long = "silent",
        help = "Stille modus: ingen utskrift, og demp stderr frå lookup"
    )]
    silent: bool,
    // Overstyr lookup-kommandoen (alias --app for YAML-kompat)
    #[arg(
        long = "lookup-tool",
        value_name = "CMD",
        visible_alias = "app",
        help = "Overstyr lookup-kommando (t.d. hfst-optimised-lookup, flookup) [alias: --app]"
    )]
    lookup_tool: Option<String>,
    // Ignorer ekstra analysar i Analyze-modus
    #[arg(
        short = 'i',
        long = "ignore-extra-analyses",
        help = "I Analyze-testar: ignorer ekstra analysar (godkjenn dersom alle forventa analysar finst)"
    )]
    ignore_extra_analyses: bool,
    // Verbose-modus: metadata og framdriftsmeldingar
    #[arg(
        short = 'v',
        long = "verbose",
        help = "Vis metadata (lookup, generator, analyser, versjon) og korte framdriftsmeldingar"
    )]
    verbose: bool,
}
fn display_path(path: &str) -> String {
    match std::fs::canonicalize(Path::new(path)) {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => path.to_string(),
    }
}
fn resolve_lookup_path(cmd: &str) -> String {
    // Dersom cmd allereie er ein sti (inneheld skråstrek), prøv å canonicalize.
    if cmd.contains(std::path::MAIN_SEPARATOR) || cmd.starts_with("./") || cmd.starts_with(".\\") {
        return display_path(cmd);
    }
    // Elles, prøv å slå opp i PATH
    match which::which(cmd) {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => cmd.to_string(), // fallback til oppgitt streng
    }
}
fn main() -> Result<()> {
    // Rayon brukar all CPU-kjernar som standard (maks parallellitet).
    let cli = Cli::parse();
    let suites_with_cfg = load_specs(&cli.tests, cli.backend.into())?;
    let mut aggregate = morph_test::types::Summary::default();
    // Print global metadata (versjon) ved verbose, om ikkje i silent-modus
    if cli.verbose && !cli.silent {
        println!(
            "[INFO] {} v{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
    }
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
        // Overstyr lookup-verktøy dersom oppgitt
        let effective_lookup = if let Some(tool) = &cli.lookup_tool {
            tool.trim().to_string()
        } else {
            swc.lookup_cmd.clone()
        };
        // Lag fulle stiar for utskrift
        let lookup_full = resolve_lookup_path(&effective_lookup);
        let gen_full = display_path(&effective_gen);
        let morph_full = effective_morph
            .as_deref()
            .map(display_path)
            .unwrap_or_else(|| "-".to_string());
        if cli.verbose && !cli.silent {
            println!("[INFO] Suite         : {}", swc.suite.name);
            println!("[INFO] Lookup tool   : {}", lookup_full);
            println!("[INFO] Generator     : {}", gen_full);
            println!("[INFO] Analyzer      : {}", morph_full);
            println!(
                "[INFO] Startar testing ({} testar)...",
                swc.suite.cases.len()
            );
        }
        let backend = ExternalBackend {
            lookup_cmd: effective_lookup,
            generator_fst: Some(effective_gen),
            analyzer_fst: effective_morph,
            timeout: Some(DEFAULT_TIMEOUT),
            quiet: cli.silent,
        };
        let summary = run_suites(&backend, &[swc.suite], cli.ignore_extra_analyses);
        if cli.verbose && !cli.silent {
            println!(
                "[INFO] Ferdig: passed {}, failed {}. Skriv rapport...",
                summary.passed, summary.failed
            );
        }
        // Berre skriv rapport dersom ikkje stille modus
        if !cli.silent {
            print_human(&summary, cli.ignore_extra_analyses);
        }
        aggregate.total += summary.total;
        aggregate.passed += summary.passed;
        aggregate.failed += summary.failed;
        aggregate.cases.extend(summary.cases);
    }
    if cli.verbose && !cli.silent {
        println!(
            "[INFO] Alle testkøyringar ferdige. Total: {}, Passed: {}, Failed: {}",
            aggregate.total, aggregate.passed, aggregate.failed
        );
    }
    // Exit-kode: 0 når alt OK, elles ≠ 0
    if aggregate.failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}
