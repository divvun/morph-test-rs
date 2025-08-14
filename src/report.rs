use crate::types::{CaseResult, Direction, Summary};
use colored::Colorize;
use std::collections::{BTreeMap, BTreeSet};
fn parse_group(name: &str) -> (&str, &str) {
    // Deler "Group: Input" i ("Group", "Input")
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
pub fn print_human(summary: &Summary, ignore_extra_analyses: bool, verbose: bool) {
    // Gruppér etter (group, direction) i rekkjefølgje vi møter dei
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct Key {
        group: String,
        dir: Direction,
    }
    let mut order: Vec<Key> = Vec::new();
    let mut groups: BTreeMap<(String, Direction), Vec<&CaseResult>> = BTreeMap::new();
    for c in &summary.cases {
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
    // For kvar blokk (gruppe+retning)
    let mut test_idx = 0usize;
    for key in order {
        let cases = match groups.get(&(key.group.clone(), key.dir.clone())) {
            Some(v) => v,
            None => continue,
        };
        if cases.is_empty() {
            continue;
        }
        // Tittel-linje
        let title = format!(
            "Test {}: {} ({})",
            test_idx,
            key.group,
            mode_label(&key.dir)
        );
        let line = dash_line(title.len());
        println!("{}", line);
        println!("{}", title);
        println!("{}", line);
        // i/n-indeksering: i = posisjon til caset i denne blokka (1-basert), n = talet på cases
        let n_cases = cases.len();
        let mut passes = 0usize;
        let mut fails = 0usize;
        let mut total_lines = 0usize;
        for (idx, case) in cases.iter().enumerate() {
            let i = idx + 1;
            let exp_set: BTreeSet<&str> = case.expected.iter().map(|s| s.as_str()).collect();
            let act_set: BTreeSet<&str> = case.actual.iter().map(|s| s.as_str()).collect();
            // Når expected er tom, skriv placeholder-linje
            if case.expected.is_empty() {
                let placeholder = match case.direction {
                    Direction::Generate => "<No lexical/generation>",
                    Direction::Analyze => "<No surface/analysis>",
                };
                let is_pass = match case.direction {
                    Direction::Analyze if ignore_extra_analyses => true, // godta ekstra analysar
                    _ => case.actual.is_empty(),
                };
                let status = if is_pass {
                    "PASS".green().bold()
                } else {
                    "FAIL".red().bold()
                };
                println!(
                    "[{}/{}][{}] {} => {}",
                    i, n_cases, status, case.input, placeholder
                );
                total_lines += 1;
                if is_pass {
                    passes += 1;
                } else {
                    fails += 1;
                }
                // Ekstra analysar (verbose + ignore) – vis dei i tillegg
                if verbose && ignore_extra_analyses && matches!(case.direction, Direction::Analyze)
                {
                    let extras: Vec<&str> = act_set.difference(&exp_set).cloned().collect();
                    for e in extras {
                        println!(
                            "[{}/{}][{}] {} => {}",
                            i,
                            n_cases,
                            "EXTRA".yellow().bold(),
                            case.input,
                            e
                        );
                        total_lines += 1;
                    }
                }
                continue;
            }
            // Éi linje per forventa verdi
            for exp in &case.expected {
                let ok = act_set.contains(exp.as_str());
                let status = if ok {
                    "PASS".green().bold()
                } else {
                    "FAIL".red().bold()
                };
                println!("[{}/{}][{}] {} => {}", i, n_cases, status, case.input, exp);
                total_lines += 1;
                if ok {
                    passes += 1;
                } else {
                    fails += 1;
                }
            }
            // For Analyze: vis ekstra analysar i verbose når -i er aktiv
            if verbose && ignore_extra_analyses && matches!(case.direction, Direction::Analyze) {
                let extras: Vec<&str> = act_set.difference(&exp_set).cloned().collect();
                for e in extras {
                    println!(
                        "[{}/{}][{}] {} => {}",
                        i,
                        n_cases,
                        "EXTRA".yellow().bold(),
                        case.input,
                        e
                    );
                    total_lines += 1;
                }
            }
        }
        println!();
        println!(
            "Test {} - Passes: {}, Fails: {}, Total: {}",
            test_idx, passes, fails, total_lines
        );
        println!();
        test_idx += 1;
    }
}
