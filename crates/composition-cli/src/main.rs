use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use composition_core::{validate_project, ValidationResult};
use composition_domain::{
    CompositionModel, Diagnostic, ModuleDefinition, Project, Severity, SourceDigest,
};
use composition_protocol::{error_response, ProtocolSession, Request, Response};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct InspectOutput<'a> {
    project: Option<&'a Project>,
    modules: &'a [ModuleDefinition],
    diagnostics: &'a [Diagnostic],
    source_digest: Option<&'a SourceDigest>,
}

#[derive(Serialize)]
struct ResolveOutput<'a> {
    model: Option<&'a CompositionModel>,
    diagnostics: &'a [Diagnostic],
    source_digest: Option<&'a SourceDigest>,
}

#[derive(Serialize)]
struct ValidateOutput<'a> {
    valid: bool,
    error_count: usize,
    warning_count: usize,
    source_digest: Option<&'a SourceDigest>,
    diagnostics: &'a [Diagnostic],
}

fn main() {
    std::process::exit(run(env::args().skip(1)));
}

fn run<I>(args: I) -> i32
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        usage();
        return 2;
    };
    match command.as_str() {
        "-h" | "--help" => {
            print_help();
            0
        }
        "-V" | "--version" => {
            println!("composition-cli {}", env!("CARGO_PKG_VERSION"));
            0
        }
        "jsonl" => {
            let Some(path) = args.next() else {
                usage();
                return 2;
            };
            if args.next().is_some() {
                usage();
                return 2;
            }
            run_jsonl(Path::new(&path))
        }
        "query" => {
            let Some(path) = args.next() else {
                usage();
                return 2;
            };
            let Some(op) = args.next() else {
                usage();
                return 2;
            };
            if args.next().is_some() {
                usage();
                return 2;
            }
            run_query(Path::new(&path), &op)
        }
        "apply" => {
            let Some(path) = args.next() else {
                usage();
                return 2;
            };
            let Some(transaction_path) = args.next() else {
                usage();
                return 2;
            };
            let mut dry_run = false;
            for arg in args {
                match arg.as_str() {
                    "--dry-run" => dry_run = true,
                    flag => {
                        eprintln!("unknown option {flag}");
                        usage();
                        return 2;
                    }
                }
            }
            run_apply(Path::new(&path), Path::new(&transaction_path), dry_run)
        }
        "inspect" | "resolve" | "validate" => {
            let mut path = None;
            let mut json = false;
            for arg in args {
                match arg.as_str() {
                    "--json" => json = true,
                    flag if flag.starts_with('-') => {
                        eprintln!("unknown option {flag}");
                        usage();
                        return 2;
                    }
                    value => {
                        if path.is_some() {
                            eprintln!("unexpected argument {value}");
                            usage();
                            return 2;
                        }
                        path = Some(PathBuf::from(value));
                    }
                }
            }
            let Some(path) = path else {
                usage();
                return 2;
            };
            let result = validate_project(&path);
            let valid = result.is_valid();
            let written = match command.as_str() {
                "inspect" => write_json(&inspect_output(&result)),
                "resolve" => write_json(&resolve_output(&result)),
                "validate" if json => write_json(&validate_output(&result)),
                "validate" => {
                    print_human(&path, &result);
                    Ok(())
                }
                _ => unreachable!(),
            };
            if let Err(error) = written {
                eprintln!("failed to serialize output: {error}");
                return 2;
            }
            if valid {
                0
            } else {
                1
            }
        }
        other => {
            eprintln!("unknown command {other}");
            usage();
            2
        }
    }
}

fn run_jsonl(root: &Path) -> i32 {
    let mut session = ProtocolSession::new(root);
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("failed to read stdin: {error}");
                return 2;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Request>(&line) {
            Ok(request) => session.dispatch(request),
            Err(error) => error_response(
                "",
                "protocol-invalid-request",
                format!("invalid JSON request: {error}"),
            ),
        };
        if let Err(error) = write_response_line(&mut stdout, &response) {
            eprintln!("failed to write response: {error}");
            return 2;
        }
    }
    0
}

fn run_query(root: &Path, op: &str) -> i32 {
    let mut session = ProtocolSession::new(root);
    let response = session.dispatch(Request {
        id: "cli-query".to_owned(),
        op: op.to_owned(),
        params: json!({}),
    });
    write_response_pretty(&response)
}

