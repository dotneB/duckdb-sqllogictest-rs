use std::process::Command;

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

#[test]
fn pass_exits_0() {
    let out = bin().arg(fixture("pass.slt")).output().unwrap();
    assert!(out.status.success());
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn mismatch_exits_2() {
    let out = bin().arg(fixture("fail.slt")).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn invalid_path_exits_1() {
    let out = bin().arg(fixture("does-not-exist.slt")).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn help_exits_0() {
    let out = bin().arg("--help").output().unwrap();
    assert!(out.status.success());
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn json_output_is_valid() {
    let out = bin()
        .args(["--format", "json", &fixture("pass.slt")])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));

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

    assert_eq!(out.status.code(), Some(0));

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
