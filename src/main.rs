use anyhow::Result;
use clap::{Parser, ValueEnum};
use colored::control::set_override as set_color_override;
use futures::future::try_join_all;
use morph_test2::backend::{Backend, DEFAULT_TIMEOUT, ExternalBackend};
use morph_test2::engine::run_suites;
use morph_test2::engine_async::run_suites_async;
use morph_test2::i18n;
use morph_test2::pool::PooledBackend;
use morph_test2::report::{OutputKind, print_human};
use morph_test2::spec::{BackendChoice, load_specs};
use morph_test2::{t, t_args};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::{error, info};

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
    about = t!("cli-about")
)]
struct Cli {
    // TEST_PATHS: one or more YAML files/directories with test data
    #[arg(value_name = "TEST_PATHS", required = true)]
    tests: Vec<PathBuf>,

    // Backend choice: default HFST. Alias for backward compatibility: -S / --section
    #[arg(
        long,
        value_enum,
        default_value = "hfst",
        visible_short_alias = 'S',
        visible_alias = "section",
        help = t!("cli-backend")
    )]
    backend: BackendOpt,

    // Override generator FST
    #[arg(
        long,
        value_name = "FILE",
        visible_alias = "gen",
        help = t!("cli-generator")
    )]
    generator: Option<String>,

    // Override analyser FST
    #[arg(
        long,
        value_name = "FILE",
        visible_aliases = ["morph", "analyzer"],
        help = t!("cli-analyser")
    )]
    analyser: Option<String>,

    // Silent mode
    #[arg(
        short = 'q',
        long = "silent",
        help = t!("cli-silent")
    )]
    silent: bool,
    // Override lookup command
    #[arg(
        long = "lookup-tool",
        value_name = "CMD",
        visible_alias = "app",
        help = t!("cli-lookup-tool")
    )]
    lookup_tool: Option<String>,

    // Ignore extra analyses in Analyze mode
    #[arg(
        short = 'i',
        long = "ignore-extra-analyses",
        help = t!("cli-ignore-extra")
    )]
    ignore_extra_analyses: bool,

    // Color control
    #[arg(
        short = 'c',
        long = "color",
        help = t!("cli-color")
    )]
    color: bool,

    #[arg(
        long = "no-color",
        help = t!("cli-no-color")
    )]
    no_color: bool,

    // Verbose
    #[arg(
        short = 'v',
        long = "verbose",
        help = t!("cli-verbose")
    )]
    verbose: bool,

    // Filter direction
    #[arg(
        short = 's',
        long = "surface",
        conflicts_with = "lexical",
        help = t!("cli-surface")
    )]
    surface: bool,

    #[arg(
        short = 'l',
        long = "lexical",
        conflicts_with = "surface",
        help = t!("cli-lexical")
    )]
    lexical: bool,

    // Filtering of report lines
    #[arg(
        short = 'f',
        long = "hide-fails",
        conflicts_with = "hide_passes",
        help = t!("cli-hide-fails")
    )]
    hide_fails: bool,

    #[arg(
        short = 'p',
        long = "hide-passes",
        conflicts_with = "hide_fails",
        help = t!("cli-hide-passes")
    )]
    hide_passes: bool,

    // -t/--test: number (1..N), full title "Group (Lexical/Generation|Surface/Analysis)" or just the group name.
    // Special: 0 / null / list lists all tests and exits.
    #[arg(
        short = 't',
        long = "test",
        value_name = "TEST",
        help = t!("cli-test")
    )]
    test: Option<String>,

    // NEW: report format
    #[arg(
        short = 'o',
        long = "output",
        value_enum,
        default_value = "normal",
        help = t!("cli-output")
    )]
    output: OutputFormat,

    // Serial execution (opt-out of default parallel processing)
    #[arg(
        long = "serial",
        help = t!("cli-serial")
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

fn mode_label(dir: &morph_test2::types::Direction) -> String {
    match dir {
        morph_test2::types::Direction::Generate => t!("direction-generate"),
        morph_test2::types::Direction::Analyze => t!("direction-analyze"),
    }
}

fn group_of_case_name(name: &str) -> &str {
    match name.split_once(": ") {
        Some((g, _)) => g,
        None => name,
    }
}

// Build block list (1-based) in encounter order across all suites
#[derive(Clone)]
struct BlockRef {
    suite_idx: usize,
    group: String,
    dir: morph_test2::types::Direction,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize localization first
    i18n::init();

    let cli = Cli::parse();

