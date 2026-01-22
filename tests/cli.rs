use std::process::{Command, Output};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_duckdb-slt"))
}

fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    path.to_string_lossy().to_string()
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
fn mismatch_exits_2() {
    let out = bin().arg(fixture("fail.slt")).output().unwrap();
    assert_exit_code(&out, 2);
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
fn json_output_is_valid() {
    let out = bin()
        .args(["--format", "json", &fixture("pass.slt")])
        .output()
        .unwrap();
    assert_exit_0(&out);

    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();

    assert!(v.get("status").is_some());
    assert_eq!(v.get("exit_code").unwrap().as_u64(), Some(0));
    assert!(v.get("counts").is_some());
}

#[test]
fn no_fail_fast_continues_to_next_file() {
    let pass = fixture("pass2.slt");
    let fail = fixture("fail.slt");

    let out = bin()
        .args(["--no-fail-fast", &fail, &pass])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(&format!("PASS {pass}")));
}

#[test]
fn extensions_install_then_load_in_order() {
    let out = bin()
        .args([
            "--extensions",
            "json",
            "--extensions",
            "json",
            &fixture("pass.slt"),
        ])
        .output()
        .unwrap();

    assert_exit_0(&out);

    let stderr = String::from_utf8(out.stderr).unwrap();
    let lines: Vec<&str> = stderr
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("INSTALL ") || l.starts_with("LOAD "))
        .collect();

    assert_eq!(
        lines,
        vec!["INSTALL json", "LOAD json", "INSTALL json", "LOAD json"]
    );
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
    let ext = fixture("extensions/quack.duckdb_extension");
    if !std::path::Path::new(&ext).exists() {
        // Local-only integration test: requires the quack.duckdb_extension fixture.
        return;
    }

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
            "--extensions",
            "spatial@core_nightly",
            &fixture("extensions_repositories.slt"),
        ])
        .output()
        .unwrap();

    assert_exit_0(&out);
}
