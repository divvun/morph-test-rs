use anyhow::Result;
use clap::{Parser, ValueEnum};
use colored::control::set_override as set_color_override;
use futures::future::try_join_all;
use morph_test2::backend::{Backend, DEFAULT_TIMEOUT, ExternalBackend};
use morph_test2::engine::run_suites;
use morph_test2::engine_async::run_suites_async;
use morph_test2::pool::PooledBackend;
use morph_test2::report::{OutputKind, print_human};
use morph_test2::spec::{BackendChoice, load_specs};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Compact,
    Terse,
    Final,
    Normal,
}
impl From<OutputFormat> for OutputKind {
    fn from(v: OutputFormat) -> Self {
        match v {
            OutputFormat::Normal => OutputKind::Normal,
            OutputFormat::Compact => OutputKind::Compact,
            OutputFormat::Terse => OutputKind::Terse,
            OutputFormat::Final => OutputKind::Final,
        }
    }
}
#[derive(Parser, Debug, Clone)]
#[command(
    version,
    author,
    about = "Morphological test runner (surface/analyze and lexical/generate)"
)]
struct Cli {
    // TEST_PATHS: ei eller fleire YAML-filer/mapper med testdata
    #[arg(value_name = "TEST_PATHS", required = true)]
    tests: Vec<PathBuf>,

    // Backend-val: standard HFST. Alias for bakoverkompatibilitet: -S / --section
    #[arg(
        long,
        value_enum,
        default_value = "hfst",
        visible_short_alias = 'S',
        visible_alias = "section",
        help = "Vel backend/section (hfst eller foma) [alias: -S/--section]"
    )]
    backend: BackendOpt,

    // Overstyr generator-FST
    #[arg(
        long,
        value_name = "FILE",
        visible_alias = "gen",
        help = "Overstyr generator-FST (.hfstol for HFST, .foma for Foma) [alias: --gen]"
    )]
    generator: Option<String>,

    // Overstyr analyser-FST
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
    // Overstyr lookup-kommandoen
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
        help = "Analyze-testar: godkjenn når alle forventa analysar finst, sjølv om det finst ekstra analysar"
    )]
    ignore_extra_analyses: bool,

    // Fargekontroll
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

    // Verbose
    #[arg(
        short = 'v',
        long = "verbose",
        help = "Vis metadata (lookup med full sti, generator/analyzer med fulle stiar, versjon) og framdriftsmeldingar. Viser òg ‘EXTRA’ for Analyze-PASS når -i er aktiv."
    )]
    verbose: bool,

    // Filtrer retning
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

    // Filtrering av rapportlinjer
    #[arg(
        short = 'f',
        long = "hide-fails",
        conflicts_with = "hide_passes",
        help = "Skjul feil (FAIL), vis berre gjennomgåtte (PASS)"
    )]
    hide_fails: bool,

    #[arg(
        short = 'p',
        long = "hide-passes",
        conflicts_with = "hide_fails",
        help = "Skjul gjennomgåtte (PASS), vis berre feil (FAIL)"
    )]
    hide_passes: bool,

    // -t/--test: tal (1..N), full tittel "Gruppe (Lexical/Generation|Surface/Analysis)" eller berre gruppenamnet.
    // Spesial: 0 / null / liste listar alle testar og avsluttar.
    #[arg(
        short = 't',
        long = "test",
        value_name = "TEST",
        help = "Køyr berre angitt test: nummer 1..N, tittel „Gruppe (Lexical/Generation|Surface/Analysis)” eller berre gruppenamnet frå YAML. Spesial: 0, ‘null’ eller ‘liste’ listar alle tilgjengelege testar og avsluttar."
    )]
    test: Option<String>,

    // NYTT: rapportformat
    #[arg(
        short = 'o',
        long = "output",
        value_enum,
        default_value = "normal",
        help = "Rapportformat: compact | terse | final | normal (standard: normal)"
    )]
    output: OutputFormat,

    // Serial execution (opt-out of default parallel processing)
    #[arg(
        long = "serial",
        help = "Bruk seriell køyring i staden for parallell processing (standardverdi er parallell)"
    )]
    use_serial: bool,
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

fn mode_label(dir: &morph_test2::types::Direction) -> &'static str {
    match dir {
        morph_test2::types::Direction::Generate => "Lexical/Generation",
        morph_test2::types::Direction::Analyze => "Surface/Analysis",
    }
}

fn group_of_case_name(name: &str) -> &str {
    match name.split_once(": ") {
        Some((g, _)) => g,
        None => name,
    }
}

