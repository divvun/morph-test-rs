# CLI help text
cli-about = Morphological test runner (surface/analyze and lexical/generate)
cli-backend = Select backend/section (hfst or foma) [alias: -S/--section]
cli-generator = Override generator FST (.hfstol for HFST, .foma for Foma) [alias: --gen]
cli-analyser = Override analyser FST (.hfstol for HFST, .foma for Foma) [alias: --morph, --analyzer]
cli-silent = Silent mode: no output, and suppress stderr from lookup
cli-lookup-tool = Override lookup command (e.g. hfst-optimised-lookup, flookup) [alias: --app]
cli-ignore-extra = Analysis tests: accept when all expected analyses exist, even if extra analyses exist
cli-color = Force colors on (default is colors on)
cli-no-color = Turn off colors in report (overrides --color)
cli-verbose = Show metadata (lookup with full path, generator/analyzer with full paths, version) and progress messages. Also shows 'EXTRA' for Analysis-PASS when -i is active.
cli-surface = Run only analysis tests (surface form → analyses)
cli-lexical = Run only generation tests (lexical tags → surface forms)
cli-hide-fails = Hide failures (FAIL), show only passed (PASS)
cli-hide-passes = Hide passed (PASS), show only failures (FAIL)
cli-test = Run only specified test: number 1..N, title "Group (Lexical/Generation|Surface/Analysis)" or just the group name from YAML. Special: 0, 'null' or 'list' lists all available tests and exits.
cli-output = Report format: compact | terse | final | normal (default: normal)
cli-serial = Use serial execution instead of parallel processing (default is parallel)

# Directions and modes
direction-generate = Lexical/Generation
direction-analyze = Surface/Analysis
mode-analyze-only = Analysis-only
mode-generate-only = Generation-only
mode-all = All

# Error messages
error-no-tests-after-filter = No tests available after filtering.
error-invalid-test-number = Invalid test number {$number}. Valid range: 1..{$max}.
error-test-not-found = Test not found with ID/title: {$test}
error-validation-failed = Error: {$error}

# Info messages
info-version = {$name} v{$version}
info-suite = Suite         : {$name}
info-lookup-tool = Lookup tool   : {$path}
info-generator = Generator     : {$path}
info-analyzer = Analyzer      : {$path}
info-starting-tests = Starting testing ({$count} tests, mode: {$mode}) (batch processing)...
info-starting-parallel = Suite: {$name} (parallel processing)...
info-finished = Finished: passed {$passed}, failed {$failed}. Writing report...
info-all-finished = All test runs finished. Total: {$total}, Passed: {$passed}, Failed: {$failed}

# Test listing
available-tests = Available tests:
test-list-item = {$index}: {$group} ({$direction})

# Report labels
report-pass = PASS
report-fail = FAIL
report-extra = EXTRA
report-error = Error
report-expected = Expected
report-got = Got
report-unexpected-results = Unexpected results: {$results}
report-no-lexical = <No lexical/generation>
report-no-surface = <No surface/analysis>
report-test-header = Test {$index}: {$group} ({$direction})
report-test-summary = Test {$index} - Passes: {$passes}, Fails: {$fails}, Total: {$total}
report-total-summary = Total passes: {$passes}, Total fails: {$fails}, Total: {$total}
report-final-counts = {$passes}/{$fails}/{$total}

# Backend error messages
backend-failed-to-start = Failed to start '{$cmd}'
backend-missing-stdin = Missing stdin
backend-process-failed = Lookup process failed with status {$status}
backend-timeout = Lookup timeout after {$seconds} s
backend-process-failed-stderr = Lookup process failed with status {$status}
Stderr: {$stderr}
backend-analyzer-not-set = Analyzer FST not set
backend-generator-not-set = Generator FST not set
backend-command-not-executable = Lookup command '{$cmd}' could not be executed: {$error}
backend-command-not-found = Lookup command '{$cmd}' not found or cannot be executed. Check that it is installed and in PATH.
backend-command-error = Cannot execute lookup command '{$cmd}': {$error}

# Pool error messages
pool-io-error = IO error reading from process: {$error}
pool-missing-stdin = Missing stdin
pool-missing-stdout = Missing stdout
pool-process-exited = Process has exited
pool-process-status-error = Error checking process status
pool-create-analyze-failed = Failed to create analyze pool
pool-create-generate-failed = Failed to create generate pool
pool-get-analyze-failed = Failed to get process from analyze pool: {$error}
pool-get-generate-failed = Failed to get process from generate pool: {$error}
pool-validate-analyze-failed = Failed to get process from analyze pool for validation: {$error}
pool-validate-generate-failed = Failed to get process from generate pool for validation: {$error}

# Spec error messages
spec-failed-to-read = Failed to read: {$file}
spec-yaml-error = YAML error in: {$file}
spec-incomplete-config = Incomplete or unclear Config in {$file}
spec-missing-config = Config missing
spec-missing-gen = Found neither HFST.Gen nor Foma.Gen in Config
spec-missing-hfst = Config.hfst missing
spec-missing-hfst-gen = Config.hfst.Gen missing
spec-missing-foma = Config.foma missing
spec-missing-foma-gen = Config.foma.Gen missing

# Debug messages
debug-batch-lookup = Running batch lookup with {$count} inputs using FST: {$fst}
debug-batch-completed = Batch lookup completed: {$inputs} inputs processed, {$results} total results
debug-pool-batch = Pool process batch: processing {$count} inputs
debug-pool-completed = Pool batch completed: {$inputs} inputs processed, {$results} total results

# Engine messages
engine-not-processed = Not processed
engine-batch-analyze-error = Batch analyze error: {$error}
engine-batch-generate-error = Batch generate error: {$error}