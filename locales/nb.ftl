# CLI help text
cli-about = Morfologisk testløper (overflate/analyser og leksikalsk/generering)
cli-backend = Velg backend/seksjon (hfst eller foma) [alias: -S/--section]
cli-generator = Overstyr generator-FST (.hfstol for HFST, .foma for Foma) [alias: --gen]
cli-analyser = Overstyr analyser-FST (.hfstol for HFST, .foma for Foma) [alias: --morph, --analyzer]
cli-silent = Stille modus: ingen utskrift, og demp stderr fra lookup
cli-lookup-tool = Overstyr lookup-kommando (f.eks. hfst-optimised-lookup, flookup) [alias: --app]
cli-ignore-extra = Analyse-tester: godkjenn når alle forventede analyser finnes, selv om det finnes ekstra analyser
cli-color = Tving farger på (standard er farger på)
cli-no-color = Slå av farger i rapporten (overstyrer --color)
cli-verbose = Vis metadata (lookup med full sti, generator/analyzer med fulle stier, versjon) og fremdriftsmeldinger. Viser også 'EXTRA' for Analyse-PASS når -i er aktiv.
cli-surface = Kjør bare analysetester (surface form → analyses)
cli-lexical = Kjør bare genereringstester (lexical tags → surface forms)
cli-hide-fails = Skjul feil (FAIL), vis bare gjennomgåtte (PASS)
cli-hide-passes = Skjul gjennomgåtte (PASS), vis bare feil (FAIL)
cli-test = Kjør bare angitt test: nummer 1..N, tittel „Gruppe (Lexical/Generation|Surface/Analysis)" eller bare gruppenavnet fra YAML. Spesial: 0, 'null' eller 'liste' lister alle tilgjengelige tester og avslutter.
cli-output = Rapportformat: compact | terse | final | normal (standard: normal)
cli-serial = Bruk seriell kjøring i stedet for parallell prosessering (standardverdi er parallell)

# Directions and modes  
direction-generate = Leksikalsk/Generering
direction-analyze = Overflate/Analyse
mode-analyze-only = Bare analyse
mode-generate-only = Bare generering
mode-all = Alle

# Error messages
error-no-tests-after-filter = Ingen tester tilgjengelig etter filtrering.
error-invalid-test-number = Ugyldig testnummer {$number}. Gyldig område: 1..{$max}.
error-test-not-found = Fant ikke test med ID/tittel: {$test}
error-validation-failed = Feil: {$error}

# Info messages
info-version = {$name} v{$version}
info-suite = Suite         : {$name}
info-lookup-tool = Lookup tool   : {$path}
info-generator = Generator     : {$path}
info-analyzer = Analyzer      : {$path}
info-starting-tests = Starter testing ({$count} tester, modus: {$mode}) (batch processing)...
info-starting-parallel = Suite: {$name} (parallel processing)...
info-finished = Ferdig: passed {$passed}, failed {$failed}. Skriver rapport...
info-all-finished = Alle testkjøringer ferdige. Total: {$total}, Passed: {$passed}, Failed: {$failed}

# Test listing
available-tests = Tilgjengelige tester:
test-list-item = {$index}: {$group} ({$direction})

# Report labels
report-pass = PASS
report-fail = FAIL
report-extra = EXTRA
report-error = Feil
report-expected = Forventet
report-got = Fikk
report-unexpected-results = Uventede resultater: {$results}
report-no-lexical = <Ingen leksikalsk/generering>
report-no-surface = <Ingen overflate/analyse>
report-test-header = Test {$index}: {$group} ({$direction})
report-test-summary = Test {$index} - Bestått: {$passes}, Feilet: {$fails}, Totalt: {$total}
report-total-summary = Totalt bestått: {$passes}, Totalt feilet: {$fails}, Totalt: {$total}
report-final-counts = {$passes}/{$fails}/{$total}

# Backend error messages
backend-failed-to-start = Klarte ikke å starte '{$cmd}'
backend-missing-stdin = Mangler stdin
backend-process-failed = Lookup-prosess feilet med status {$status}
backend-timeout = Lookup tidsavbrudd etter {$seconds} s
backend-process-failed-stderr = Lookup-prosess feilet med status {$status}
Stderr: {$stderr}
backend-analyzer-not-set = Analyzer-FST ikke satt
backend-generator-not-set = Generator-FST ikke satt
backend-command-not-executable = Lookup-kommando '{$cmd}' kunne ikke kjøres: {$error}
backend-command-not-found = Lookup-kommando '{$cmd}' finnes ikke eller kan ikke kjøres. Sjekk at den er installert og i PATH.
backend-command-error = Kan ikke kjøre lookup-kommando '{$cmd}': {$error}

# Pool error messages
pool-io-error = IO-feil ved lesing fra prosess: {$error}
pool-missing-stdin = Mangler stdin
pool-missing-stdout = Mangler stdout
pool-process-exited = Prosessen har avsluttet
pool-process-status-error = Feil ved sjekking av prosessstatus
pool-create-analyze-failed = Klarte ikke å opprette analysepulje
pool-create-generate-failed = Klarte ikke å opprette genereringspulje
pool-get-analyze-failed = Klarte ikke å hente prosess fra analysepulje: {$error}
pool-get-generate-failed = Klarte ikke å hente prosess fra genereringspulje: {$error}
pool-validate-analyze-failed = Klarte ikke å hente prosess fra analysepulje for validering: {$error}
pool-validate-generate-failed = Klarte ikke å hente prosess fra genereringspulje for validering: {$error}

# Spec error messages
spec-failed-to-read = Klarte ikke å lese: {$file}
spec-yaml-error = YAML-feil i: {$file}
spec-incomplete-config = Mangelfull eller utydelig Config i {$file}
spec-missing-config = Config mangler
spec-missing-gen = Fant verken HFST.Gen eller Foma.Gen i Config
spec-missing-hfst = Config.hfst mangler
spec-missing-hfst-gen = Config.hfst.Gen mangler
spec-missing-foma = Config.foma mangler
spec-missing-foma-gen = Config.foma.Gen mangler

# Debug messages
debug-batch-lookup = Kjører batch-oppslag med {$count} inndata med FST: {$fst}
debug-batch-completed = Batch-oppslag fullført: {$inputs} inndata prosessert, {$results} totale resultater
debug-pool-batch = Pulje-prosess batch: prosesserer {$count} inndata
debug-pool-completed = Pulje-batch fullført: {$inputs} inndata prosessert, {$results} totale resultater

# Engine messages
engine-not-processed = Ikke prosessert
engine-batch-analyze-error = Batch-analysefeil: {$error}
engine-batch-generate-error = Batch-genereringsfeil: {$error}