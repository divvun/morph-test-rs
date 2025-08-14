use anyhow::{anyhow, Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;
/// 30 sekund per oppslag
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
/// Generisk backend som køyrer eit eksternt lookup-program (hfst-lookup, lookup, osb.)
#[derive(Debug, Clone)]
pub struct ExternalBackend {
    pub lookup_cmd: String,        // "hfst-lookup" eller "lookup"
    pub generator_fst: Option<String>,
    pub analyzer_fst: Option<String>,
    pub timeout: Option<Duration>,
}
pub trait Backend: Send + Sync {
    fn analyze(&self, input: &str) -> Result<Vec<String>>;
    fn generate(&self, input: &str) -> Result<Vec<String>>;
}
impl ExternalBackend {
    fn run_lookup(&self, fst: &str, input: &str) -> Result<Vec<String>> {
        let timeout = self.timeout.unwrap_or(DEFAULT_TIMEOUT);
        let mut child = Command::new(&self.lookup_cmd)
            .arg(fst)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("Klarte ikkje å starte '{}'", self.lookup_cmd))?;
        // Send éi linje input
        {
            let stdin = child.stdin.as_mut().ok_or_else(|| anyhow!("Manglar stdin"))?;
            stdin.write_all(input.as_bytes())?;
            stdin.write_all(b"\n")?;
            // Viktig: lukk stdin slik at lookup ikkje ventar på meir
            // (nokre verktøy terminerer ikkje før stdin er lukka).
        }
        // Vent med tidsavbrot
        match child.wait_timeout(timeout)? {
            Some(status) => {
                if !status.success() {
                    return Err(anyhow!("Lookup-prosess feila med status {}", status));
                }
            }
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(anyhow!("Lookup tidsavbrot etter {} s", timeout.as_secs()));
            }
        }
        let out = child.wait_with_output()?;
        if !out.status.success() {
            return Err(anyhow!("Lookup-prosess feila med status {}", out.status));
        }
        // Parser: forventa "input<TAB>output<TAB>..." per linje.
        // Ikkje trim ut-kolonna; vi vil ha eksakt tekstlikskap.
        let stdout = String::from_utf8_lossy(&out.stdout);
        let mut results = Vec::new();
        for raw_line in stdout.lines() {
            let trimmed = raw_line.trim();
            if trimmed.is_empty() { continue; }
            if trimmed.starts_with('!') || trimmed.starts_with('#') { continue; }
            let cols: Vec<&str> = raw_line.split('\t').collect();
            if cols.len() >= 2 {
                let out = cols[1].to_string();
                if !out.is_empty() && out != "@" {
                    results.push(out);
                }
            }
        }
        Ok(results)
    }
}
impl Backend for ExternalBackend {
    fn analyze(&self, input: &str) -> Result<Vec<String>> {
        let fst = self.analyzer_fst.as_ref().ok_or_else(|| anyhow!("Analyzer-FST ikkje sett"))?;
        self.run_lookup(fst, input)
    }
    fn generate(&self, input: &str) -> Result<Vec<String>> {
        let fst = self.generator_fst.as_ref().ok_or_else(|| anyhow!("Generator-FST ikkje sett"))?;
        self.run_lookup(fst, input)
    }
}
