use composition_change_gate::{
    evaluate, parse_task_card, ChangeKind, ChangedFile, Document, Finding, GateInput, Manifest,
    Severity, TaskCard,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Report,
    Block,
}

struct Args {
    base: String,
    head: String,
    mode: Mode,
    manifest_dir: PathBuf,
    task_dir: PathBuf,
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) => {
            eprintln!("change-gate: {error}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    let mut changed = git_changed_files(&args.base, &args.head)?;
    changed.sort_by(|left, right| left.path.cmp(&right.path));
    changed.dedup_by(|left, right| left.path == right.path);

    let manifests = load_manifests(&args.manifest_dir)?;
    let task_cards = load_task_cards(&args.task_dir)?;
    let findings = evaluate(&GateInput {
        changed_files: changed.clone(),
        manifests: manifests.clone(),
        task_cards,
    });

    println!(
        "Change gate — {} mode",
        if args.mode == Mode::Report {
            "report"
        } else {
            "block"
        }
    );
    println!("base {} → head {}", args.base, args.head);
    println!(
        "changed files: {} · manifests: {}",
        changed.len(),
        manifests.len()
    );
    for finding in &findings {
        print_finding(finding);
    }
    let errors = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count();
    let warnings = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Warning)
        .count();
    println!("summary: {errors} error(s), {warnings} warning(s)");

    if args.mode == Mode::Block && errors > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn parse_args() -> Result<Args, String> {
    let mut args = std::env::args().skip(1);
    let mut base = std::env::var("GITHUB_BASE_SHA").unwrap_or_else(|_| "HEAD^".to_owned());
    let mut head = std::env::var("GITHUB_HEAD_SHA").unwrap_or_else(|_| "HEAD".to_owned());
    let mut mode = Mode::Report;
    let mut manifest_dir = PathBuf::from("changes");
    let mut task_dir = PathBuf::from("working/tasks");

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--base" => base = next_value(&mut args, "--base")?,
            "--head" => head = next_value(&mut args, "--head")?,
            "--mode" => {
                mode = match next_value(&mut args, "--mode")?.as_str() {
                    "report" => Mode::Report,
                    "block" => Mode::Block,
                    other => return Err(format!("unknown mode {other}; use report or block")),
                }
            }
            "--manifest-dir" => {
                manifest_dir = PathBuf::from(next_value(&mut args, "--manifest-dir")?)
            }
            "--task-dir" => task_dir = PathBuf::from(next_value(&mut args, "--task-dir")?),
            "--help" | "-h" => {
                println!(
                    "Usage: composition-change-gate [--base <git-ref>] [--head <git-ref>] [--mode report|block] [--manifest-dir changes] [--task-dir working/tasks]"
                );
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }

    Ok(Args {
        base,
        head,
        mode,
        manifest_dir,
        task_dir,
    })
}

fn next_value(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{name} requires a value"))
}

fn git_changed_files(base: &str, head: &str) -> Result<Vec<ChangedFile>, String> {
    if base.trim().is_empty() || base.chars().all(|character| character == '0') {
        return Ok(Vec::new());
    }
    if base == head {
        return Ok(Vec::new());
    }
    let output = Command::new("git")
        .args(["diff", "--name-status", "--no-renames", base, head])
        .output()
        .map_err(|error| format!("cannot run git diff: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git diff {base} {head} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let mut files = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut parts = line.split('\t');
        let status = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("");
        if path.is_empty() {
            continue;
        }
        files.push(ChangedFile {
            path: path.replace('\\', "/"),
            kind: ChangeKind::from_git_status(status),
        });
    }
    Ok(files)
}

fn load_manifests(dir: &Path) -> Result<Vec<Manifest>, String> {
    let mut manifests = Vec::new();
    let entries =
        fs::read_dir(dir).map_err(|error| format!("cannot read {}: {error}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read manifest directory: {error}"))?;
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if !name.ends_with(".yaml") && !name.ends_with(".yml") {
            continue;
        }
        if name == "000-template.yaml" {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let document: Document = serde_yaml::from_str(&text)
            .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;
        manifests.push(document.change);
    }
    Ok(manifests)
}

fn load_task_cards(dir: &Path) -> Result<BTreeMap<String, TaskCard>, String> {
    let mut cards = BTreeMap::new();
    let entries =
        fs::read_dir(dir).map_err(|error| format!("cannot read {}: {error}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read task directory: {error}"))?;
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if !name.ends_with(".md") || name == "000-template.md" {
            continue;
        }
        let Some(prefix) = name.split('-').next() else {
            continue;
        };
        if prefix.is_empty() {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        cards.insert(prefix.to_owned(), parse_task_card(&text));
    }
    Ok(cards)
}

fn print_finding(finding: &Finding) {
    let severity = match finding.severity {
        Severity::Error => "ERROR",
        Severity::Warning => "WARN ",
    };
    let path = finding
        .path
        .as_deref()
        .map(|path| format!(" path={path}"))
        .unwrap_or_default();
    let manifest = finding
        .manifest
        .as_deref()
        .map(|manifest| format!(" manifest={manifest}"))
        .unwrap_or_default();
    println!(
        "{severity} [{}]{}{} {}",
        finding.code, path, manifest, finding.message
    );
}
