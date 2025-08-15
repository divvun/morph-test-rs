use crate::types::{CaseResult, Direction, Summary};
use colored::Colorize;
use indexmap::IndexMap;
use std::collections::BTreeSet;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OutputKind {
    Normal,
    Compact,
    Terse,
    Final,
}

fn parse_group(name: &str) -> (&str, &str) {
    match name.split_once(": ") {
        Some((g, rest)) => (g, rest),
        None => (name, ""),
    }
}

fn mode_label(dir: &Direction) -> &'static str {
    match dir {
        Direction::Generate => "Lexical/Generation",
        Direction::Analyze => "Surface/Analysis",
    }
}

fn dash_line(width: usize) -> String {
    "-".repeat(width)
}

fn print_failure_detailed(case: &CaseResult, i: usize, n_cases: usize, expected_item: &str) {
    let width = n_cases.to_string().len();
    println!(
        "[{:>width$}/{:>width$}][{}] {} => {}",
        i,
        n_cases,
        "FAIL".red().bold(),
        case.input,
        expected_item,
        width = width
    );

    if let Some(error) = &case.error {
        println!("         Error: {}", error.red());
    } else {
        let actual_str = if case.actual.is_empty() {
            "<none>".dimmed().to_string()
        } else {
            case.actual.join(", ")
        };
        println!("         Expected: {}", expected_item.green());
        println!("         Got:      {}", actual_str.yellow());
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    group: String,
    dir: Direction,
}

// Bygg blokker (gruppering per (gruppe, retning)) i encounter-ordning
#[allow(clippy::type_complexity)]
fn build_blocks(
    cases: &[CaseResult],
) -> (
    Vec<(String, Direction)>,
    IndexMap<(String, Direction), Vec<&CaseResult>>,
) {
    let mut order: Vec<Key> = Vec::new();
    let mut groups: IndexMap<(String, Direction), Vec<&CaseResult>> = IndexMap::new();
    for c in cases {
        let (group, _) = parse_group(&c.name);
        let key = (group.to_string(), c.direction.clone());
        if !groups.contains_key(&key) {
            order.push(Key {
                group: group.to_string(),
                dir: c.direction.clone(),
            });
            groups.insert(key.clone(), Vec::new());
        }
        groups.get_mut(&key).unwrap().push(c);
    }
    let seq: Vec<(String, Direction)> = order.into_iter().map(|k| (k.group, k.dir)).collect();
    (seq, groups)
}

// Intern: normal-formatet (dagens format)
fn print_human_normal(
    summary: &Summary,
    ignore_extra_analyses: bool,
    verbose: bool,
    hide_fails: bool,
    hide_passes: bool,
) {
    let (seq, groups) = build_blocks(&summary.cases);
    // For kvar blokk (gruppe+retning)
    let mut test_idx = 0usize; // 0-basert nummerering (som Python)
    for key in seq {
        let cases = match groups.get(&key) {
            Some(v) => v,
            None => continue,
        };
        if cases.is_empty() {
            continue;
        }
        // Tittel-linje
        let title = format!("Test {}: {} ({})", test_idx, key.0, mode_label(&key.1));
        let line = dash_line(title.len());
        println!("{line}");
        println!("{title}");
        println!("{line}");
        let n_cases = cases.len();
        let mut passes = 0usize;
        let mut fails = 0usize;
        let mut total_checks = 0usize; // tel berre forventa/placeholder-liner (ikkje EXTRA)
        for (idx, case) in cases.iter().enumerate() {
            let i = idx + 1;
            let exp_set: BTreeSet<&str> = case.expected.iter().map(|s| s.as_str()).collect();
            let act_set: BTreeSet<&str> = case.actual.iter().map(|s| s.as_str()).collect();
            // Når expected er tom, lag ei placeholder-linje
            if case.expected.is_empty() {
                let placeholder = match case.direction {
                    Direction::Generate => "<No lexical/generation>",
                    Direction::Analyze => "<No surface/analysis>",
                };
                let is_pass = is_pass_empty_expected(case, ignore_extra_analyses);
                let _status_str = if is_pass {
                    "PASS".green().bold()
                } else {
                    "FAIL".red().bold()
                };
                let hide_line = (is_pass && hide_passes) || (!is_pass && hide_fails);
                if !hide_line {
                    let width = n_cases.to_string().len();
                    if is_pass {
                        println!(
                            "[{:>width$}/{:>width$}][{}] {} => {}",
                            i,
                            n_cases,
                            "PASS".green().bold(),
                            case.input,
                            placeholder,
                            width = width
                        );
                    } else {
                        print_failure_detailed(case, i, n_cases, placeholder);
                    }
                }
                total_checks += 1;
                if is_pass {
                    passes += 1;
                } else {
                    fails += 1;
                }
                // Ekstra analysar (verbose + ignore) – vis som [EXTRA], men ikkje rekn dei inn i teljinga
                if verbose && ignore_extra_analyses && matches!(case.direction, Direction::Analyze)
                {
                    let extras: Vec<&str> = act_set.difference(&exp_set).cloned().collect();
                    if !extras.is_empty() && !hide_passes {
                        let width = n_cases.to_string().len();
                        for e in extras {
                            println!(
                                "[{:>width$}/{:>width$}][{}] {} => {}",
                                i,
                                n_cases,
                                "EXTRA".yellow().bold(),
                                case.input,
                                e,
                                width = width
                            );
                        }
                    }
                }
                // Ekstra analysar som FAIL når -i IKKJE er aktiv
                if !ignore_extra_analyses && matches!(case.direction, Direction::Analyze) {
                    let extras: Vec<&str> = act_set.difference(&exp_set).cloned().collect();
                    if !extras.is_empty() && !hide_fails {
                        let extras_str = extras.join(", ");
                        print_failure_detailed(
                            case,
                            i,
                            n_cases,
                            &format!("Unexpected results: {}", extras_str),
                        );
                        total_checks += 1;
                        fails += 1;
                    }
                }
                continue;
            }
            // Éi linje per forventa verdi (PASS/FAIL)
            for exp in &case.expected {
                let ok = act_set.contains(exp.as_str());
                let hide_line = (ok && hide_passes) || (!ok && hide_fails);
                if !hide_line {
                    let width = n_cases.to_string().len();
                    if ok {
                        println!(
                            "[{:>width$}/{:>width$}][{}] {} => {}",
                            i,
                            n_cases,
                            "PASS".green().bold(),
                            case.input,
                            exp,
                            width = width
                        );
                    } else {
                        print_failure_detailed(case, i, n_cases, exp);
                    }
                }
                total_checks += 1;
                if ok {
                    passes += 1;
                } else {
                    fails += 1;
                }
            }
            // For Analyze: vis ekstra analysar i verbose når -i er aktiv
            if verbose && ignore_extra_analyses && matches!(case.direction, Direction::Analyze) {
                let extras: Vec<&str> = act_set.difference(&exp_set).cloned().collect();
                if !extras.is_empty() && !hide_passes {
                    let width = n_cases.to_string().len();
                    for e in extras {
                        println!(
                            "[{:>width$}/{:>width$}][{}] {} => {}",
                            i,
                            n_cases,
                            "EXTRA".yellow().bold(),
                            case.input,
                            e,
                            width = width
                        );
                    }
                }
            }
            // For Analyze: vis ekstra analysar som FAIL når -i IKKJE er aktiv
            if !ignore_extra_analyses && matches!(case.direction, Direction::Analyze) {
                let extras: Vec<&str> = act_set.difference(&exp_set).cloned().collect();
                if !extras.is_empty() && !hide_fails {
                    let extras_str = extras.join(", ");
                    print_failure_detailed(
                        case,
                        i,
                        n_cases,
                        &format!("Unexpected results: {}", extras_str),
                    );
                    total_checks += 1;
                    fails += 1;
                }
            }
        }
        println!();
        println!("Test {test_idx} - Passes: {passes}, Fails: {fails}, Total: {total_checks}");
        println!();
        test_idx += 1;
    }
}

// Nytt: compact-format
fn print_human_compact(summary: &Summary, ignore_extra_analyses: bool) {
    let (seq, groups) = build_blocks(&summary.cases);
    let mut total_passes = 0usize;
    let mut total_fails = 0usize;
    let mut total_checks = 0usize;
    let mut test_idx = 0usize; // 0-basert (som Python)
    for key in seq {
        let cases = match groups.get(&key) {
            Some(v) => v,
            None => continue,
        };
        if cases.is_empty() {
            continue;
        }

        let (passes, fails, checks) = calculate_counts(cases, ignore_extra_analyses);

        let status = if fails == 0 {
            "[PASS]".green().bold().to_string()
        } else {
            "[FAIL]".red().bold().to_string()
        };
        println!(
            "{} Test {}: {} ({}) {}/{}/{}",
            status,
            test_idx,
            key.0,
            mode_label(&key.1),
            passes,
            fails,
            checks
        );
        total_passes += passes;
        total_fails += fails;
        total_checks += checks;
        test_idx += 1;
    }
    println!("Total passes: {total_passes}, Total fails: {total_fails}, Total: {total_checks}");
}

// Nytt: terse-format (prikker/utrop for kvar sjekk, éi line per testblokk, og PASS/FAIL til slutt)
fn print_human_terse(summary: &Summary, ignore_extra_analyses: bool) {
    let (seq, groups) = build_blocks(&summary.cases);
    let mut any_fail = false;
    for key in seq {
        let cases = match groups.get(&key) {
            Some(v) => v,
            None => continue,
        };
        if cases.is_empty() {
            println!();
            continue;
        }
        let mut line = String::new();
        for case in cases {
            let act_set: BTreeSet<&str> = case.actual.iter().map(|s| s.as_str()).collect();
            if case.expected.is_empty() {
                let is_pass = is_pass_empty_expected(case, ignore_extra_analyses);
                line.push(if is_pass { '.' } else { '!' });
                if !is_pass {
                    any_fail = true;
                }
                continue;
            }
            for exp in &case.expected {
                let ok = act_set.contains(exp.as_str());
                line.push(if ok { '.' } else { '!' });
                if !ok {
                    any_fail = true;
                }
            }
        }
        println!("{line}");
    }
    println!("{}", if any_fail { "FAIL" } else { "PASS" });
}

// Nytt: final-format (berre totalsamandrag P/F/T)
fn print_human_final(summary: &Summary, ignore_extra_analyses: bool) {
    // Use centralized counting function
    let all_cases: Vec<&CaseResult> = summary.cases.iter().collect();
    let (total_passes, total_fails, total_checks) =
        calculate_counts(&all_cases, ignore_extra_analyses);
    println!("{total_passes}/{total_fails}/{total_checks}");
}

fn is_pass_empty_expected(case: &CaseResult, ignore_extra_analyses: bool) -> bool {
    match case.direction {
        Direction::Analyze if ignore_extra_analyses => true,
        _ => case.actual.is_empty(),
    }
}

// Centralized counting function to ensure consistency across all reporting formats
pub fn calculate_counts(
    cases: &[&CaseResult],
    ignore_extra_analyses: bool,
) -> (usize, usize, usize) {
    let mut total_passes = 0;
    let mut total_fails = 0;
    let mut total_checks = 0;

    for case in cases {
        let act_set: BTreeSet<&str> = case.actual.iter().map(|s| s.as_str()).collect();

        if case.expected.is_empty() {
            let is_pass = is_pass_empty_expected(case, ignore_extra_analyses);
            total_checks += 1;
            if is_pass {
                total_passes += 1;
            } else {
                total_fails += 1;
            }

            // Count extra analyses as failures when not ignoring them
            if !ignore_extra_analyses && matches!(case.direction, Direction::Analyze) {
                let exp_set: BTreeSet<&str> = case.expected.iter().map(|s| s.as_str()).collect();
                let extras: Vec<&str> = act_set.difference(&exp_set).cloned().collect();
                if !extras.is_empty() {
                    total_checks += 1;
                    total_fails += 1;
                }
            }
        } else {
            for exp in &case.expected {
                let ok = act_set.contains(exp.as_str());
                total_checks += 1;
                if ok {
                    total_passes += 1;
                } else {
                    total_fails += 1;
                }
            }

            // Count extra analyses as failures when not ignoring them
            if !ignore_extra_analyses && matches!(case.direction, Direction::Analyze) {
                let exp_set: BTreeSet<&str> = case.expected.iter().map(|s| s.as_str()).collect();
                let extras: Vec<&str> = act_set.difference(&exp_set).cloned().collect();
                if !extras.is_empty() {
                    total_checks += 1;
                    total_fails += 1;
                }
            }
        }
    }

    (total_passes, total_fails, total_checks)
}

// Offentleg API: ruter til riktig format
pub fn print_human(
    summary: &Summary,
    ignore_extra_analyses: bool,
    verbose: bool,
    hide_fails: bool,
    hide_passes: bool,
    output: OutputKind,
) {
    match output {
        OutputKind::Normal => {
            print_human_normal(
                summary,
                ignore_extra_analyses,
                verbose,
                hide_fails,
                hide_passes,
            );
        }
        OutputKind::Compact => {
            // compact bryr seg ikkje om verbose/hide-flagg; skriv testlinjer + totalsamandrag
            print_human_compact(summary, ignore_extra_analyses);
        }
        OutputKind::Terse => {
            // terse: prikker/utrop per sjekk, PASS/FAIL til slutt
            print_human_terse(summary, ignore_extra_analyses);
        }
        OutputKind::Final => {
            // final: berre totalsamandrag P/F/T
            print_human_final(summary, ignore_extra_analyses);
        }
    }
}
