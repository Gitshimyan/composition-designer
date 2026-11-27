use composition_change_gate::{
    changed_files_from_git, evaluate, load_manifests, load_task_cards, GateInput,
};
use std::path::PathBuf;

#[test]
fn repository_report_is_readable() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let base = std::env::var("GATE_BASE").unwrap_or_else(|_| "HEAD^".to_owned());
    let files = match changed_files_from_git(&root, &base, "HEAD") {
        Ok(files) => files,
        Err(error) if error.contains("ambiguous argument") => return,
        Err(error) => panic!("read git diff: {error}"),
    };
    if files.is_empty() {
        return;
    }
    let manifests = load_manifests(&root.join("changes")).expect("load manifests");
    let cards = load_task_cards(&root.join("working/tasks")).expect("load task cards");
    let findings = evaluate(&GateInput {
        changed_files: files,
        manifests,
        task_cards: cards,
    });
    // Report mode is intentionally non-blocking; this test only proves the real
    // repository diff and manifests can be parsed and evaluated.
    println!("gate findings: {}", findings.len());
}
