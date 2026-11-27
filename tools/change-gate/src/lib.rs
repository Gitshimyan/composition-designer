use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Other,
}

impl ChangeKind {
    pub fn from_git_status(status: &str) -> Self {
        match status.chars().next().unwrap_or('M') {
            'A' => Self::Added,
            'D' => Self::Deleted,
            _ => Self::Modified,
        }
    }

    pub fn is_added(self) -> bool {
        self == Self::Added
    }
}

#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub kind: ChangeKind,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Document {
    pub change: Manifest,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub id: String,
    #[serde(default)]
    pub intent: String,
    pub authorization: Authorization,
    pub targets: Vec<String>,
    #[serde(default)]
    pub operations: Vec<String>,
    #[serde(default)]
    pub scope: Scope,
    #[serde(default)]
    pub unresolved_items: Vec<String>,
    #[serde(default)]
    pub coverage_status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Authorization {
    Legacy(String),
    Set(AuthorizationSet),
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AuthorizationSet {
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub approved: Vec<String>,
    #[serde(default)]
    pub evidence: Option<String>,
}

impl Authorization {
    pub fn required(&self) -> BTreeSet<String> {
        match self {
            Self::Legacy(value) => BTreeSet::from([value.clone()]),
            Self::Set(set) => set.required.iter().cloned().collect(),
        }
    }

    pub fn approved(&self) -> BTreeSet<String> {
        match self {
            Self::Legacy(value) => BTreeSet::from([value.clone()]),
            Self::Set(set) => set.approved.iter().cloned().collect(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Scope {
    #[serde(default)]
    pub allowed: Vec<String>,
    #[serde(default)]
    pub forbidden: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TaskCard {
    pub coverage_status: Option<String>,
    pub open_items: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GateInput {
    pub changed_files: Vec<ChangedFile>,
    pub manifests: Vec<Manifest>,
    pub task_cards: BTreeMap<String, TaskCard>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub path: Option<String>,
    pub manifest: Option<String>,
}

impl Finding {
    fn error(code: &str, message: String, path: Option<String>, manifest: Option<String>) -> Self {
        Self {
            severity: Severity::Error,
            code: code.to_owned(),
            message,
            path,
            manifest,
        }
    }

    fn warning(
        code: &str,
        message: String,
        path: Option<String>,
        manifest: Option<String>,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            code: code.to_owned(),
            message,
            path,
            manifest,
        }
    }
}

pub fn evaluate(input: &GateInput) -> Vec<Finding> {
    let mut findings = Vec::new();

    for file in &input.changed_files {
        let candidates = covering_manifests(&input.manifests, &file.path);
        if candidates.is_empty() {
            findings.push(Finding::error(
                "undeclared-path",
                format!("{} is not covered by any changes/*.yaml target", file.path),
                Some(file.path.clone()),
                None,
            ));
            continue;
        }

        // A file may be covered by several manifests (for example a broad
        // historical target and a current task target). The gate accepts the
        // change when at least one covering manifest authorizes it; otherwise
        // it reports the most specific manifest's failures.
        let mut first_failure = None;
        let mut authorized = false;
        for manifest in candidates {
            let candidate_findings = file_findings(file, manifest);
            if candidate_findings.is_empty() {
                authorized = true;
                break;
            }
            if first_failure.is_none() {
                first_failure = Some(candidate_findings);
            }
        }
        if !authorized {
            findings.extend(first_failure.unwrap_or_default());
        }
    }

    for manifest in &input.manifests {
        if !is_relevant(manifest, &input.changed_files) {
            continue;
        }
        let Some(coverage_status) = &manifest.coverage_status else {
            continue;
        };
        let Some(prefix) = task_prefix(&manifest.id) else {
            findings.push(Finding::warning(
                "coverage-task-unresolved",
                format!("manifest {} has no task prefix", manifest.id),
                None,
                Some(manifest.id.clone()),
            ));
            continue;
        };
        let Some(card) = input.task_cards.get(prefix) else {
            findings.push(Finding::warning(
                "coverage-task-missing",
                format!(
                    "manifest {} points at task {} but no card was loaded",
                    manifest.id, prefix
                ),
                None,
                Some(manifest.id.clone()),
            ));
            continue;
        };
        if let Some(card_status) = &card.coverage_status {
            if card_status != coverage_status {
                findings.push(Finding::error(
                    "coverage-mismatch",
                    format!(
                        "manifest {} has coverage_status={} but task {} has {}",
                        manifest.id, coverage_status, prefix, card_status
                    ),
                    None,
                    Some(manifest.id.clone()),
                ));
            }
        } else {
            findings.push(Finding::warning(
                "coverage-card-missing",
                format!("task {} has no coverage_status header", prefix),
                None,
                Some(manifest.id.clone()),
            ));
        }

        let declared = manifest
            .unresolved_items
            .iter()
            .filter(|item| is_uw_id(item))
            .cloned()
            .collect::<BTreeSet<_>>();
        let actual = card
            .open_items
            .iter()
            .filter(|item| is_uw_id(item))
            .cloned()
            .collect::<BTreeSet<_>>();
        if declared != actual {
            findings.push(Finding::error(
                "unresolved-items-mismatch",
                format!(
                    "manifest {} unresolved_items=[{}] but task {} open_items=[{}]",
                    manifest.id,
                    declared.into_iter().collect::<Vec<_>>().join(", "),
                    prefix,
                    actual.into_iter().collect::<Vec<_>>().join(", ")
                ),
                None,
                Some(manifest.id.clone()),
            ));
        }
    }

    findings
}

pub fn changed_files_from_git(
    repo_root: &Path,
    base: &str,
    head: &str,
) -> Result<Vec<ChangedFile>, String> {
    if base.trim().is_empty() || base.chars().all(|character| character == '0') {
        return Ok(Vec::new());
    }
    if base == head {
        return Ok(Vec::new());
    }
    let output = Command::new("git")
        .current_dir(repo_root)
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

pub fn load_manifests(dir: &Path) -> Result<Vec<Manifest>, String> {
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

pub fn load_task_cards(dir: &Path) -> Result<BTreeMap<String, TaskCard>, String> {
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

pub fn parse_task_card(text: &str) -> TaskCard {
    let mut card = TaskCard::default();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("- ") else {
            continue;
        };
        if let Some(value) = rest.strip_prefix("coverage_status：") {
            card.coverage_status = Some(value.trim().to_owned());
        } else if let Some(value) = rest.strip_prefix("open_items：") {
            card.open_items = value
                .split([',', '，', '、'])
                .map(str::trim)
                .filter(|item| is_uw_id(item))
                .map(str::to_owned)
                .collect();
        }
    }
    card
}

pub fn task_prefix(manifest_id: &str) -> Option<&str> {
    manifest_id
        .strip_prefix("change.")
        .and_then(|rest| rest.split('-').next())
        .filter(|prefix| !prefix.is_empty())
}

fn is_relevant(manifest: &Manifest, files: &[ChangedFile]) -> bool {
    files.iter().any(|file| {
        manifest
            .targets
            .iter()
            .any(|target| target_match(target, &file.path).is_some())
    })
}

fn covering_manifests<'a>(manifests: &'a [Manifest], path: &str) -> Vec<&'a Manifest> {
    let mut covering = manifests
        .iter()
        .filter_map(|manifest| {
            manifest
                .targets
                .iter()
                .filter_map(|target| target_match(target, path))
                .max()
                .map(|score| (score, manifest))
        })
        .collect::<Vec<_>>();
    covering.sort_by(|(score_a, a), (score_b, b)| {
        score_b
            .cmp(score_a)
            .then_with(|| {
                b.coverage_status
                    .is_some()
                    .cmp(&a.coverage_status.is_some())
            })
            .then_with(|| b.id.cmp(&a.id))
    });
    covering.into_iter().map(|(_, manifest)| manifest).collect()
}

fn file_findings(file: &ChangedFile, manifest: &Manifest) -> Vec<Finding> {
    let mut findings = Vec::new();
    let required = required_authorizations(&file.path, file.kind);
    let required_set = manifest.authorization.required();
    let approved_set = manifest.authorization.approved();
    for permission in &required {
        if !required_set.contains(*permission) {
            findings.push(Finding::error(
                "authorization-required-missing",
                format!(
                    "{} requires {} but manifest {} does not declare it",
                    file.path, permission, manifest.id
                ),
                Some(file.path.clone()),
                Some(manifest.id.clone()),
            ));
        }
        if !approved_set.contains(*permission) {
            findings.push(Finding::error(
                "authorization-not-approved",
                format!(
                    "{} requires {} but manifest {} has no approval evidence",
                    file.path, permission, manifest.id
                ),
                Some(file.path.clone()),
                Some(manifest.id.clone()),
            ));
        }
    }

    let operation = required_operation(&file.path, file.kind);
    if !manifest.operations.iter().any(|value| value == operation) {
        findings.push(Finding::error(
            "operation-missing",
            format!(
                "{} requires operation {} but manifest {} lists [{}]",
                file.path,
                operation,
                manifest.id,
                manifest.operations.join(", ")
            ),
            Some(file.path.clone()),
            Some(manifest.id.clone()),
        ));
    }

    if forbidden_hit(&manifest.scope.forbidden, &file.path) {
        findings.push(Finding::error(
            "forbidden-scope",
            format!(
                "{} is explicitly forbidden by manifest {}",
                file.path, manifest.id
            ),
            Some(file.path.clone()),
            Some(manifest.id.clone()),
        ));
    }
    findings
}

fn required_authorizations(path: &str, kind: ChangeKind) -> BTreeSet<&'static str> {
    let mut required = BTreeSet::new();
    if is_sensitive(path) {
        required.insert("rearchitect");
    }
    if is_new_crate(path, kind) || is_command_change(path) || is_workflow_change(path) {
        required.insert("extend");
    }
    if is_compose(path) {
        required.insert("compose");
    }
    required
}

fn required_operation(path: &str, kind: ChangeKind) -> &'static str {
    if is_compose(path) {
        "compose"
    } else if is_sensitive(path)
        || is_new_crate(path, kind)
        || is_command_change(path)
        || is_workflow_change(path)
    {
        "extend"
    } else {
        "modify"
    }
}

fn is_sensitive(path: &str) -> bool {
    path == "AGENTS.md"
        || path == "Cargo.toml"
        || path == "Cargo.lock"
        || path == "changes/000-template.yaml"
        || path.starts_with("rust-toolchain")
        || path.starts_with(".cargo/")
        || path.starts_with("docs/design/")
        || path.starts_with("docs/spec/")
}

fn is_new_crate(path: &str, kind: ChangeKind) -> bool {
    kind.is_added()
        && path.ends_with("/Cargo.toml")
        && (path.starts_with("crates/") || path.starts_with("tools/"))
}

fn is_command_change(path: &str) -> bool {
    path == "crates/composition-editor/src/command.rs"
        || path.starts_with("crates/composition-editor/src/command/")
        || path == "docs/spec/command-schema-v1.md"
}

fn is_workflow_change(path: &str) -> bool {
    path.starts_with(".github/workflows/")
}

fn is_compose(path: &str) -> bool {
    path.starts_with("examples/") || path.starts_with("working/pilots/")
}

fn is_uw_id(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("UW-") else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|character| character.is_ascii_digit())
}

fn forbidden_hit(forbidden: &[String], path: &str) -> bool {
    forbidden
        .iter()
        .filter(|token| looks_like_path(token))
        .any(|token| target_match(token, path).is_some())
}

fn looks_like_path(token: &str) -> bool {
    token.contains('/')
        || token.contains('*')
        || token.ends_with(".md")
        || token.ends_with(".toml")
        || token.ends_with(".rs")
        || token.ends_with(".yaml")
        || token.ends_with(".yml")
        || token == "AGENTS.md"
}

fn target_match(target: &str, path: &str) -> Option<usize> {
    let target = target
        .trim()
        .trim_start_matches("./")
        .trim_end_matches('/')
        .replace('\\', "/");
    if target.is_empty() || target == "." {
        return Some(0);
    }
    if target.contains('*') {
        return wildcard_match(&target, path).then_some(target.len());
    }
    if path == target || path.starts_with(&(target.clone() + "/")) {
        Some(target.len())
    } else {
        None
    }
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let mut p = 0;
    let mut v = 0;
    let mut star = None;
    let mut star_value = 0;
    while v < value.len() {
        if p < pattern.len() && (pattern[p] == value[v] || pattern[p] == b'?') {
            p += 1;
            v += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            star_value = v;
            p += 1;
        } else if let Some(star_position) = star {
            p = star_position + 1;
            star_value += 1;
            v = star_value;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(
        id: &str,
        required: &[&str],
        approved: &[&str],
        targets: &[&str],
        operations: &[&str],
    ) -> Manifest {
        Manifest {
            id: id.to_owned(),
            intent: String::new(),
            authorization: Authorization::Set(AuthorizationSet {
                required: required.iter().map(|value| (*value).to_owned()).collect(),
                approved: approved.iter().map(|value| (*value).to_owned()).collect(),
                evidence: None,
            }),
            targets: targets.iter().map(|value| (*value).to_owned()).collect(),
            operations: operations.iter().map(|value| (*value).to_owned()).collect(),
            scope: Scope::default(),
            unresolved_items: Vec::new(),
            coverage_status: None,
        }
    }

    fn file(path: &str, kind: ChangeKind) -> ChangedFile {
        ChangedFile {
            path: path.to_owned(),
            kind,
        }
    }

    fn input(files: Vec<ChangedFile>, manifests: Vec<Manifest>) -> GateInput {
        GateInput {
            changed_files: files,
            manifests,
            task_cards: BTreeMap::new(),
        }
    }

    fn codes(findings: &[Finding]) -> Vec<&str> {
        findings
            .iter()
            .map(|finding| finding.code.as_str())
            .collect()
    }

    #[test]
    fn passes_when_file_is_covered_and_authorized() {
        let findings = evaluate(&input(
            vec![file(
                "crates/composition-core/src/lib.rs",
                ChangeKind::Modified,
            )],
            vec![manifest(
                "change.026-change-review",
                &["modify"],
                &["modify"],
                &["crates/composition-core"],
                &["modify"],
            )],
        ));
        assert!(findings.is_empty(), "{:?}", findings);
    }

    #[test]
    fn accepts_any_authorized_covering_manifest() {
        let strict = manifest(
            "change.G04-machine-gate",
            &["extend", "rearchitect"],
            &["extend", "rearchitect"],
            &["README.md"],
            &["extend"],
        );
        let docs = manifest(
            "change.030-docs",
            &["modify"],
            &["modify"],
            &["README.md"],
            &["modify"],
        );
        let findings = evaluate(&input(
            vec![file("README.md", ChangeKind::Modified)],
            vec![strict, docs],
        ));
        assert!(findings.is_empty(), "{:?}", findings);
    }

    #[test]
    fn reports_undeclared_path() {
        let findings = evaluate(&input(
            vec![file("unowned/file.txt", ChangeKind::Modified)],
            vec![manifest(
                "change.026-change-review",
                &["modify"],
                &["modify"],
                &["crates/composition-core"],
                &["modify"],
            )],
        ));
        assert_eq!(codes(&findings), vec!["undeclared-path"]);
    }

    #[test]
    fn requires_extend_for_new_crate() {
        let findings = evaluate(&input(
            vec![file("crates/new-crate/Cargo.toml", ChangeKind::Added)],
            vec![manifest(
                "change.026-change-review",
                &["modify"],
                &["modify"],
                &["crates"],
                &["modify"],
            )],
        ));
        assert!(codes(&findings).contains(&"authorization-required-missing"));
        assert!(codes(&findings).contains(&"operation-missing"));
    }

    #[test]
    fn requires_compose_and_rearchitect_for_sensitive_paths() {
        let findings = evaluate(&input(
            vec![
                file(
                    "working/pilots/demo/composition/main.toml",
                    ChangeKind::Modified,
                ),
                file("docs/spec/composition-schema-v1.md", ChangeKind::Modified),
            ],
            vec![manifest(
                "change.026-change-review",
                &["modify"],
                &["modify"],
                &["working", "docs/spec"],
                &["modify"],
            )],
        ));
        assert!(codes(&findings).contains(&"authorization-required-missing"));
        assert!(codes(&findings).contains(&"operation-missing"));
    }

    #[test]
    fn reports_forbidden_scope() {
        let mut guarded = manifest(
            "change.026-change-review",
            &["modify"],
            &["modify"],
            &["crates"],
            &["modify"],
        );
        guarded.scope.forbidden = vec!["crates/secret".to_owned()];
        let findings = evaluate(&input(
            vec![file("crates/secret/src/lib.rs", ChangeKind::Modified)],
            vec![guarded],
        ));
        assert!(codes(&findings).contains(&"forbidden-scope"));
    }

    #[test]
    fn checks_coverage_and_unresolved_items() {
        let mut covered = manifest(
            "change.027-m6-closeout",
            &["modify"],
            &["modify"],
            &["working"],
            &["modify"],
        );
        covered.coverage_status = Some("complete".to_owned());
        let mut cards = BTreeMap::new();
        cards.insert(
            "027".to_owned(),
            TaskCard {
                coverage_status: Some("partial".to_owned()),
                open_items: vec!["UW-029".to_owned()],
            },
        );
        let findings = evaluate(&GateInput {
            changed_files: vec![file("working/plan.md", ChangeKind::Modified)],
            manifests: vec![covered],
            task_cards: cards,
        });
        assert!(codes(&findings).contains(&"coverage-mismatch"));
        assert!(codes(&findings).contains(&"unresolved-items-mismatch"));
    }

    #[test]
    fn parses_task_card_headers() {
        let card = parse_task_card(
            "- coverage_status：complete_with_deferred_items\n- open_items：UW-025、UW-029\n",
        );
        assert_eq!(
            card.coverage_status.as_deref(),
            Some("complete_with_deferred_items")
        );
        assert_eq!(card.open_items, vec!["UW-025", "UW-029"]);
    }
}
