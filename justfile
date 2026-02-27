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