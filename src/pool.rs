use crate::{t, t_args};
use anyhow::{Context, Result, anyhow};
use deadpool::managed::{Manager, Metrics, Pool, RecycleError, RecycleResult};
use futures::future::try_join_all;
use indexmap::IndexMap;
use std::borrow::Cow;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tracing::debug;

/// 30 seconds per lookup
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// A persistent FST process that can handle multiple queries
pub struct FstProcess {
    pub child: Child,
    pub stdin: BufWriter<ChildStdin>,
    pub stdout: BufReader<ChildStdout>,
}

impl FstProcess {
    /// Send a batch of inputs and read results
    pub async fn process_batch(&mut self, inputs: &[String]) -> Result<Vec<Vec<String>>> {
        debug!("{}", t_args!("debug-pool-batch", "count" => inputs.len()));
        // Send all inputs
        for input in inputs {
            let input_trimmed = input.trim();
            self.stdin.write_all(input_trimmed.as_bytes()).await?;
            self.stdin.write_all(b"\n").await?;
        }
        self.stdin.flush().await?;

        // Read results - FST tools output input\toutput format
        let mut results_map: IndexMap<String, std::collections::BTreeSet<String>> = IndexMap::new();
        let mut lines_read = 0;
        let _expected_inputs: std::collections::HashSet<&str> =
            inputs.iter().map(|s| s.trim()).collect();

        // Initialize all inputs in the map to preserve order and handle no-result cases
        for input in inputs {
            results_map.insert(input.trim().to_string(), std::collections::BTreeSet::new());
        }

        // Read all available output until timeout or reasonable limit
        let mut line = String::new();
        let max_lines = inputs.len() * 50; // More generous limit for multiple results per input

        while lines_read < max_lines {
            line.clear();
            match tokio::time::timeout(Duration::from_millis(500), self.stdout.read_line(&mut line))
                .await
            {
                Ok(Ok(0)) => break, // EOF
                Ok(Ok(_)) => {
                    lines_read += 1;
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Skip comments and warnings (only if the line doesn't contain a tab, meaning it's not FST output)
                    if (trimmed.starts_with('!') || trimmed.starts_with('#')) && !trimmed.contains('\t') {
                        continue;
                    }

                    let cols: Vec<&str> = trimmed.split('\t').collect();
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
                Ok(Err(e)) => return Err(anyhow!(t_args!("pool-io-error", "error" => &e))),
                Err(_) => {
                    // Timeout is expected when no more output is available
                    break;
                }
            }
        }

        // Build ordered results matching input order
        let mut all_results = Vec::new();
        for input in inputs {
            let input_key = input.trim().to_string();
            let results_set = results_map.shift_remove(&input_key).unwrap_or_default();
            let results: Vec<String> = results_set.into_iter().collect();
            all_results.push(results);
        }

        debug!(
            "{}",
            t_args!("debug-pool-completed",
                "inputs" => inputs.len(),
                "results" => all_results.iter().map(|r| r.len()).sum::<usize>()
            )
        );
        Ok(all_results)
    }
}

/// Manager for creating and recycling FST processes
pub struct FstProcessManager {
    pub lookup_cmd: String,
    pub fst_path: String,
    pub quiet: bool,
}

impl Manager for FstProcessManager {
    type Type = FstProcess;
    type Error = anyhow::Error;

    async fn create(&self) -> Result<FstProcess> {
        let mut cmd = Command::new(&self.lookup_cmd);
        cmd.arg(&self.fst_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(if self.quiet {
                Stdio::null()
            } else {
                Stdio::piped()
            });

        let mut child = cmd
            .spawn()
            .with_context(|| t_args!("backend-failed-to-start", "cmd" => &self.lookup_cmd))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!(t!("pool-missing-stdin")))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!(t!("pool-missing-stdout")))?;

        Ok(FstProcess {
            child,
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
        })
    }

    async fn recycle(
        &self,
        obj: &mut FstProcess,
        _metrics: &Metrics,
    ) -> RecycleResult<Self::Error> {
        // Check if the child process is still alive
        match obj.child.try_wait() {
            Ok(Some(_)) => Err(RecycleError::Message(Cow::Owned(t!("pool-process-exited")))),
            Ok(None) => Ok(()), // Still running
            Err(_e) => Err(RecycleError::Message(Cow::Owned(t!(
                "pool-process-status-error"
            )))),
        }
    }
}

