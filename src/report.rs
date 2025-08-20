use crate::types::{CaseResult, Direction, Summary};
use crate::{t, t_args};
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

fn mode_label(dir: &Direction) -> String {
    match dir {
        Direction::Generate => t!("direction-generate"),
        Direction::Analyze => t!("direction-analyze"),
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
        println!("         {}: {}", t!("report-error"), error.red());
    } else {
        let actual_str = if case.actual.is_empty() {
            "<none>".dimmed().to_string()
        } else {
            case.actual.join(", ")
        };
        println!(
            "         {}: {}",
            t!("report-expected"),
            expected_item.green()
        );
        println!("         {}: {}", t!("report-got"), actual_str.yellow());
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    group: String,
    dir: Direction,
}

// Build blocks (grouping per (group, direction)) in encounter order
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

// Internal: normal format (current format)
fn print_human_normal(
    summary: &Summary,
    ignore_extra_analyses: bool,
    verbose: bool,
    hide_fails: bool,
    hide_passes: bool,
) {
    let (seq, groups) = build_blocks(&summary.cases);
    // For each block (group+direction)
    let mut test_idx = 1usize; // 1-based numbering
    for key in seq {
        let cases = match groups.get(&key) {
            Some(v) => v,
            None => continue,
        };
        if cases.is_empty() {
            continue;
        }
        // Title line
        let title = t_args!("report-test-header",
            "index" => test_idx,
            "group" => &key.0,
            "direction" => &mode_label(&key.1)
        );
        let line = dash_line(title.len());
        println!("{line}");
        println!("{title}");
        println!("{line}");
        let n_cases = cases.len();
        let mut passes = 0usize;
        let mut fails = 0usize;
        let mut total_checks = 0usize; // count only expected/placeholder lines (not EXTRA)
        for (idx, case) in cases.iter().enumerate() {
            let i = idx + 1;
            let exp_set: BTreeSet<&str> = case.expected.iter().map(|s| s.as_str()).collect();
            let act_set: BTreeSet<&str> = case.actual.iter().map(|s| s.as_str()).collect();
            // When expected is empty, create a placeholder line
            if case.expected.is_empty() {
                let placeholder = match case.direction {
                    Direction::Generate => t!("report-no-lexical"),
                    Direction::Analyze => t!("report-no-surface"),
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
                        print_failure_detailed(case, i, n_cases, &placeholder);
                    }
                }
                total_checks += 1;
                if is_pass {
                    passes += 1;
                } else {
                    fails += 1;
                }
                // Extra analyses (verbose + ignore) – show as [EXTRA], but don't count them in totals
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
                // Extra analyses as FAIL when -i is NOT active
                if !ignore_extra_analyses && matches!(case.direction, Direction::Analyze) {
                    let extras: Vec<&str> = act_set.difference(&exp_set).cloned().collect();
                    if !extras.is_empty() && !hide_fails {
                        let extras_str = extras.join(", ");
                        print_failure_detailed(
                            case,
                            i,
                            n_cases,
                            &t_args!("report-unexpected-results", "results" => &extras_str),
                        );
                        total_checks += 1;
                        fails += 1;
                    }
                }
                continue;
            }
            // One line per expected value (PASS/FAIL)
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
            // For Analyze: show extra analyses in verbose when -i is active
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
            // For Analyze: show extra analyses as FAIL when -i is NOT active
            if !ignore_extra_analyses && matches!(case.direction, Direction::Analyze) {
                let extras: Vec<&str> = act_set.difference(&exp_set).cloned().collect();
                if !extras.is_empty() && !hide_fails {
                    let extras_str = extras.join(", ");
                    print_failure_detailed(
                        case,
                        i,
                        n_cases,
                        &t_args!("report-unexpected-results", "results" => &extras_str),
                    );
                    total_checks += 1;
                    fails += 1;
                }
            }
        }
        println!();
        println!(
            "{}",
            t_args!("report-test-summary",
                "index" => test_idx,
                "passes" => passes,
                "fails" => fails,
                "total" => total_checks
            )
        );
        println!();
        test_idx += 1;
    }

    // Print overall summary like Python does
    let all_cases: Vec<&CaseResult> = summary.cases.iter().collect();
    let (total_passes, total_fails, total_checks) =
        calculate_counts(&all_cases, ignore_extra_analyses);
    println!(
        "{}",
        t_args!("report-total-summary",
            "passes" => total_passes,
            "fails" => total_fails,
            "total" => total_checks
        )
    );
}

