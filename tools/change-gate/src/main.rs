use composition_change_gate::{
    changed_files_from_git, evaluate, load_manifests, load_task_cards, Finding, GateInput, Severity,
};
use std::path::PathBuf;

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
    let mut changed = changed_files_from_git(&PathBuf::from("."), &args.base, &args.head)?;
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