// Bygg blokkliste (1-basert) i encounter-ordning over alle suites
#[derive(Clone)]
struct BlockRef {
    suite_idx: usize,
    group: String,
    dir: morph_test2::types::Direction,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    // Fargar: standard på, --no-color slår av
    if cli.no_color {
        set_color_override(false);
    } else {
        set_color_override(true);
    }

    // Last suites frå teststiar
    let mut suites = load_specs(&cli.tests, cli.backend.into())?;

    // Filtrér retning før vi bygger blokker
    for swc in &mut suites {
        if cli.surface {
            swc.suite
                .cases
                .retain(|c| matches!(c.direction, morph_test2::types::Direction::Analyze));
        } else if cli.lexical {
            swc.suite
                .cases
                .retain(|c| matches!(c.direction, morph_test2::types::Direction::Generate));
        }
    }

    suites.retain(|swc| !swc.suite.cases.is_empty());
    let mut blocks: Vec<BlockRef> = Vec::new();
    for (si, swc) in suites.iter().enumerate() {
        let mut seen: HashSet<(String, morph_test2::types::Direction)> = HashSet::new();
        for c in &swc.suite.cases {
            let g = group_of_case_name(&c.name).to_string();
            let key = (g.clone(), c.direction.clone());
            if seen.insert(key) {
                blocks.push(BlockRef {
                    suite_idx: si,
                    group: g,
                    dir: c.direction.clone(),
                });
            }
        }
    }

    // -t/--test: spesial 0/null/liste => list opp og avslutt
    if let Some(sel) = &cli.test {
        if blocks.is_empty() {
            eprintln!("Ingen testar tilgjengeleg etter filtrering.");
            std::process::exit(2);
        }
        let trimmed = sel.trim();
        if trimmed == "0"
            || trimmed.eq_ignore_ascii_case("null")
            || trimmed.eq_ignore_ascii_case("liste")
        {
            println!("Tilgjengelege testar (1-basert):");
            for (idx, b) in blocks.iter().enumerate() {
                println!("  {}: {} ({})", idx + 1, b.group, mode_label(&b.dir));
            }
            return Ok(());
        }
        // Vel ut blokk(er) etter input: nummer, full tittel eller gruppenamn
        let mut selected: Vec<BlockRef> = Vec::new();
        if let Ok(n) = trimmed.parse::<usize>() {
            if n == 0 || n > blocks.len() {
                eprintln!(
                    "Ugyldig testnummer {}. Gyldig område: 1..{}.",
                    n,
                    blocks.len()
                );
                eprintln!("Tilgjengelege testar (1-basert):");
                for (idx, b) in blocks.iter().enumerate() {
                    eprintln!("  {}: {} ({})", idx + 1, b.group, mode_label(&b.dir));
                }
                std::process::exit(2);
            }
            selected.push(blocks[n - 1].clone());
        } else {
            for b in &blocks {
                let title = format!("{} ({})", b.group, mode_label(&b.dir));
                if title == trimmed {
                    selected.push(b.clone());
                }
            }
            if selected.is_empty() {
                for b in &blocks {
                    if b.group == trimmed {
                        selected.push(b.clone());
                    }
                }
            }
            if selected.is_empty() {
                eprintln!("Fann ikkje test med ID/tittel: {trimmed}");
                eprintln!("Tilgjengelege testar (1-basert):");
                for (idx, b) in blocks.iter().enumerate() {
                    eprintln!("  {}: {} ({})", idx + 1, b.group, mode_label(&b.dir));
                }
                std::process::exit(2);
            }
        }
        // Filtrer suites til berre valde blokker
        for (si, swc) in suites.iter_mut().enumerate() {
            let allowed: Vec<(String, morph_test2::types::Direction)> = selected
                .iter()
                .filter(|b| b.suite_idx == si)
                .map(|b| (b.group.clone(), b.dir.clone()))
                .collect();
            if allowed.is_empty() {
                swc.suite.cases.clear();
                continue;
            }
            swc.suite.cases.retain(|c| {
                allowed
                    .iter()
                    .any(|(g, d)| group_of_case_name(&c.name) == g && &c.direction == d)
            });
        }
        suites.retain(|swc| !swc.suite.cases.is_empty());
    }

