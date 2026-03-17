# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4](https://github.com/kkruglik/verdict/compare/verdict-core-v0.1.3...verdict-core-v0.1.4) - 2026-03-17

### Other

- Merge pull request #23 from kkruglik/feature/19-fix-docs
- add crates.io badge to verdict-core README

## [0.1.3](https://github.com/kkruglik/verdict/compare/verdict-core-v0.1.2...verdict-core-v0.1.3) - 2026-03-17

### Fixed

- fix constraints table in verdict-core README

## [0.1.2](https://github.com/kkruglik/verdict/compare/verdict-core-v0.1.1...verdict-core-v0.1.2) - 2026-03-17

### Other

- Merge branch 'main' into feature/18-fix-ci-issues
- add readme field to verdict-core Cargo.toml
- add README for verdict-core

## [0.1.1](https://github.com/kkruglik/verdict/compare/verdict-core-v0.1.0...verdict-core-v0.1.1) - 2026-03-17

### Other

- release v0.1.0

## [0.1.0](https://github.com/kkruglik/verdict/releases/tag/verdict-core-v0.1.0) - 2026-03-17

### Fixed

- fix operand handling in python bindings and clean up column ops
- fix sum returning Some(0) for all-null columns, add edge case tests
- fixed clippy issues

### Other

- add crates.io metadata to verdict-core
- applied fmt
- update tests for ValidationReport and failed_values
- add optional json feature flag to verdict-core
- re-export Keep enum from dataset
- add duplicated method to column types
- add ValidationReport failed_values and ValidateConfig
- update csv bench to use Path::new for from_csv calls
- update csv tests to use Path::new for from_csv calls
- change from_csv signature to accept &Path instead of &str
- ignore mixed Between operand tests until feature is implemented
- remove unused FloatColumn import in rules/mod.rs
- Merge branch 'main' into feature/11-code-refactoring
- applied cargo fmt
- reorder type and function definitions for readability
- add criterion benchmarks for csv loading
- add tests for csv shape mismatch error
- optimize csv loading: capacity hints, index-based loop, shape validation
- applied formatting
- optimize csv loader: single-pass ColBuilder, 512KB buffer, strict parse_bool
- update fixtures: replace yes/no with true/false for strict parse_bool
- update column pair tests to use rule builder syntax
- add Display for Constraint, derive Default for RuleBuilder
- add Debug derive to Column and typed column structs
- add column pair tests and update existing tests for Operand variants
- add Operand enum and RuleBuilder, update Constraint variants to use Operand
- add ComparableOps impls for typed column pairs and StringOps for Column
- remove direct comparison/string methods from impl Column, add Debug derives
- Add Clone derive to Constraint, Rule and Schema types
- Add Clone derive to Column types for PyO3 interop
- applied fmt
- Extract CSV loading into feature-gated module in verdict-core
- Add Display impl for ValidationResult and fix clippy issues
- applied fmt
- Add unique_count, duplicates_count, is_in column ops and validation tests
- Implement check functions and wire up validate dispatch
- Add validation rules module with Rule, Constraint, and validate() scaffold
- applied cargo fmt
- Add StringOps impl, Column enum delegation, and comprehensive tests
- Add ComparableOps implementations for Int, Float and Str columns
- Add NumericOps implementation for FloatColumn
- Add column ops traits and delegate common methods to typed columns
- added new tests for accessors
- refactored dataset mod, added new methods
- added flags crates
- added tests
- added fixtures
- added lib.rs
- created errors
- created dataset structs

### Removed

- removed line