impl Drop for FstProcess {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

/// Pooled backend that manages FST processes efficiently
pub struct PooledBackend {
    analyze_pool: Option<Pool<FstProcessManager>>,
    generate_pool: Option<Pool<FstProcessManager>>,
}

impl PooledBackend {
    pub async fn new(
        lookup_cmd: String,
        analyzer_fst: Option<String>,
        generator_fst: Option<String>,
        quiet: bool,
    ) -> Result<Self> {
        let pool_size = num_cpus::get().max(1);

        let analyze_pool = if let Some(fst_path) = analyzer_fst {
            let manager = FstProcessManager {
                lookup_cmd: lookup_cmd.clone(),
                fst_path,
                quiet,
            };
            Some(
                Pool::builder(manager)
                    .max_size(pool_size)
                    .build()
                    .context(t!("pool-create-analyze-failed"))?,
            )
        } else {
            None
        };

        let generate_pool = if let Some(fst_path) = generator_fst {
            let manager = FstProcessManager {
                lookup_cmd,
                fst_path,
                quiet,
            };
            Some(
                Pool::builder(manager)
                    .max_size(pool_size)
                    .build()
                    .context(t!("pool-create-generate-failed"))?,
            )
        } else {
            None
        };

        Ok(Self {
            analyze_pool,
            generate_pool,
        })
    }

    pub async fn analyze_batch(&self, inputs: &[String]) -> Result<Vec<Vec<String>>> {
        let pool = self
            .analyze_pool
            .as_ref()
            .ok_or_else(|| anyhow!(t!("backend-analyzer-not-set")))?;

        if inputs.is_empty() {
            return Ok(vec![]);
        }

        // Split into chunks based on pool size
        let pool_status = pool.status();
        let chunk_size = (inputs.len() / pool_status.max_size).max(1);
        let chunks: Vec<_> = inputs.chunks(chunk_size).collect();

        // Process chunks in parallel
        let futures = chunks.iter().map(|chunk| {
            let pool = pool.clone();
            async move {
                let mut process = pool
                    .get()
                    .await
                    .map_err(|e| anyhow!(t_args!("pool-get-analyze-failed", "error" => &e)))?;
                process.process_batch(chunk).await
            }
        });

        let chunk_results = try_join_all(futures).await?;

        // Flatten results maintaining order
        Ok(chunk_results.into_iter().flatten().collect())
    }

    pub async fn generate_batch(&self, inputs: &[String]) -> Result<Vec<Vec<String>>> {
        let pool = self
            .generate_pool
            .as_ref()
            .ok_or_else(|| anyhow!(t!("backend-generator-not-set")))?;

        if inputs.is_empty() {
            return Ok(vec![]);
        }

        // Split into chunks based on pool size
        let pool_status = pool.status();
        let chunk_size = (inputs.len() / pool_status.max_size).max(1);
        let chunks: Vec<_> = inputs.chunks(chunk_size).collect();

        // Process chunks in parallel
        let futures = chunks.iter().map(|chunk| {
            let pool = pool.clone();
            async move {
                let mut process = pool
                    .get()
                    .await
                    .map_err(|e| anyhow!(t_args!("pool-get-generate-failed", "error" => &e)))?;
                process.process_batch(chunk).await
            }
        });

        let chunk_results = try_join_all(futures).await?;

        // Flatten results maintaining order
        Ok(chunk_results.into_iter().flatten().collect())
    }

    pub async fn validate(&self) -> Result<()> {
        // Test that we can spawn and use a process from each pool
        if let Some(pool) = &self.analyze_pool {
            let _process = pool
                .get()
                .await
                .map_err(|e| anyhow!(t_args!("pool-validate-analyze-failed", "error" => &e)))?;
            // The process creation in the manager already validates the command exists
        }

        if let Some(pool) = &self.generate_pool {
            let _process = pool
                .get()
                .await
                .map_err(|e| anyhow!(t_args!("pool-validate-generate-failed", "error" => &e)))?;
            // The process creation in the manager already validates the command exists
        }

        Ok(())
    }
}
