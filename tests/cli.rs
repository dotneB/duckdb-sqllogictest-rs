use std::process::{Command, Output};

fn bin() -> Command {
    // DuckDB stores extension state in ~/.duckdb. On Windows CI, multiple tests
    // can race creating it (reported as "Cannot create a file when that file already exists").
    // Isolate each CLI invocation to a fresh temp home to avoid global state.
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_duckdb-slt"));

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let home =
        std::env::temp_dir().join(format!("duckdb-slt-home-{}-{}", std::process::id(), nanos));
    std::fs::create_dir_all(&home).unwrap();

    cmd.env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("DUCKDB_HOME", &home);

    cmd
}

fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    path.to_string_lossy().to_string()
}

fn extension_relpath(name: &str) -> String {
    let platform_dir = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "windows_amd64".to_string(),
        ("linux", "x86_64") => "linux_amd64".to_string(),
        ("macos", "aarch64") => "osx_arm64".to_string(),
        (os, arch) => format!("{}_{}", os, arch),
    };
    format!("extensions/{}/{}.duckdb_extension", platform_dir, name)
}

fn require_extension_fixture(name: &str) -> Option<String> {
    let ext = fixture(&extension_relpath(name));
    if !std::path::Path::new(&ext).exists() {
        return None;
    }
    Some(ext)
}

fn display_path(path: &str) -> String {
    let cwd = std::env::current_dir().unwrap();
    let p = std::path::Path::new(path);
    match p.strip_prefix(&cwd) {
        Ok(rel) => rel.to_string_lossy().to_string(),
        Err(_) => path.to_string(),
    }
}

fn fixtures_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn assert_exit_0(out: &Output) {
    assert_exit_code(out, 0);
}

fn assert_exit_code(out: &Output, code: i32) {
    if out.status.code() != Some(code) {
        eprintln!("exit status: {:?}", out.status.code());
        eprintln!("--- stdout ---\n{}", String::from_utf8_lossy(&out.stdout));
        eprintln!("--- stderr ---\n{}", String::from_utf8_lossy(&out.stderr));
    }
    assert_eq!(out.status.code(), Some(code));
}

fn assert_stderr_nonempty(out: &Output) {
    if out.stderr.is_empty() {
        eprintln!("exit status: {:?}", out.status.code());
        eprintln!("--- stdout ---\n{}", String::from_utf8_lossy(&out.stdout));
        eprintln!("--- stderr ---\n{}", String::from_utf8_lossy(&out.stderr));
    }
    assert!(!out.stderr.is_empty());
}

#[test]
fn pass_exits_0() {
    let out = bin().arg(fixture("pass.slt")).output().unwrap();
    assert_exit_0(&out);
}

#[test]
fn canonical_values_pass() {
    let out = bin().arg(fixture("canonical_values.slt")).output().unwrap();
    assert_exit_0(&out);
}

#[test]
fn type_fidelity_fixture_passes() {
    let out = bin().arg(fixture("type_fidelity.slt")).output().unwrap();
    assert_exit_0(&out);
}

#[test]
fn zero_rows_query_pass() {
    let out = bin().arg(fixture("zero_rows.slt")).output().unwrap();
    assert_exit_0(&out);
}

#[test]
fn statement_returning_rows_pass() {
    let out = bin()
        .arg(fixture("statement_returning_rows.slt"))
        .output()
        .unwrap();
    assert_exit_0(&out);
}

#[test]
fn mismatch_exits_2() {
    let out = bin().arg(fixture("fail.slt")).output().unwrap();
    assert_exit_code(&out, 2);
}

#[test]
fn mismatch_output_includes_record_and_sql() {
    let out = bin().arg(fixture("fail_labeled.slt")).output().unwrap();
    assert_exit_code(&out, 2);

    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("record:"));
    assert!(stderr.contains("name=my_record"));
    assert!(stderr.contains("SELECT 1;"));
}

