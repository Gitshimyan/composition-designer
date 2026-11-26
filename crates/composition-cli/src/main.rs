use std::env;
use std::path::{Path, PathBuf};

use composition_core::{validate_project, ValidationResult};
use composition_domain::{
    CompositionModel, Diagnostic, ModuleDefinition, Project, Severity, SourceDigest,
};
use serde::Serialize;

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
    eprintln!("Usage: composition-cli <inspect|resolve|validate> <project-dir> [--json]");
}

fn print_help() {
    println!("composition-cli {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Usage:");
    println!("  composition-cli inspect <project-dir>");
    println!("  composition-cli resolve <project-dir>");
    println!("  composition-cli validate <project-dir> [--json]");
    println!();
    println!("Commands:");
    println!("  inspect   project, module contracts, diagnostics and source digest");
    println!("  resolve   normalized CompositionModel, diagnostics and source digest");
    println!("  validate  validation summary and diagnostics (human by default)");
    println!();
    println!("Exit codes: 0 valid, 1 errors, 2 usage/internal error.");
}
