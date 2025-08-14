use crate::types::Summary;
use colored::Colorize;
use std::collections::BTreeSet;
pub fn render_human(summary: &Summary) -> String {
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
        if c.passed {
            out.push_str(&format!("{} {}\n", "[OK]".green().bold(), c.name.green()));
        } else {
            out.push_str(&format!(
                "{} {}\n",
                "[FAIL]".red().bold(),
                c.name.red().bold()
            ));
            out.push_str(&format!("  {} {}\n", "input   :".bold(), c.input));
            if let Some(err) = &c.error {
                out.push_str(&format!("  {} {}\n", "error   :".bold(), err.red()));
            } else {
                out.push_str(&format!("  {} {:?}\n", "expected:".bold(), c.expected));
                out.push_str(&format!("  {} {:?}\n", "actual  :".bold(), c.actual));
                // Diff for set-likskap
                let exp: BTreeSet<&str> = c.expected.iter().map(|s| s.as_str()).collect();
                let act: BTreeSet<&str> = c.actual.iter().map(|s| s.as_str()).collect();
                let missing: Vec<&str> = exp.difference(&act).cloned().collect();
                let extra: Vec<&str> = act.difference(&exp).cloned().collect();
                if !missing.is_empty() {
                    out.push_str(&format!("  {} {:?}\n", "missing :".bold(), missing));
                }
                if !extra.is_empty() {
                    out.push_str(&format!("  {} {:?}\n", "unexpected:".bold(), extra));
                }
            }
        }
    }
    out
}
pub fn print_human(summary: &Summary) {
    print!("{}", render_human(summary));
}
