# morph-test-rs aka `morph-test2`

Rust version of [morph-test](https://github.com/divvun/morph-test). It is a feature complete, drop-in replacement for `morph-test`, with the command name `morph-test2`.

Main features:
- faster, by utilising parallel processing
- memory safe (Rust)
- easily installable binaries (coming)

## Installation

- clone this repo
- cd into the repo
- `cargo install --path .`

Coming: downloadable binaries.

## Usage

For usage instructions, see the link above to `morph-test`.

### Changes or additional features compared to `morph-test`

- in test reports, test suites are numbered starting from 1. `morph-test` is starting from 0.
- when running individual tests, one can refer to the tests both by name (ID in the yaml file) and by number. To list all available tests, specify one of `0`, `null` or `liste` as the name of the test.
- verbose mode gives some more information than the original `morph-test`
- the tool can take a file name pattern or a directory as argument, and will then run all test files matching the pattern or in the specified directory
- the flag `--pool` enables further multiprocessing features and thus more speed-up

# License

Licensed under [CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/).

# Contribution

Fork and PR on Github.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be licensed as above, without any additional terms or conditions.