#[test]
fn invalid_path_exits_1() {
    let out = bin().arg(fixture("does-not-exist.slt")).output().unwrap();
    assert_exit_code(&out, 1);
}

#[test]
fn help_exits_0() {
    let out = bin().arg("--help").output().unwrap();
    assert_exit_0(&out);
}

#[test]
fn pass_outputs_relative_path_when_possible() {
    let pass = fixture("pass.slt");
    let out = bin().arg(&pass).output().unwrap();
    assert_exit_0(&out);

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(&format!("test {} ... ok", display_path(&pass))));
    assert!(!stdout.contains(env!("CARGO_MANIFEST_DIR")));
}

#[test]
fn unsigned_extensions_disabled_by_default() {
    let out = bin()
        .arg(fixture("unsigned_extensions_disabled.slt"))
        .output()
        .unwrap();
    assert_exit_0(&out);
}

#[test]
fn allow_unsigned_extensions_flag_enables_setting() {
    let out = bin()
        .args([
            "--allow-unsigned-extensions",
            &fixture("unsigned_extensions_enabled.slt"),
        ])
        .output()
        .unwrap();
    assert_exit_0(&out);
}

#[test]
fn extensions_empty_spec_exits_1() {
    let out = bin()
        .args(["--extensions", "", &fixture("pass.slt")])
        .output()
        .unwrap();
    assert_exit_code(&out, 1);
    assert_stderr_nonempty(&out);
}

#[test]
fn extension_path_can_install_load_and_run_query() {
    // Local-only integration test: requires the built extension fixture.
    let Some(ext) = require_extension_fixture("quack") else {
        return;
    };

    let out = bin()
        .args([
            "--allow-unsigned-extensions",
            "--extensions",
            &ext,
            &fixture("quack_hello.slt"),
        ])
        .output()
        .unwrap();
    assert_exit_0(&out);
}

#[test]
fn extensions_can_install_from_core_and_core_nightly() {
    let out = bin()
        .args([
            "--extensions",
            "httpfs@core",
            // "--extensions",
            // "spatial@core_nightly",
            &fixture("extensions_repositories.slt"),
        ])
        .output()
        .unwrap();

    assert_exit_0(&out);
}

#[test]
fn glob_expands_files_and_runs_each() {
    let pattern = fixtures_dir()
        .join("pass*.slt")
        .to_string_lossy()
        .to_string();
    let out = bin().arg(pattern).output().unwrap();
    assert_exit_0(&out);
}

#[test]
fn require_missing_extension_is_ignored_and_file_runs() {
    let path = fixture("require_missing_extension.slt");
    let out = bin().arg(&path).output().unwrap();
    assert_exit_0(&out);
}

#[test]
fn require_does_not_break_failure_location_reporting() {
    let path = fixture("require_fail.slt");
    let out = bin().arg(&path).output().unwrap();
    assert_exit_code(&out, 2);

    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains(&format!("  at: {}:3", display_path(&path))));
}

#[test]
fn require_can_load_extension_by_name_after_install() {
    // Local-only integration test: requires the built extension fixture.
    let Some(ext) = require_extension_fixture("quack") else {
        return;
    };

    let out = bin()
        .args([
            "--allow-unsigned-extensions",
            "--extensions",
            &ext,
            &fixture("require_quack_hello.slt"),
        ])
        .output()
        .unwrap();

    assert_exit_0(&out);
}

#[test]
fn query_column_count_too_few_fails_with_expected_actual_counts() {
    let path = fixture("column_count_too_few.slt");
    let out = bin().arg(&path).output().unwrap();
    assert_exit_code(&out, 2);

    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Expected 3 columns, but got 1 columns"));
}

#[test]
fn query_column_count_too_many_fails_with_expected_actual_counts() {
    let path = fixture("column_count_too_many.slt");
    let out = bin().arg(&path).output().unwrap();
    assert_exit_code(&out, 2);

    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Expected 1 columns, but got 2 columns"));
}
