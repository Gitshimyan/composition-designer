use std::fs;
use std::path::{Path, PathBuf};

use composition_domain::Revision;
use serde::{Deserialize, Serialize};

use crate::command::Transaction;
use crate::diff::ModelDiff;

const STATE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SessionState {
    pub version: u32,
    pub profile: Option<String>,
    pub base_revision: Revision,
    pub base_source_text: String,
    pub commits: Vec<CommitState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CommitState {
    pub request_id: String,
    pub transaction: Transaction,
    pub revision: Revision,
    pub source_text: String,
    pub diff: ModelDiff,
}

pub(crate) fn state_path(root: &Path, profile: Option<&str>) -> PathBuf {
    let key = profile.unwrap_or("default");
    root.join(".designer")
        .join("state")
        .join(format!("{key}.json"))
}

pub(crate) fn load(root: &Path, profile: Option<&str>) -> Option<SessionState> {
    let text = fs::read_to_string(state_path(root, profile)).ok()?;
    let state: SessionState = serde_json::from_str(&text).ok()?;
    if state.version != STATE_VERSION || state.profile.as_deref() != profile {
        return None;
    }
    Some(state)
}

pub(crate) fn save(root: &Path, profile: Option<&str>, state: &SessionState) -> Result<(), String> {
    let path = state_path(root, profile);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create state directory: {error}"))?;
    }
    let suffix = std::process::id();
    let temp = path.with_extension(format!("json.tmp-{suffix}"));
    let text = serde_json::to_string(state).map_err(|error| error.to_string())?;
    fs::write(&temp, text).map_err(|error| format!("cannot write state: {error}"))?;
    fs::rename(&temp, &path).map_err(|error| {
        let _ = fs::remove_file(&temp);
        format!("cannot replace state: {error}")
    })
}
