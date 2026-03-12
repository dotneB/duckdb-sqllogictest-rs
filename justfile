debug:
  cargo build

release:
  cargo build --release

test: debug
  cargo test

test-release: release
  cargo test

check:
  cargo fmt --check
  cargo clippy -- -D warnings

check-fix:
  cargo fmt
  cargo clippy --fix

dev: check test

full: check test test-release

download-fixtures version:
  curl -L "http://community-extensions.duckdb.org/v{{version}}/windows_amd64/quack.duckdb_extension.gz" -o tests/fixtures/extensions/windows_amd64/quack.duckdb_extension.gz
  gunzip -f tests/fixtures/extensions/windows_amd64/quack.duckdb_extension.gz
  curl -L "http://community-extensions.duckdb.org/v{{version}}/linux_amd64/quack.duckdb_extension.gz" -o tests/fixtures/extensions/linux_amd64/quack.duckdb_extension.gz
  gunzip -f tests/fixtures/extensions/linux_amd64/quack.duckdb_extension.gz
  curl -L "http://community-extensions.duckdb.org/v{{version}}/osx_arm64/quack.duckdb_extension.gz" -o tests/fixtures/extensions/osx_arm64/quack.duckdb_extension.gz
  gunzip -f tests/fixtures/extensions/osx_arm64/quack.duckdb_extension.gz
