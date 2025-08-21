use crate::{t, t_args};
use anyhow::{Context, Result, anyhow};
use indexmap::IndexMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;
use tracing::debug;
use wait_timeout::ChildExt;

/// 30 seconds per lookup
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Generic backend that runs an external lookup program (hfst-optimised-lookup, flookup, etc.)
#[derive(Debug, Clone)]
pub struct ExternalBackend {
    pub lookup_cmd: String, // "hfst-optimised-lookup" or "flookup"
    pub generator_fst: Option<String>,
    pub analyzer_fst: Option<String>,
    pub timeout: Option<Duration>,
    pub quiet: bool, // suppress stderr from lookup when true
}

pub trait Backend: Send + Sync {
    fn analyze_batch(&self, inputs: &[String]) -> Result<Vec<Vec<String>>>;
    fn generate_batch(&self, inputs: &[String]) -> Result<Vec<Vec<String>>>;
    fn validate(&self) -> Result<()>;
}

impl ExternalBackend {
    fn run_lookup_batch(&self, fst: &str, inputs: &[String]) -> Result<Vec<Vec<String>>> {
        let timeout = self.timeout.unwrap_or(DEFAULT_TIMEOUT);
        debug!(
            "{}",
            t_args!("debug-batch-lookup",
                "count" => inputs.len(),
                "fst" => fst
            )
        );

        let mut cmd = Command::new(&self.lookup_cmd);
        cmd.arg(fst)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(if self.quiet {
                Stdio::null()
            } else {
                Stdio::inherit()
            });

        let mut child = cmd
            .spawn()
            .with_context(|| t_args!("backend-failed-to-start", "cmd" => &self.lookup_cmd))?;

        // Send all inputs at once
        {
            let stdin = child
                .stdin
                .as_mut()
                .ok_or_else(|| anyhow!(t!("backend-missing-stdin")))?;

            for input in inputs {
                let input_trimmed = input.trim();
                stdin.write_all(input_trimmed.as_bytes())?;
                stdin.write_all(b"\n")?;
            }
            // Drop stdin to close it and signal EOF
        }

        // Wait for completion with timeout
        match child.wait_timeout(timeout)? {
            Some(status) => {
                if !status.success() {
                    return Err(anyhow!(
                        t_args!("backend-process-failed", "status" => status)
                    ));
                }
            }
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(anyhow!(
                    t_args!("backend-timeout", "seconds" => timeout.as_secs())
                ));
            }
        }

        let out = child.wait_with_output()?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(anyhow!(t_args!("backend-process-failed-stderr",
                "status" => out.status,
                "stderr" => stderr.as_ref()
            )));
        }

        // Parse batch output - FST tools output: input\toutput format
        let stdout = String::from_utf8_lossy(&out.stdout);
        let mut results_map: IndexMap<String, std::collections::BTreeSet<String>> = IndexMap::new();

        // Initialize all inputs in the map to preserve order and handle no-result cases
        for input in inputs {
            results_map.insert(input.trim().to_string(), std::collections::BTreeSet::new());
        }

        for raw_line in stdout.lines() {
            let trimmed = raw_line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with('!') || trimmed.starts_with('#') {
                continue;
            }

            let cols: Vec<&str> = raw_line.split('\t').collect();
            if cols.len() >= 2 {
                let input = cols[0].trim().to_string();
                let output = cols[1].trim().to_string();

                // Handle +inf (no result) marker
                if output == "+inf" {
                    // Input with no results - already initialized as empty Set
                    continue;
                }

                // Handle cases where FST couldn't process: input==output with +? in third column
                if cols.len() >= 3 && input == output && cols[2].trim().contains("+?") {
                    // FST failed to generate/analyze - treat as no result
                    continue;
                }

                if !output.is_empty() && output != "@" {
                    if let Some(results) = results_map.get_mut(&input) {
                        results.insert(output);
                    }
                }
            }
        }

        // Build ordered results matching input order
        let mut all_results = Vec::new();
        for input in inputs {
            let input_key = input.trim().to_string();
            let results_set = results_map.get(&input_key).cloned().unwrap_or_default();
            let results: Vec<String> = results_set.into_iter().collect();
            all_results.push(results);
        }

        debug!(
            "{}",
            t_args!("debug-batch-completed",
                "inputs" => inputs.len(),
                "results" => all_results.iter().map(|r| r.len()).sum::<usize>()
            )
        );
        Ok(all_results)
    }
}

impl Backend for ExternalBackend {
    fn analyze_batch(&self, inputs: &[String]) -> Result<Vec<Vec<String>>> {
        let fst = self
            .analyzer_fst
            .as_ref()
            .ok_or_else(|| anyhow!(t!("backend-analyzer-not-set")))?;
        self.run_lookup_batch(fst, inputs)
    }

    fn generate_batch(&self, inputs: &[String]) -> Result<Vec<Vec<String>>> {
        let fst = self
            .generator_fst
            .as_ref()
            .ok_or_else(|| anyhow!(t!("backend-generator-not-set")))?;
        self.run_lookup_batch(fst, inputs)
    }

    fn validate(&self) -> Result<()> {
        // Check if lookup command exists and is executable
        use std::process::Command;

        let mut cmd = Command::new(&self.lookup_cmd);
        cmd.arg("--help")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        match cmd.spawn() {
            Ok(mut child) => {
                match child.wait() {
                    Ok(_status) => {
                        // Command exists and ran, don't care about exit code for --help
                        Ok(())
                    }
                    Err(e) => Err(anyhow!(t_args!("backend-command-not-executable",
                        "cmd" => &self.lookup_cmd,
                        "error" => &e
                    ))),
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Err(anyhow!(
                        t_args!("backend-command-not-found", "cmd" => &self.lookup_cmd)
                    ))
                } else {
                    Err(anyhow!(t_args!("backend-command-error",
                        "cmd" => &self.lookup_cmd,
                        "error" => &e
                    )))
                }
            }
        }
    }
}
