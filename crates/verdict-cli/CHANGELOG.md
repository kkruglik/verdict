# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.6](https://github.com/kkruglik/verdict/compare/verdict-cli-v0.1.5...verdict-cli-v0.1.6) - 2026-03-22

### Other

- update verdict-cli readme positioning
- update GitHub Actions example to use verdict action

## [0.1.5](https://github.com/kkruglik/verdict/compare/verdict-cli-v0.1.4...verdict-cli-v0.1.5) - 2026-03-22

### Other

- applied cargo fmt
- update cli readme with yaml and date datetime docs
- update cli changelog with yaml and date support
- add yaml schema support and date datetime dtypes to cli

### Added

- YAML schema support — `.yaml` / `.yml` files are detected by extension and parsed with `serde_yaml`; JSON behavior unchanged
- `date` and `datetime` column dtypes with optional `format` field
- `after`, `before`, `between_dates` constraints for date and datetime columns

## [0.1.4](https://github.com/kkruglik/verdict/compare/verdict-cli-v0.1.3...verdict-cli-v0.1.4) - 2026-03-17

### Other

- Merge pull request #23 from kkruglik/feature/19-fix-docs
- improve verdict-cli SEO: title, description, Cargo.toml
- rewrite verdict-cli README with SEO and CI examples

## [0.1.3](https://github.com/kkruglik/verdict/compare/verdict-cli-v0.1.2...verdict-cli-v0.1.3) - 2026-03-17

### Other

- updated the following local packages: verdict-core

## [0.1.2](https://github.com/kkruglik/verdict/compare/verdict-cli-v0.1.1...verdict-cli-v0.1.2) - 2026-03-17

### Other

- Merge branch 'main' into feature/18-fix-ci-issues
- add readme field to verdict-cli Cargo.toml
- add README for verdict-cli

## [0.1.1](https://github.com/kkruglik/verdict/compare/verdict-cli-v0.1.0...verdict-cli-v0.1.1) - 2026-03-17

### Other

- release v0.1.0

## [0.1.0](https://github.com/kkruglik/verdict/releases/tag/verdict-cli-v0.1.0) - 2026-03-17

### Other

- passed value from cli to core
- add crates.io metadata to verdict-cli
- enable json feature in verdict-cli
- update cli to use ValidationReport and ValidateConfig
- apply fmt and positional args, json default format
- addapplied fmt
- add verdict-cli binary crate with JSON output and exit codes
