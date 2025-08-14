use anyhow::Result;
use clap::{Parser, ValueEnum};
use colored::control::set_override as set_color_override;
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
#[command(
    version,
    author,
    about = "Morphological test runner (surface/analyze and lexical/generate)"
)]
struct Cli {
    #[arg(value_name = "TEST_PATHS", required = true)]
    tests: Vec<PathBuf>,
    #[arg(
        long,
        value_enum,
        default_value = "hfst",
        help = "Vel backend (hfst eller foma)"
    )]
    backend: BackendOpt,
    #[arg(
        long,
        value_name = "FILE",
        visible_alias = "gen",
        help = "Overstyr generator-FST (.hfstol for HFST, .foma for Foma) [alias: --gen]"
    )]
    generator: Option<String>,
    #[arg(long, value_name = "FILE", visible_aliases = ["morph", "analyzer"], help = "Overstyr analyser-FST (.hfstol for HFST, .foma for Foma) [alias: --morph, --analyzer]")]
    analyser: Option<String>,
    #[arg(
        short = 'q',
        long = "silent",
        help = "Stille modus: ingen utskrift, og demp stderr frå lookup"
    )]
    silent: bool,
    #[arg(
        long = "lookup-tool",
        value_name = "CMD",
        visible_alias = "app",
        help = "Overstyr lookup-kommando (t.d. hfst-optimised-lookup, flookup) [alias: --app]"
    )]
    lookup_tool: Option<String>,
    #[arg(
        short = 'i',
        long = "ignore-extra-analyses",
        help = "Analyze-testar: godkjenn når alle forventa analysar finst, sjølv om det finst ekstra analysar"
    )]
    ignore_extra_analyses: bool,
    #[arg(
        short = 'c',
        long = "color",
        help = "Tving fargar på (standard er fargar på)"
    )]
    color: bool,
    #[arg(
        long = "no-color",
        help = "Slå av fargar i rapporten (overstyrer --color)"
    )]
    no_color: bool,
    #[arg(
        short = 'v',
        long = "verbose",
        help = "Vis metadata (lookup med full sti, generator/analyzer med fulle stiar, versjon) og framdriftsmeldingar. Viser òg ‘EXTRA’ for Analyze-PASS når -i er aktiv."
    )]
    verbose: bool,
    #[arg(
        short = 's',
        long = "surface",
        conflicts_with = "lexical",
        help = "Køyr berre analysetestar (surface form → analyses)"
    )]
    surface: bool,
    #[arg(
        short = 'l',
        long = "lexical",
        conflicts_with = "surface",
        help = "Køyr berre genereringstestar (lexical tags → surface forms)"
    )]
    lexical: bool,
}
fn display_path(path: &str) -> String {
    match std::fs::canonicalize(Path::new(path)) {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => path.to_string(),
    }
}
fn resolve_lookup_path(cmd: &str) -> String {
    if cmd.contains(std::path::MAIN_SEPARATOR) || cmd.starts_with("./") || cmd.starts_with(".\\") {
        return display_path(cmd);
    }
    match which::which(cmd) {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => cmd.to_string(),
    }
}
fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.no_color {
        set_color_override(false);
    } else {
        set_color_override(true);
    }
    let suites_with_cfg = load_specs(&cli.tests, cli.backend.into())?;
    let mut aggregate = morph_test::types::Summary::default();
    if cli.verbose && !cli.silent {
        println!(
            "[INFO] {} v{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
    }
    for mut swc in suites_with_cfg {
        // Filtrér cases etter ønskja retning (no finst begge i suite.cases)
        if cli.surface {
            swc.suite
                .cases
                .retain(|c| matches!(c.direction, morph_test::types::Direction::Analyze));
        } else if cli.lexical {
            swc.suite
                .cases
                .retain(|c| matches!(c.direction, morph_test::types::Direction::Generate));
        }
        if swc.suite.cases.is_empty() {
            continue;
        }
        let effective_gen = cli.generator.clone().unwrap_or_else(|| swc.gen_fst.clone());
        let effective_morph = if let Some(m) = &cli.analyser {
            Some(m.clone())
        } else {
            swc.morph_fst.clone()
        };
        let effective_lookup = cli
            .lookup_tool
            .clone()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| swc.lookup_cmd.clone());
        if cli.verbose && !cli.silent {
            let lookup_full = resolve_lookup_path(&effective_lookup);
            let gen_full = display_path(&effective_gen);
            let morph_full = effective_morph
                .as_deref()
                .map(display_path)
                .unwrap_or_else(|| "-".to_string());
            let mode_txt = if cli.surface {
                "Analyze-only"
            } else if cli.lexical {
                "Generate-only"
            } else {
                "All"
            };
            println!("[INFO] Suite         : {}", swc.suite.name);
            println!("[INFO] Lookup tool   : {}", lookup_full);
            println!("[INFO] Generator     : {}", gen_full);
            println!("[INFO] Analyzer      : {}", morph_full);
            println!(
                "[INFO] Startar testing ({} testar, modus: {})...",
                swc.suite.cases.len(),
                mode_txt
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
        if !cli.silent {
            print_human(&summary, cli.ignore_extra_analyses, cli.verbose);
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
    if aggregate.failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}
