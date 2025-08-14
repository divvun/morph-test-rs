use crate::types::{CaseResult, Direction, Summary};
use colored::Colorize;
use std::collections::{BTreeMap, BTreeSet};
fn group_key(name: &str) -> &str {
    // CaseResult.name vart sett til "group: input" i parseren; ta alt før første ": "
    name.splitn(2, ": ").next().unwrap_or(name)
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
    // Gruppér etter gruppe-namn i same rekkjefølgje som først observert
    let mut order: Vec<String> = Vec::new();
    let mut groups: BTreeMap<String, Vec<&CaseResult>> = BTreeMap::new();
    for c in &summary.cases {
        let g = group_key(&c.name).to_string();
        if !groups.contains_key(&g) {
            order.push(g.clone());
            groups.insert(g.clone(), Vec::new());
        }
        groups.get_mut(&g).unwrap().push(c);
    }
    let mut test_idx = 0;
    for g in order {
        let cases = &groups[&g];
        if cases.is_empty() {
            continue;
        }
        // Retning per gruppe (antar homogen retning i gruppa)
        let dir = &cases[0].direction;
        let mode = mode_label(dir);
        // Overskrift og strekar
        let title = format!("Test {}: {} ({})", test_idx, g, mode);
        let line = dash_line(title.len());
        println!("{}", line);
        println!("{}", title);
        println!("{}", line);
        // Talet på "innslag" i gruppa
        let n_items = cases.len();
        let mut passes = 0usize;
        let mut fails = 0usize;
        let mut total_lines = 0usize;
        // Iterér i den rekkjefølgja casane kjem
        for (i, case) in cases.iter().enumerate() {
            // Sett for snabb-lookup
            let exp_set: BTreeSet<&str> = case.expected.iter().map(|s| s.as_str()).collect();
            let act_set: BTreeSet<&str> = case.actual.iter().map(|s| s.as_str()).collect();
            // Når expected er tom, skriv éi linje med placeholder
            if case.expected.is_empty() {
                let placeholder = match case.direction {
                    Direction::Generate => "<No lexical/generation>",
                    Direction::Analyze => "<No surface/analysis>",
                };
                let is_pass = case.actual.is_empty()
                    || (matches!(case.direction, Direction::Analyze)
                        && ignore_extra_analyses
                        && !case.actual.is_empty());
                // Merk: for Analyze + ignore, tom expected og non-tom actual vil formelt vere PASS på subset-kriteriet,
                // men då vil vi òg potensielt skrive EXTRA-linjer under verbose (sjå nedanfor).
                let status = if is_pass {
                    "PASS".green().bold()
                } else {
                    "FAIL".red().bold()
                };
                println!(
                    "[{}/{}][{}] {} => {}",
                    i + 1,
                    n_items,
                    status,
                    case.input,
                    placeholder
                );
                total_lines += 1;
                if is_pass {
                    passes += 1;
                } else {
                    fails += 1;
                }
                // Ekstra analysar i Analyze + ignore + verbose
                if verbose && ignore_extra_analyses && matches!(case.direction, Direction::Analyze)
                {
                    let extras: Vec<&str> = act_set.difference(&exp_set).cloned().collect();
                    for e in extras {
                        println!(
                            "[{}/{}][{}] {} => {}",
                            i + 1,
                            n_items,
                            "EXTRA".yellow().bold(),
                            case.input,
                            e
                        );
                        total_lines += 1;
                    }
                }
                continue;
            }
            // For kvar forventa verdi, skriv PASS/FAIL-line
            for exp in &case.expected {
                let ok = act_set.contains(exp.as_str());
                let status = if ok {
                    "PASS".green().bold()
                } else {
                    "FAIL".red().bold()
                };
                println!(
                    "[{}/{}][{}] {} => {}",
                    i + 1,
                    n_items,
                    status,
                    case.input,
                    exp
                );
                total_lines += 1;
                if ok {
                    passes += 1;
                } else {
                    fails += 1;
                }
            }
            // Ved Analyze + ignore + verbose: vis ekstra analysar (gul) som ikkje var i expected
            if verbose && ignore_extra_analyses && matches!(case.direction, Direction::Analyze) {
                let extras: Vec<&str> = act_set.difference(&exp_set).cloned().collect();
                for e in extras {
                    println!(
                        "[{}/{}][{}] {} => {}",
                        i + 1,
                        n_items,
                        "EXTRA".yellow().bold(),
                        case.input,
                        e
                    );
                    total_lines += 1;
                }
            }
            // For Generate (Lexical/Generation) kan det finnast uventa former i actual.
            // Desse tel ikkje som eigne linjer i denne kompakte rapporten for å halde formatet stramt.
            // Feil vert fanga opp via FAIL-linjer når forventa manglar, og totalsummen vil spegle det.
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
