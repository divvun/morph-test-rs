# CLI help text
cli-about = Morfologisk testløypar (overflate/analyser og leksikalsk/generering)
cli-backend = Vel backend/seksjon (hfst eller foma) [alias: -S/--section]
cli-generator = Overstyr generator-FST (.hfstol for HFST, .foma for Foma) [alias: --gen]
cli-analyser = Overstyr analyser-FST (.hfstol for HFST, .foma for Foma) [alias: --morph, --analyzer]
cli-silent = Stille modus: ingen utskrift, og demp stderr frå lookup
cli-lookup-tool = Overstyr lookup-kommando (t.d. hfst-optimised-lookup, flookup) [alias: --app]
cli-ignore-extra = Analyze-testar: godkjenn når alle forventa analysar finst, sjølv om det finst ekstra analysar
cli-color = Tving fargar på (standard er fargar på)
cli-no-color = Slå av fargar i rapporten (overstyrer --color)
cli-verbose = Vis metadata (lookup med full sti, generator/analyzer med fulle stiar, versjon) og framdriftsmeldingar. Viser òg 'EXTRA' for Analyze-PASS når -i er aktiv.
cli-surface = Køyr berre analysetestar (surface form → analyses)
cli-lexical = Køyr berre genereringstestar (lexical tags → surface forms)
cli-hide-fails = Skjul feil (FAIL), vis berre gjennomgåtte (PASS)
cli-hide-passes = Skjul gjennomgåtte (PASS), vis berre feil (FAIL)
cli-test = Køyr berre angitt test: nummer 1..N, tittel „Gruppe (Lexical/Generation|Surface/Analysis)" eller berre gruppenamnet frå YAML. Spesial: 0, 'null' eller 'liste' listar alle tilgjengelege testar og avsluttar.
cli-output = Rapportformat: compact | terse | final | normal (standard: normal)
cli-serial = Bruk seriell køyring i staden for parallell processing (standardverdi er parallell)

# Directions and modes  
direction-generate = Leksikalsk/Generering
direction-analyze = Overflate/Analyse
mode-analyze-only = Berre analyse
mode-generate-only = Berre generering
mode-all = Alle

# Error messages
error-no-tests-after-filter = Ingen testar tilgjengeleg etter filtrering.
error-invalid-test-number = Ugyldig testnummer {$number}. Gyldig område: 1..{$max}.
error-test-not-found = Fann ikkje test med ID/tittel: {$test}
error-validation-failed = Feil: {$error}

# Info messages
info-version = {$name} v{$version}
info-suite = Suite         : {$name}
info-lookup-tool = Lookup tool   : {$path}
info-generator = Generator     : {$path}
info-analyzer = Analyzer      : {$path}
info-starting-tests = Startar testing ({$count} testar, modus: {$mode}) (batch processing)...
info-starting-parallel = Suite: {$name} (parallel processing)...
info-finished = Ferdig: passed {$passed}, failed {$failed}. Skriv rapport...
info-all-finished = Alle testkøyringar ferdige. Total: {$total}, Passed: {$passed}, Failed: {$failed}

# Test listing
available-tests = Tilgjengelege testar:
test-list-item = {$index}: {$group} ({$direction})

# Report labels
report-pass = PASS
report-fail = FAIL
report-extra = EXTRA
report-error = Feil
report-expected = Forventa
report-got = Fekk
report-unexpected-results = Uventa resultat: {$results}
report-no-lexical = <Ingen leksikalsk/generering>
report-no-surface = <Ingen overflate/analyse>
report-test-header = Test {$index}: {$group} ({$direction})
report-test-summary = Test {$index} - Bestått: {$passes}, Feila: {$fails}, Totalt: {$total}
report-total-summary = Totalt bestått: {$passes}, Totalt feila: {$fails}, Totalt: {$total}
report-final-counts = {$passes}/{$fails}/{$total}

# Backend error messages
backend-failed-to-start = Klarte ikkje å starta '{$cmd}'
backend-missing-stdin = Manglar stdin
backend-process-failed = Lookup-prosess feila med status {$status}
backend-timeout = Lookup tidsavbrot etter {$seconds} s
backend-process-failed-stderr = Lookup-prosess feila med status {$status}
Stderr: {$stderr}
backend-analyzer-not-set = Analyzer-FST ikkje sett
backend-generator-not-set = Generator-FST ikkje sett
backend-command-not-executable = Lookup-kommando '{$cmd}' kunne ikkje køyrast: {$error}
backend-command-not-found = Lookup-kommando '{$cmd}' finst ikkje eller kan ikkje køyrast. Sjekk at den er installert og i PATH.
backend-command-error = Kan ikkje køyre lookup-kommando '{$cmd}': {$error}

# Pool error messages
pool-io-error = IO-feil ved lesing frå prosess: {$error}
pool-missing-stdin = Manglar stdin
pool-missing-stdout = Manglar stdout
pool-process-exited = Prosessen har avslutta
pool-process-status-error = Feil ved sjekking av prosessstatus
pool-create-analyze-failed = Klarte ikkje å oppretta analysepulje
pool-create-generate-failed = Klarte ikkje å oppretta genereringspulje
pool-get-analyze-failed = Klarte ikkje å henta prosess frå analysepulje: {$error}
pool-get-generate-failed = Klarte ikkje å henta prosess frå genereringspulje: {$error}
pool-validate-analyze-failed = Klarte ikkje å henta prosess frå analysepulje for validering: {$error}
pool-validate-generate-failed = Klarte ikkje å henta prosess frå genereringspulje for validering: {$error}

# Spec error messages
spec-failed-to-read = Klarte ikkje å lesa: {$file}
spec-yaml-error = YAML-feil i: {$file}
spec-incomplete-config = Mangelfull eller utydeleg Config i {$file}
spec-missing-config = Config manglar
spec-missing-gen = Fann verken HFST.Gen eller Foma.Gen i Config
spec-missing-hfst = Config.hfst manglar
spec-missing-hfst-gen = Config.hfst.Gen manglar
spec-missing-foma = Config.foma manglar
spec-missing-foma-gen = Config.foma.Gen manglar

# Debug messages
debug-batch-lookup = Køyrer batch-oppslag med {$count} inndata med FST: {$fst}
debug-batch-completed = Batch-oppslag fullført: {$inputs} inndata prosesserte, {$results} totale resultat
debug-pool-batch = Pulje-prosess batch: prosesserer {$count} inndata
debug-pool-completed = Pulje-batch fullført: {$inputs} inndata prosesserte, {$results} totale resultat

# Engine messages
engine-not-processed = Ikkje prosessert
engine-batch-analyze-error = Batch-analysefeil: {$error}
engine-batch-generate-error = Batch-genereringsfeil: {$error}