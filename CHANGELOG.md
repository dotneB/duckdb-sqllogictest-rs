# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.8](https://github.com/dotneB/duckdb-sqllogictest-rs/compare/v0.1.7...v0.1.8) - 2026-03-24

### <!-- 0 -->⛰️ Features

- bump duckdb to 1.5.1

### <!-- 6 -->🧪 Testing

- disable quack tests

## [0.1.7](https://github.com/dotneB/duckdb-sqllogictest-rs/compare/v0.1.6...v0.1.7) - 2026-03-12

### <!-- 0 -->⛰️ Features

- add support for skipif and onlyif keywords
- bump duckdb to 1.5.0

### <!-- 2 -->🚜 Refactor

- split main in more focused modules

### <!-- 7 -->🤖 CI

- bump toolchain to 1.94

## [0.1.6](https://github.com/dotneB/duckdb-sqllogictest-rs/compare/v0.1.5...v0.1.6) - 2026-01-27

### <!-- 0 -->⛰️ Features

- pin duckdb to 1.4.4

### <!-- 6 -->🧪 Testing

- proper amd64 version

## [0.1.5](https://github.com/dotneB/duckdb-sqllogictest-rs/compare/v0.1.4...v0.1.5) - 2026-01-26

### <!-- 0 -->⛰️ Features

- pin duckdb to 1.4.3

### <!-- 1 -->🐛 Bug Fixes

- TIMETZ stringification

## [0.1.4](https://github.com/dotneB/duckdb-sqllogictest-rs/compare/v0.1.3...v0.1.4) - 2026-01-26

### <!-- 0 -->⛰️ Features

- add validation to the number of columns

## [0.1.3](https://github.com/dotneB/duckdb-sqllogictest-rs/compare/v0.1.2...v0.1.3) - 2026-01-26

### <!-- 0 -->⛰️ Features

- add support for the 'require' keyword

### <!-- 3 -->📚 Documentation

- update description

## [0.1.2](https://github.com/dotneB/duckdb-sqllogictest-rs/compare/v0.1.1...v0.1.2) - 2026-01-24

### <!-- 7 -->🤖 CI

- debug release with cargo dist

## [0.1.1](https://github.com/dotneB/duckdb-sqllogictest-rs/compare/v0.1.0...v0.1.1) - 2026-01-24

### <!-- 7 -->🤖 CI

- isolate test runs duckdb home
- add release-plz.toml
- ignore docs
- integrate cargo dist

### <!-- 8 -->⚙️ Miscellaneous Tasks

- update readme
- docs.rs failing because it's only a binary
- change changelog generator config

## [0.1.0](https://github.com/dotneB/duckdb-sqllogictest-rs/releases/tag/v0.1.0) - 2026-01-23

### <!-- 0 -->⛰️ Features

- initial project setup
- initial cli contract
- initial duckdb connection and extension parsing
- update duckdb-driver value formatting
- support files glob
- uses sqllogictest-rs formatting
- more duckdb type formatting

### <!-- 1 -->🐛 Bug Fixes

- improve output for tests

### <!-- 6 -->🧪 Testing

- guard extension test on platform

### <!-- 7 -->🤖 CI

- release-plz init
- change to release-plz

### <!-- 8 -->⚙️ Miscellaneous Tasks

- update readme
- update deps and toolchain