    let mut aggregate = morph_test2::types::Summary::default();
    if cli.verbose && !cli.silent {
        println!(
            "[INFO] {} v{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
    }
    if cli.use_serial {
        // Use traditional sequential processing
        process_suites_sequential(suites, &cli, &mut aggregate).await?;
    } else {
        // Use process pool for parallel execution (default)
        process_suites_with_pool(suites, &cli, &mut aggregate).await?;
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

async fn process_suites_sequential(
    suites: Vec<morph_test2::spec::SuiteWithConfig>,
    cli: &Cli,
    aggregate: &mut morph_test2::types::Summary,
) -> Result<()> {
    // Køyr per suite
    for swc in suites {
        // Overstyr frå CLI
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
            println!("[INFO] Lookup tool   : {lookup_full}");
            println!("[INFO] Generator     : {gen_full}");
            println!("[INFO] Analyzer      : {morph_full}");
            println!(
                "[INFO] Startar testing ({} testar, modus: {}) (batch processing)...",
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

        // Validate backend before running tests - fail fast on configuration errors
        if let Err(e) = backend.validate() {
            eprintln!("Feil: {e}");
            std::process::exit(2);
        }

        let summary = run_suites(&backend, &[swc.suite], cli.ignore_extra_analyses);

        if cli.verbose && !cli.silent {
            println!(
                "[INFO] Ferdig: passed {}, failed {}. Skriv rapport...",
                summary.passed, summary.failed
            );
        }

        if !cli.silent {
            print_human(
                &summary,
                cli.ignore_extra_analyses,
                cli.verbose,
                cli.hide_fails,
                cli.hide_passes,
                cli.output.into(),
            );
        }

        aggregate.total += summary.total;
        aggregate.passed += summary.passed;
        aggregate.failed += summary.failed;
        aggregate.total_expectations += summary.total_expectations;
        aggregate.passed_expectations += summary.passed_expectations;
        aggregate.failed_expectations += summary.failed_expectations;
        aggregate.cases.extend(summary.cases);
    }
    Ok(())
}

async fn process_suites_with_pool(
    suites: Vec<morph_test2::spec::SuiteWithConfig>,
    cli: &Cli,
    aggregate: &mut morph_test2::types::Summary,
) -> Result<()> {
    // Group suites by backend configuration to share pools
    let mut backend_groups: HashMap<String, Vec<morph_test2::spec::SuiteWithConfig>> =
        HashMap::new();

    for swc in suites {
        // Create a key based on lookup command and FST files
        let effective_gen = cli.generator.clone().unwrap_or_else(|| swc.gen_fst.clone());
        let effective_morph = cli.analyser.clone().or(swc.morph_fst.clone());
        let effective_lookup = cli
            .lookup_tool
            .clone()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| swc.lookup_cmd.clone());

        let key = format!(
            "{}__{}__{}",
            effective_lookup,
            effective_gen,
            effective_morph.unwrap_or_default()
        );

        backend_groups.entry(key).or_default().push(swc);
    }

    // Process each backend group in parallel
    let group_futures: Vec<_> = backend_groups
        .into_values()
        .map(|group_suites| {
            let cli = cli.clone();
            async move {
                // Create pooled backend for this group
                let first_suite = &group_suites[0];
                let effective_gen = cli
                    .generator
                    .clone()
                    .unwrap_or_else(|| first_suite.gen_fst.clone());
                let effective_morph = cli.analyser.clone().or(first_suite.morph_fst.clone());
                let effective_lookup = cli
                    .lookup_tool
                    .clone()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| first_suite.lookup_cmd.clone());

                let pooled_backend = PooledBackend::new(
                    effective_lookup,
                    effective_morph,
                    Some(effective_gen),
                    cli.silent,
                )
                .await?;

                // Validate backend
                pooled_backend.validate().await?;

                // Process all suites in this group sequentially (they share the same backend)
                let mut group_summaries = Vec::new();
                for swc in group_suites {
                    if cli.verbose && !cli.silent {
                        println!("[INFO] Suite: {} (parallel processing)...", swc.suite.name);
                    }

                    let summary =
                        run_suites_async(&pooled_backend, &[swc.suite], cli.ignore_extra_analyses)
                            .await?;

                    if cli.verbose && !cli.silent {
                        println!(
                            "[INFO] Ferdig: passed {}, failed {}. Skriv rapport...",
                            summary.passed, summary.failed
                        );
                    }

                    if !cli.silent {
                        print_human(
                            &summary,
                            cli.ignore_extra_analyses,
                            cli.verbose,
                            cli.hide_fails,
                            cli.hide_passes,
                            cli.output.into(),
                        );
                    }

                    group_summaries.push(summary);
                }

                Ok::<Vec<morph_test2::types::Summary>, anyhow::Error>(group_summaries)
            }
        })
        .collect();

    // Await all groups and aggregate results
    let all_group_results = try_join_all(group_futures).await?;
    for group_summaries in all_group_results {
        for summary in group_summaries {
            aggregate.total += summary.total;
            aggregate.passed += summary.passed;
            aggregate.failed += summary.failed;
            aggregate.total_expectations += summary.total_expectations;
            aggregate.passed_expectations += summary.passed_expectations;
            aggregate.failed_expectations += summary.failed_expectations;
            aggregate.cases.extend(summary.cases);
        }
    }

    Ok(())
}