// New: compact format
fn print_human_compact(summary: &Summary, ignore_extra_analyses: bool) {
    let (seq, groups) = build_blocks(&summary.cases);
    let mut total_passes = 0usize;
    let mut total_fails = 0usize;
    let mut total_checks = 0usize;
    let mut test_idx = 1usize; // 1-based
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
            "{} {}",
            status,
            t_args!("report-test-header",
                "index" => test_idx,
                "group" => &key.0,
                "direction" => &mode_label(&key.1)
            )
        );
        println!(
            "{}",
            t_args!("report-final-counts",
                "passes" => passes,
                "fails" => fails,
                "total" => checks
            )
        );
        total_passes += passes;
        total_fails += fails;
        total_checks += checks;
        test_idx += 1;
    }
    println!(
        "{}",
        t_args!("report-total-summary",
            "passes" => total_passes,
            "fails" => total_fails,
            "total" => total_checks
        )
    );
}

// New: terse format (dots/exclamations for each check, one line per test block, and PASS/FAIL at the end)
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
    println!(
        "{}",
        if any_fail {
            t!("report-fail")
        } else {
            t!("report-pass")
        }
    );
}

// New: final format (only total summary P/F/T)
fn print_human_final(summary: &Summary, ignore_extra_analyses: bool) {
    // Use centralized counting function
    let all_cases: Vec<&CaseResult> = summary.cases.iter().collect();
    let (total_passes, total_fails, total_checks) =
        calculate_counts(&all_cases, ignore_extra_analyses);
    println!(
        "{}",
        t_args!("report-final-counts",
            "passes" => total_passes,
            "fails" => total_fails,
            "total" => total_checks
        )
    );
}

fn is_pass_empty_expected(case: &CaseResult, ignore_extra_analyses: bool) -> bool {
    match case.direction {
        Direction::Analyze if ignore_extra_analyses => true,
        _ => case.actual.is_empty(),
    }
}

// Centralized counting function to ensure consistency across all reporting formats
// Counts individual expectations like Python version does
pub fn calculate_counts(
    cases: &[&CaseResult],
    ignore_extra_analyses: bool,
) -> (usize, usize, usize) {
    let mut total_passes = 0;
    let mut total_fails = 0;
    let mut total_checks = 0;

    for case in cases {
        let act_set: BTreeSet<&str> = case.actual.iter().map(|s| s.as_str()).collect();
        let exp_set: BTreeSet<&str> = case.expected.iter().map(|s| s.as_str()).collect();

        // Handle empty expected case as a single check
        if case.expected.is_empty() {
            let is_pass = is_pass_empty_expected(case, ignore_extra_analyses);
            total_checks += 1;
            if is_pass {
                total_passes += 1;
            } else {
                total_fails += 1;
            }

            // For analyze direction when not ignoring extras, count extras as additional fails
            if !ignore_extra_analyses && matches!(case.direction, Direction::Analyze) {
                let extras: Vec<&str> = act_set.difference(&exp_set).cloned().collect();
                if !extras.is_empty() {
                    total_checks += 1;
                    total_fails += 1;
                }
            }
        } else {
            // Count each individual expectation
            for exp in &case.expected {
                let ok = act_set.contains(exp.as_str());
                total_checks += 1;
                if ok {
                    total_passes += 1;
                } else {
                    total_fails += 1;
                }
            }

            // For analyze direction when not ignoring extras, count extras as additional fails
            if !ignore_extra_analyses && matches!(case.direction, Direction::Analyze) {
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

// Public API: routes to correct format
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
            // compact doesn't care about verbose/hide flags; writes test lines + total summary
            print_human_compact(summary, ignore_extra_analyses);
        }
        OutputKind::Terse => {
            // terse: dots/exclamations per check, PASS/FAIL at the end
            print_human_terse(summary, ignore_extra_analyses);
        }
        OutputKind::Final => {
            // final: only total summary P/F/T
            print_human_final(summary, ignore_extra_analyses);
        }
    }
}
