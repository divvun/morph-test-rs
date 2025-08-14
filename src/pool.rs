use anyhow::{Context, Result, anyhow};
use deadpool::managed::{Manager, Metrics, Pool, RecycleError, RecycleResult};
use futures::future::try_join_all;
use indexmap::IndexMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

/// 30 sekund per oppslag
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
        // Send all inputs
        for input in inputs {
            let input_trimmed = input.trim();
            self.stdin.write_all(input_trimmed.as_bytes()).await?;
            self.stdin.write_all(b"\n").await?;
        }
        self.stdin.flush().await?;

        // Read results - FST tools output input\toutput format
        let mut results_map: IndexMap<String, Vec<String>> = IndexMap::new();
        let mut lines_read = 0;
        let expected_inputs: std::collections::HashSet<&str> =
            inputs.iter().map(|s| s.trim()).collect();

        // Read until we have results for all inputs or timeout
        let mut line = String::new();
        while lines_read < inputs.len() * 10 {
            // Reasonable upper bound
            line.clear();
            match tokio::time::timeout(DEFAULT_TIMEOUT, self.stdout.read_line(&mut line)).await {
                Ok(Ok(0)) => break, // EOF
                Ok(Ok(_)) => {
                    lines_read += 1;
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if trimmed.starts_with('!') || trimmed.starts_with('#') {
                        continue;
                    }

                    let cols: Vec<&str> = trimmed.split('\t').collect();
                    if cols.len() >= 2 {
                        let input = cols[0].trim().to_string();
                        let output = cols[1].trim().to_string();
                        if !output.is_empty() && output != "@" {
                            results_map.entry(input).or_default().push(output);
                        }
                    }
                }
                Ok(Err(e)) => return Err(anyhow!("IO error reading from process: {}", e)),
                Err(_) => return Err(anyhow!("Timeout reading from FST process")),
            }

            // Check if we have at least one result for each input
            if results_map.keys().len() >= expected_inputs.len() {
                break;
            }
        }

        // Build ordered results matching input order
        let mut all_results = Vec::new();
        for input in inputs {
            let input_key = input.trim().to_string();
            let results = results_map.shift_remove(&input_key).unwrap_or_default();
            all_results.push(results);
        }

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
            .with_context(|| format!("Klarte ikkje å starta '{}'", self.lookup_cmd))?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow!("Manglar stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Manglar stdout"))?;

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
            Ok(Some(_)) => Err(RecycleError::message("Process has exited")),
            Ok(None) => Ok(()), // Still running
            Err(_e) => Err(RecycleError::message("Error checking process status")),
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
                    .context("Failed to create analyze pool")?,
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
                    .context("Failed to create generate pool")?,
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
            .ok_or_else(|| anyhow!("Analyzer-FST ikkje sett"))?;

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
                    .map_err(|e| anyhow!("Failed to get process from analyze pool: {:?}", e))?;
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
            .ok_or_else(|| anyhow!("Generator-FST ikkje sett"))?;

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
                    .map_err(|e| anyhow!("Failed to get process from generate pool: {:?}", e))?;
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
            let _process = pool.get().await.map_err(|e| {
                anyhow!(
                    "Failed to get process from analyze pool for validation: {:?}",
                    e
                )
            })?;
            // The process creation in the manager already validates the command exists
        }

        if let Some(pool) = &self.generate_pool {
            let _process = pool.get().await.map_err(|e| {
                anyhow!(
                    "Failed to get process from generate pool for validation: {:?}",
                    e
                )
            })?;
            // The process creation in the manager already validates the command exists
        }

        Ok(())
    }
}
