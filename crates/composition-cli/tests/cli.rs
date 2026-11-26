use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::Value;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn example(relative: &str) -> String {
    workspace_root()
        .join("examples")
        .join(relative)
        .to_string_lossy()
        .into_owned()
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_composition-cli"))
        .args(args)
        .output()
        .expect("run composition-cli")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("utf-8 stdout")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("utf-8 stderr")
}

fn json(output: &Output) -> Value {
    serde_json::from_str(&stdout(output)).expect("valid JSON output")
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn assert_golden(name: &str, actual: &str) {
    let directory = golden_dir();
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join(name);
    let actual = format!("{}\n", actual.trim_end());
    if !path.exists() {
        if std::env::var("UPDATE_GOLDEN").is_ok() {
            fs::write(&path, &actual).unwrap();
        } else {
            panic!(
                "missing golden {}; rerun with UPDATE_GOLDEN=1 to create it",
                path.display()
            );
        }
    }
    let expected = fs::read_to_string(&path).unwrap();
    assert_eq!(actual, expected, "golden mismatch: {}", path.display());
}

#[test]
fn validate_minimal_project_json() {
    let output = run(&["validate", &example("minimal-project"), "--json"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let value = json(&output);
    assert_eq!(value["valid"], true);
    assert_eq!(value["error_count"], 0);
    assert_eq!(value["diagnostics"].as_array().unwrap().len(), 0);
    assert_eq!(
        value["source_digest"],
        "sha256:10cef70e22f16ef050fd9d1ea3d0f4ba3502235641aaf4f88b91260daa34768e"
    );
    assert_golden("validate-minimal.json", &stdout(&output));
}

#[test]
fn validate_connection_errors_json() {
    let output = run(&[
        "validate",
        &example("invalid-projects/connections/errors"),
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let value = json(&output);
    assert_eq!(value["valid"], false);
    let codes: BTreeSet<String> = value["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|diagnostic| diagnostic["code"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        codes,
        [
            "connection-direction-invalid",
            "connection-type-mismatch",
            "connection-duplicate",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
    assert_golden("validate-connections-errors.json", &stdout(&output));
}

#[test]
fn inspect_minimal_project_json() {
    let output = run(&["inspect", &example("minimal-project")]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let value = json(&output);
    assert_eq!(value["project"]["id"], "example.minimal");
    assert_eq!(value["modules"].as_array().unwrap().len(), 2);
    assert_golden("inspect-minimal.json", &stdout(&output));
}

#[test]
fn resolve_minimal_project_json() {
    let output = run(&["resolve", &example("minimal-project")]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let value = json(&output);
    assert_eq!(
        value["model"]["composition"]["instances"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        value["model"]["composition"]["bindings"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_golden("resolve-minimal.json", &stdout(&output));
}

#[test]
fn validate_entry_missing_reports_json_error() {
    let output = run(&[
        "validate",
        &example("invalid-projects/files/entry-missing"),
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let value = json(&output);
    assert_eq!(value["valid"], false);
    assert_eq!(value["diagnostics"][0]["code"], "file-project-missing");
}

#[test]
fn validate_human_output_is_readable() {
    let output = run(&["validate", &example("minimal-project")]);
    assert_eq!(output.status.code(), Some(0));
    let text = stdout(&output);
    assert!(text.contains("valid (0 errors, 0 warnings)"), "{text}");
}

#[test]
fn usage_without_arguments() {
    let output = run(&[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("Usage: composition-cli"));
}

#[test]
fn help_is_available() {
    let output = run(&["--help"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("inspect"));
    assert!(stdout(&output).contains("resolve"));
    assert!(stdout(&output).contains("validate"));
}