fn run_apply(root: &Path, transaction_path: &Path, dry_run: bool) -> i32 {
    let text = match fs::read_to_string(transaction_path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("cannot read {}: {error}", transaction_path.display());
            return 2;
        }
    };
    let transaction = match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(transaction) => transaction,
        Err(error) => {
            eprintln!("invalid transaction JSON: {error}");
            return 2;
        }
    };
    let mut session = ProtocolSession::new(root);
    let response = session.dispatch(Request {
        id: "cli-apply".to_owned(),
        op: if dry_run {
            "prepare_transaction".to_owned()
        } else {
            "apply_transaction".to_owned()
        },
        params: json!({ "transaction": transaction }),
    });
    write_response_pretty(&response)
}

fn write_response_pretty(response: &Response) -> i32 {
    match serde_json::to_string_pretty(response) {
        Ok(text) => {
            println!("{text}");
            if response.ok {
                0
            } else {
                1
            }
        }
        Err(error) => {
            eprintln!("failed to serialize response: {error}");
            2
        }
    }
}

fn write_response_line(
    writer: &mut impl Write,
    response: &Response,
) -> Result<(), Box<dyn std::error::Error>> {
    let text = serde_json::to_string(response)?;
    writeln!(writer, "{text}")?;
    writer.flush()?;
    Ok(())
}

fn inspect_output(result: &ValidationResult) -> InspectOutput<'_> {
    let (project, modules) = match &result.model {
        Some(model) => (Some(&model.project), model.modules.as_slice()),
        None => (None, &[] as &[ModuleDefinition]),
    };
    InspectOutput {
        project,
        modules,
        diagnostics: &result.diagnostics,
        source_digest: result.source_digest.as_ref(),
    }
}

fn resolve_output(result: &ValidationResult) -> ResolveOutput<'_> {
    ResolveOutput {
        model: result.model.as_ref(),
        diagnostics: &result.diagnostics,
        source_digest: result.source_digest.as_ref(),
    }
}

fn validate_output(result: &ValidationResult) -> ValidateOutput<'_> {
    ValidateOutput {
        valid: result.is_valid(),
        error_count: result.error_count(),
        warning_count: result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Warning)
            .count(),
        source_digest: result.source_digest.as_ref(),
        diagnostics: &result.diagnostics,
    }
}

fn write_json<T>(value: &T) -> Result<(), serde_json::Error>
where
    T: Serialize,
{
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn print_human(path: &Path, result: &ValidationResult) {
    let errors = result.error_count();
    let warnings = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Warning)
        .count();
    println!(
        "{}: {} ({} errors, {} warnings)",
        path.display(),
        if result.is_valid() {
            "valid"
        } else {
            "invalid"
        },
        errors,
        warnings
    );
    for diagnostic in &result.diagnostics {
        let location = match (&diagnostic.file, diagnostic.range) {
            (Some(file), Some(range)) => {
                format!(" ({file}:{}:{})", range.start.line, range.start.column)
            }
            (Some(file), None) => format!(" ({file})"),
            _ => String::new(),
        };
        println!(
            "{} [{}] {}{}",
            severity_name(diagnostic.severity),
            diagnostic.code,
            diagnostic.message,
            location
        );
    }
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

fn usage() {
    eprintln!("Usage: composition-cli <inspect|resolve|validate|query|apply|jsonl> ...");
}

fn print_help() {
    println!("composition-cli {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Usage:");
    println!("  composition-cli inspect <project-dir>");
    println!("  composition-cli resolve <project-dir>");
    println!("  composition-cli validate <project-dir> [--json]");
    println!("  composition-cli query <project-dir> <op>");
    println!("  composition-cli apply <project-dir> <transaction.json> [--dry-run]");
    println!("  composition-cli jsonl <project-dir>");
    println!();
    println!("Commands:");
    println!("  inspect   project, module contracts, diagnostics and source digest");
    println!("  resolve   normalized CompositionModel, diagnostics and source digest");
    println!("  validate  validation summary and diagnostics (human by default)");
    println!("  query     protocol query: inspect | resolve | validate | source");
    println!("  apply     protocol transaction from a JSON file (use --dry-run to prepare)");
    println!("  jsonl     line-delimited JSON requests on stdin / responses on stdout");
    println!();
    println!("Exit codes: 0 valid, 1 errors, 2 usage/internal error.");
}
