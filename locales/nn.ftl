# CLI help text
cli-about = Morfologisk testkøyrar (overflate/analyse og leksikalsk/generering)
cli-backend = Vel fst-format/seksjon (hfst eller foma) [alias: -S/--section]
cli-generator = Overstyr generator-FST (.hfstol for HFST, .foma for Foma) [alias: --gen]
cli-analyser = Overstyr analyse-FST (.hfstol for HFST, .foma for Foma) [alias: --morph, --analyzer]
cli-silent = Stille modus: inga utskrift, og demp stderr frå lookup
cli-lookup-tool = Overstyr lookup-kommando (t.d. hfst-optimised-lookup, flookup) [alias: --app]
cli-ignore-extra = For analysetestar: godkjenn når alle forventa analysar finst, sjølv om det finst ekstra analysar
cli-color = Tving fargar på (standard)
cli-no-color = Slå av fargar i rapporten (overstyrer --color)
cli-verbose = Vis metadata (lookup med full sti, generator/analyzer med fulle stiar, versjon) og framdriftsmeldingar. Viser òg 'EXTRA'-analyser for analyse-PASS når -i er aktiv.
cli-surface = Køyr berre analysetestar (overflateform → analyse)
cli-lexical = Køyr berre genereringstestar (analyse → overflateform)
cli-hide-fails = Skjul feil (FAIL), vis berre godkjende (PASS)
cli-hide-passes = Skjul godkjende (PASS), vis berre feil (FAIL)
cli-test = Køyr berre oppgjeven test: nummer 1..N, tittel „Gruppe (Lexical/Generation|Surface/Analysis)" eller berre gruppenamnet frå YAML. Spesialtestnamn: 0, 'null' eller 'liste' listar alle tilgjengelege testsett.
cli-output = Rapportformat: normal | compact | terse | final (standard: normal)
cli-serial = Bruk seriell køyring i staden for parallellprosessering (parallell er standard)

# Directions and modes  
direction-generate = Leksikalsk/Generering
direction-analyze = Overflate/Analyse
mode-analyze-only = Berre analyse
mode-generate-only = Berre generering
mode-all = Alle

# Error messages
error-no-tests-after-filter = Ingen testar tilgjengelege etter filtrering.
error-invalid-test-number = Ugyldig testnummer {$number}. Gyldig område: 1..{$max}.
error-test-not-found = Fann ikkje test med ID/tittel: {$test}
error-validation-failed = Feil: {$error}

# Info messages
info-version     = {$name} v{$version}
info-suite       = Suite          : {$name}
info-lookup-tool = Lookup-kommando: {$path}
info-generator   = Generator      : {$path}
info-analyzer    = Analysator     : {$path}
info-starting-tests = Byrjar å testa ({$count} testar, modus: {$mode}) (batch processing)...
info-starting-parallel = Suite: {$name} (parallel processing)...
info-finished = Ferdig: godkjende {$passed}, feila {$failed}. Skriv rapport...
info-all-finished = Alle testkøyringar ferdige. I alt: {$total}, Godkjende: {$passed}, Feila: {$failed}

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
report-total-summary = I alt godkjent: {$passes}, I alt feila: {$fails}, I alt: {$total}
report-final-counts = {$passes}/{$fails}/{$total}

# Backend error messages
backend-failed-to-start = Klarte ikkje å starta '{$cmd}'
backend-missing-stdin = Manglar stdin
backend-process-failed = Lookup-prosess feila med status {$status}
backend-timeout = Lookup-tidsavbrot etter {$seconds} s
backend-process-failed-stderr = Lookup-prosess feila med status {$status}
Stderr: {$stderr}
backend-analyzer-not-set = Analysator-FST ikkje sett
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
spec-missing-gen = Fann korkje HFST.Gen eller Foma.Gen i Config
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

# I18N system messages  
i18n-init-failed = Klarte ikkje å initialisere lokaliseringsystemet
i18n-missing-key = Manglar omsetjingsnøkkel: {$key}
i18n-missing-format = MANGLAR: {$key}
i18n-parse-failed = Klarte ikkje å parse: {$input}

# CLI error messages
cli-error-missing-args = Desse obligatoriske argumenta vart ikkje oppgjevne:
cli-error-usage = Bruk:
cli-error-help-info = For meir informasjon, prøv '--help'.
cli-error-invalid-value = Ugyldig verdi '{$value}' for '{$arg}'
cli-error-unexpected-arg = Fann argument '{$arg}' som ikkje var venta, eller ikkje er gyldig i denne konteksten
cli-error-label = feil:
cli-tip-label = tips:
cli-unexpected-argument = uventa argument