    // Initialize tracing based on verbose flag and environment
    let filter = if cli.verbose {
        // With -v, show INFO and above, but allow RUST_LOG to override for debug/trace
        std::env::var("RUST_LOG").unwrap_or_else(|_| "morph_test2=info".to_string())
    } else {
        // Without -v, only show warnings and errors
        std::env::var("RUST_LOG").unwrap_or_else(|_| "morph_test2=warn".to_string())
    };

    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Colors: default on, --no-color turns off
    if cli.no_color {
        set_color_override(false);
    } else {
        set_color_override(true);
    }

    // Load suites from test paths
    let mut suites = load_specs(&cli.tests, cli.backend.into())?;

    // Filter direction before we build blocks
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

    // -t/--test: special 0/null/list => list and exit
    if let Some(sel) = &cli.test {
        if blocks.is_empty() {
            error!("{}", t!("error-no-tests-after-filter"));
            std::process::exit(2);
        }
        let trimmed = sel.trim();
        if trimmed == "0"
            || trimmed.eq_ignore_ascii_case("null")
            || trimmed.eq_ignore_ascii_case("liste")
        {
            println!("{}", t!("available-tests"));
            for (idx, b) in blocks.iter().enumerate() {
                println!(
                    "{}",
                    t_args!("test-list-item",
                        "index" => (idx + 1),
                        "group" => &b.group,
                        "direction" => &mode_label(&b.dir)
                    )
                );
            }
            return Ok(());
        }
        // Select block(s) by input: number, full title or group name
        let mut selected: Vec<BlockRef> = Vec::new();
        if let Ok(n) = trimmed.parse::<usize>() {
            if n == 0 || n > blocks.len() {
                error!(
                    "{}",
                    t_args!("error-invalid-test-number",
                        "number" => n,
                        "max" => blocks.len()
                    )
                );
                eprintln!("{}", t!("available-tests"));
                for (idx, b) in blocks.iter().enumerate() {
                    eprintln!(
                        "{}",
                        t_args!("test-list-item",
                            "index" => (idx + 1),
                            "group" => &b.group,
                            "direction" => &mode_label(&b.dir)
                        )
                    );
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
                error!("{}", t_args!("error-test-not-found", "test" => trimmed));
                eprintln!("{}", t!("available-tests"));
                for (idx, b) in blocks.iter().enumerate() {
                    eprintln!(
                        "{}",
                        t_args!("test-list-item",
                            "index" => (idx + 1),
                            "group" => &b.group,
                            "direction" => &mode_label(&b.dir)
                        )
                    );
                }
                std::process::exit(2);
            }
        }
        // Filter suites to only selected blocks
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
        info!(
            "{}",
            t_args!("info-version",
                "name" => env!("CARGO_PKG_NAME"),
                "version" => env!("CARGO_PKG_VERSION")
            )
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
        info!(
            "{}",
            t_args!("info-all-finished",
                "total" => aggregate.total,
                "passed" => aggregate.passed,
                "failed" => aggregate.failed
            )
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
    // Run per suite
    for swc in suites {
        // Override from CLI
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
                t!("mode-analyze-only")
            } else if cli.lexical {
                t!("mode-generate-only")
            } else {
                t!("mode-all")
            };
            info!("{}", t_args!("info-suite", "name" => &swc.suite.name));
            info!("{}", t_args!("info-lookup-tool", "path" => &lookup_full));
            info!("{}", t_args!("info-generator", "path" => &gen_full));
            info!("{}", t_args!("info-analyzer", "path" => &morph_full));
            info!(
                "{}",
                t_args!("info-starting-tests",
                    "count" => swc.suite.cases.len(),
                    "mode" => &mode_txt
                )
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
            error!("{}", t_args!("error-validation-failed", "error" => e));
            std::process::exit(2);
        }

        let summary = run_suites(&backend, &[swc.suite], cli.ignore_extra_analyses);

        if cli.verbose && !cli.silent {
            info!(
                "{}",
                t_args!("info-finished",
                    "passed" => summary.passed,
                    "failed" => summary.failed
                )
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
                        info!(
                            "{}",
                            t_args!("info-starting-parallel", "name" => &swc.suite.name)
                        );
                    }

                    let summary =
                        run_suites_async(&pooled_backend, &[swc.suite], cli.ignore_extra_analyses)
                            .await?;

                    if cli.verbose && !cli.silent {
                        info!(
                            "{}",
                            t_args!("info-finished",
                                "passed" => summary.passed,
                                "failed" => summary.failed
                            )
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
