# CLI help text
cli-about = Morfologisk testkjører (overflate/analyse og leksikalsk/generering)
cli-backend = Velg fst-format/seksjon (hfst eller foma) [alias: -S/--section]
cli-generator = Overstyr generator-FST (.hfstol for HFST, .foma for Foma) [alias: --gen]
cli-analyser = Overstyr analysator-FST (.hfstol for HFST, .foma for Foma) [alias: --morph, --analyzer]
cli-silent = Stille modus: ingen utskrift, og demp stderr fra lookup
cli-lookup-tool = Overstyr lookup-kommando (f.eks. hfst-optimised-lookup, flookup) [alias: --app]
cli-ignore-extra = For analysetester: godkjenn når alle forventede analyser finnes, selv om det finnes ekstra analyser
cli-color = Tving farger på (standard er farger på)
cli-no-color = Slå av farger i rapporten (overstyrer --color)
cli-verbose = Vis metadata (lookup med full sti, generator/analysator med fulle stier, versjon) og framdriftsmeldinger. Viser også 'EXTRA' for Analyse-PASS når -i er aktiv.
cli-surface = Kjør bare analysetester (overflateform → analyse)
cli-lexical = Kjør bare genereringstester (analyse → overflateform)
cli-hide-fails = Skjul feil (FAIL), vis bare godkjente (PASS)
cli-hide-passes = Skjul godkjente (PASS), vis bare feil (FAIL)
cli-test = Kjør bare angitt test: nummer 1..N, tittel „Gruppe (Lexical/Generation|Surface/Analysis)" eller bare gruppenavnet fra YAML. Spesialtestnavn: 0, 'null' eller 'liste' lister alle tilgjengelige tester.
cli-output = Rapportformat: normal | compact | terse | final (standard: normal)
cli-serial = Bruk seriell kjøring i stedet for parallell prosessering (parallell er standard)

# Directions and modes  
direction-generate = Leksikalsk/Generering
direction-analyze = Overflate/Analyse
mode-analyze-only = Bare analyse
mode-generate-only = Bare generering
mode-all = Alle

# Error messages
error-no-tests-after-filter = Ingen tester tilgjengelige etter filtrering.
error-invalid-test-number = Ugyldig testnummer {$number}. Gyldig område: 1..{$max}.
error-test-not-found = Fant ikke test med ID/tittel: {$test}
error-validation-failed = Feil: {$error}

# Info messages
info-version     = {$name} v{$version}
info-suite       = Suite          : {$name}
info-lookup-tool = Lookup-kommando: {$path}
info-generator   = Generator      : {$path}
info-analyzer    = Analysator     : {$path}
info-starting-tests = Begynner å teste ({$count} tester, modus: {$mode}) (batch processing)...
info-starting-parallel = Suite: {$name} (parallellprosessering)...
info-finished = Ferdig: godkjente {$passed}, feila {$failed}. Skriver rapport...
info-all-finished = Alle testkjøringer ferdige. Totalt: {$total}, Godkjente: {$passed}, Feila: {$failed}

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
report-test-summary = Test {$index} - Bestått: {$passes}, Feila: {$fails}, Totalt: {$total}
report-total-summary = Totalt bestått: {$passes}, Totalt feila: {$fails}, Totalt: {$total}
report-final-counts = {$passes}/{$fails}/{$total}

# Backend error messages
backend-failed-to-start = Klarte ikke å starte '{$cmd}'
backend-missing-stdin = Mangler stdin
backend-process-failed = Lookup-prosess feilet med status {$status}
backend-timeout = Lookup tidsavbrudd etter {$seconds} s
backend-process-failed-stderr = Lookup-prosess feilet med status {$status}
Stderr: {$stderr}
backend-analyzer-not-set = Analysator-FST ikke satt
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

# I18N system messages
i18n-init-failed = Klarte ikke å initialisere lokaliseringssystemet
i18n-missing-key = Mangler oversettelsesnøkkel: {$key}
i18n-missing-format = MANGLER: {$key}
i18n-parse-failed = Klarte ikke å parse: {$input}

# CLI error messages
cli-error-missing-args = Følgende obligatoriske argumenter ble ikke oppgitt:
cli-error-usage = Bruk:
cli-error-help-info = For mer informasjon, prøv '--help'.
cli-error-invalid-value = Ugyldig verdi '{$value}' for '{$arg}'
cli-error-unexpected-arg = Fant argument '{$arg}' som ikke var forventet, eller ikke er gyldig i denne konteksten
cli-error-label = feil:
cli-tip-label = tips:
cli-unexpected-argument = uventet argument