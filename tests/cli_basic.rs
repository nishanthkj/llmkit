use assert_cmd::prelude::*;
use assert_cmd::cargo::cargo_bin;
use predicates::prelude::*;
use serde_json::Value;
use std::process::Command;

fn run_with_stdin(stdin: &str, args: &[&str]) -> Value {
    let mut cmd = Command::from(cargo_bin("llmkit"));
    cmd.args(args);
    let assert = cmd
        .write_stdin(stdin)
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"^\s*\{"#).unwrap());
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    serde_json::from_str::<Value>(&out).unwrap()
}

#[test]
fn cli_reads_stdin_and_outputs_json() {
    let v = run_with_stdin(r#"{"x":1}"#, &[]);
    assert_eq!(v.get("Format").unwrap(), "json");
    assert!(v.get("json").is_some());
    assert!(v.get("normal").is_some());
}

#[test]
fn cli_targets_limits_output() {
    let v = run_with_stdin(r#"{"x":1}"#, &["--targets", "json,yaml"]);
    assert_eq!(v.get("Format").unwrap(), "json");
    assert!(v.get("json").is_some());
    assert!(v.get("yaml").is_some()); // may be null if feature disabled
    assert!(v.get("csv").is_none());
}

#[test]
fn cli_single_format_flag() {
    let v = run_with_stdin(r#"{"x":1}"#, &["--format", "yaml"]);
    assert_eq!(v.get("Format").unwrap(), "json");
    assert!(v.get("json").is_none());
    assert!(v.get("yaml").is_some());
}
