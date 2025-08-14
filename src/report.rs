use crate::types::{Direction, Summary};
use colored::Colorize;
use std::collections::BTreeSet;
pub fn render_human(summary: &Summary, ignore_extra_analyses: bool, verbose: bool) -> String {
    let mut out = String::new();
    let header = format!(
        "Total: {}, Passed: {}, Failed: {}",
        summary.total,
        summary.passed.to_string().green(),
        if summary.failed > 0 {
            summary.failed.to_string().red().bold().to_string()
        } else {
            summary.failed.to_string().green().to_string()
        }
    );
    out.push_str(&header);
    out.push('\n');
    for c in &summary.cases {
        let dir_label = match c.direction {
            Direction::Generate => "GENERATE",
            Direction::Analyze => "ANALYZE",
        };
        if c.passed {
            out.push_str(&format!(
                "{} {} {}\n",
                "[OK]".green().bold(),
                dir_label.green(),
                c.name.green()
            ));
            if verbose {
                match c.direction {
                    Direction::Generate => {
                        out.push_str(&format!("  {} {:?}\n", "generated:".bold(), c.actual));
                    }
                    Direction::Analyze => {
                        out.push_str(&format!("  {} {:?}\n", "analyses :".bold(), c.actual));
                        // Vis alltid ekstra analysar i verbose-modus (om dei finst),
                        // uavhengig av filter-/køyringsmodus. Farge: gul når dei er ignorerte,
                        // men i pass-tilfelle kan det berre skje om -i var aktiv.
                        let exp: BTreeSet<&str> = c.expected.iter().map(|s| s.as_str()).collect();
                        let act: BTreeSet<&str> = c.actual.iter().map(|s| s.as_str()).collect();
                        let extra: Vec<&str> = act.difference(&exp).cloned().collect();
                        if !extra.is_empty() {
                            if ignore_extra_analyses {
                                out.push_str(&format!(
                                    "  {} {:?}\n",
                                    "extra analyses (ignored):".bold().yellow(),
                                    extra
                                ));
                            } else {
                                // Teoretisk kjem ikkje dette i PASS (utan -i), men held likevel på semantikken.
                                out.push_str(&format!(
                                    "  {} {:?}\n",
                                    "extra analyses:".bold(),
                                    extra
                                ));
                            }
                        }
                    }
                }
            }
        } else {
            out.push_str(&format!(
                "{} {} {}\n",
                "[FAIL]".red().bold(),
                dir_label.red().bold(),
                c.name.red().bold()
            ));
            out.push_str(&format!("  {} {}\n", "input   :".bold(), c.input));
            if let Some(err) = &c.error {
                out.push_str(&format!("  {} {}\n", "error   :".bold(), err.red()));
            } else {
                match c.direction {
                    Direction::Generate => {
                        out.push_str(&format!(
                            "  {} {:?}\n",
                            "expected forms:".bold(),
                            c.expected
                        ));
                        out.push_str(&format!("  {} {:?}\n", "generated     :".bold(), c.actual));
                    }
                    Direction::Analyze => {
                        out.push_str(&format!(
                            "  {} {:?}\n",
                            "expected analyses:".bold(),
                            c.expected
                        ));
                        out.push_str(&format!(
                            "  {} {:?}\n",
                            "analyses          :".bold(),
                            c.actual
                        ));
                    }
                }
                // Diff (alltid)
                let exp: BTreeSet<&str> = c.expected.iter().map(|s| s.as_str()).collect();
                let act: BTreeSet<&str> = c.actual.iter().map(|s| s.as_str()).collect();
                let missing: Vec<&str> = exp.difference(&act).cloned().collect();
                let extra: Vec<&str> = act.difference(&exp).cloned().collect();
                if !missing.is_empty() {
                    out.push_str(&format!("  {} {:?}\n", "missing :".bold(), missing));
                }
                if !extra.is_empty() {
                    if matches!(c.direction, Direction::Analyze) && ignore_extra_analyses {
                        // Når -i er på, merk at dette er ekstra analysar som kunne vore ignorerte (men her feila testen likevel,
                        // t.d. fordi noko mangla).
                        out.push_str(&format!(
                            "  {} {:?}\n",
                            "extra analyses (ignored):".bold().yellow(),
                            extra
                        ));
                    } else {
                        out.push_str(&format!("  {} {:?}\n", "unexpected:".bold(), extra));
                    }
                }
            }
        }
    }
    out
}
pub fn print_human(summary: &Summary, ignore_extra_analyses: bool, verbose: bool) {
    print!("{}", render_human(summary, ignore_extra_analyses, verbose));
}
