# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.1.0 (2025-12-04)


### ⚠ BREAKING CHANGES

* ChippClient::new() returns Result instead of panicking ([#6](https://github.com/paulbreuler/chipp-rs/issues/6))

### Features

* adapt .augment configuration for chipp-rs SDK ([1331af3](https://github.com/paulbreuler/chipp-rs/commit/1331af304075fe3ea252b5333cbf52dc311bb4ad))
* add code coverage enforcement to CI/CD ([c059269](https://github.com/paulbreuler/chipp-rs/commit/c05926960ec3a02849b7c0e54ffe8256edce6b7c))
* add comprehensive error handling example ([e9a73b4](https://github.com/paulbreuler/chipp-rs/commit/e9a73b45a3808eb9a3cf69a5b0dd67c7af40d102))
* add git-cliff configuration for automated CHANGELOG generation ([7677b62](https://github.com/paulbreuler/chipp-rs/commit/7677b625f5f160b866068d8d5cbd9ae92abfb1ae))
* add justfile with quality command and common tasks ([af3362c](https://github.com/paulbreuler/chipp-rs/commit/af3362cbb75daa21886c5c2c0bf13fd9a47693ec))
* add pre-commit configuration ([830a99b](https://github.com/paulbreuler/chipp-rs/commit/830a99b73bdb52a5ce725178ad01e3fda27b879c))
* add quality check script for local development ([ee1380e](https://github.com/paulbreuler/chipp-rs/commit/ee1380e0b694ad1b86f16ddce297ecda58e67b94))
* **augment:** add Augment command definitions ([0241659](https://github.com/paulbreuler/chipp-rs/commit/0241659025a18d98eeb65249197b1d7a83ab2a60))
* **augment:** add Augment configuration documentation ([34e211e](https://github.com/paulbreuler/chipp-rs/commit/34e211efc92165e951b32dc96849046580166e13))
* **augment:** add core SDK development rules ([11f9d64](https://github.com/paulbreuler/chipp-rs/commit/11f9d646e7c1d3267bee156daf6d681868ee3002))
* **augment:** add supporting SDK development rules ([f9b09cf](https://github.com/paulbreuler/chipp-rs/commit/f9b09cfe1dc4b5c54f3ff50236b6df86ee8b6b3b))
* comprehensive SDK improvements - TDD test suite, coverage enforcement, and tooling ([85c4e19](https://github.com/paulbreuler/chipp-rs/commit/85c4e196397c3a4275ea62d0cde6ccb212169982))
* comprehensive SDK improvements - TDD test suite, coverage enforcement, and tooling ([85c4e19](https://github.com/paulbreuler/chipp-rs/commit/85c4e196397c3a4275ea62d0cde6ccb212169982))
* implement comprehensive TDD test suite, achieve 98.80% coverage ([410f0c9](https://github.com/paulbreuler/chipp-rs/commit/410f0c97bd59a28889845a151eb4afd0081a91e5))
* implement retry logic with exponential backoff ([e79e4d7](https://github.com/paulbreuler/chipp-rs/commit/e79e4d7673a788e66e37acde5d8a6b5baee3e5ed))
* improve coverage reporting with --show-missing-lines ([daf1399](https://github.com/paulbreuler/chipp-rs/commit/daf1399388859e384a7708f4ab77c807a95b869c))


### Bug Fixes

* ChippClient::new() returns Result instead of panicking ([#6](https://github.com/paulbreuler/chipp-rs/issues/6)) ([d98d5e9](https://github.com/paulbreuler/chipp-rs/commit/d98d5e92ee9d4c651765bd711b0d618bcc6d1a44))
* remove broken prerelease config ([909c959](https://github.com/paulbreuler/chipp-rs/commit/909c959de426ab805656d6a678392e11e13e2db0))

## [Unreleased]

### Added

- Non-streaming chat completions via `chat()`
- Streaming chat completions via `chat_stream()` with Server-Sent Events
- Automatic session management with `chatSessionId` tracking
- Retry logic with configurable exponential backoff
- Configurable request timeouts
- Comprehensive error types with `ChippClientError`
