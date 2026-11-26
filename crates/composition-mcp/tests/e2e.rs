use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use serde_json::{json, Value};

struct McpProcess {
    child: Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

impl McpProcess {
    fn send(&mut self, message: Value) -> Value {
        writeln!(self.stdin, "{message}").unwrap();
        self.stdin.flush().unwrap();
        let mut line = String::new();
        self.stdout.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    }

    fn call(&mut self, id: u64, method: &str, params: Value) -> Value {
        self.send(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        }))
    }
}

fn example() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples")
        .join("minimal-project")
}

fn copy_dir(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target_path = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target_path);
        } else {
            fs::copy(entry.path(), target_path).unwrap();
        }
    }
}

fn tool_payload(response: &Value) -> Value {
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("tool text");
    serde_json::from_str(text).expect("protocol JSON")
}

#[test]
fn agent_session_prepares_applies_and_validates() {
    let root = std::env::temp_dir().join(format!("composition-mcp-e2e-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    copy_dir(&example(), &root);

    let mut child = Command::new(env!("CARGO_BIN_EXE_composition-mcp"))
        .arg("--project")
        .arg(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn composition-mcp");
    let stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap());
    let mut mcp = McpProcess {
        child,
        stdin,
        stdout,
    };

    let init = mcp.call(1, "initialize", json!({ "protocolVersion": "2024-11-05" }));
    assert_eq!(init["result"]["serverInfo"]["name"], "composition-designer");

    let inspect = tool_payload(&mcp.call(
        2,
        "tools/call",
        json!({ "name": "composition_inspect", "arguments": {} }),
    ));
    let digest = inspect["result"]["sourceDigest"]
        .as_str()
        .expect("digest")
        .to_owned();
    assert_eq!(inspect["result"]["canEdit"], true);

    let resolve = tool_payload(&mcp.call(
        3,
        "tools/call",
        json!({ "name": "composition_resolve", "arguments": {} }),
    ));
    let seq = resolve["result"]["model"]["revision"]["seq"]
        .as_u64()
        .expect("revision seq");

    let transaction = json!({
        "request_id": "agent-e2e-1",
        "base_revision": { "seq": seq, "source_digest": digest },
        "commands": [{
            "type": "set_config",
            "instance": "app",
            "property": "title",
            "value": "Agent E2E"
        }]
    });

    let prepared = tool_payload(&mcp.call(
        4,
        "tools/call",
        json!({
            "name": "composition_prepare_transaction",
            "arguments": { "transaction": transaction.clone() }
        }),
    ));
    assert_eq!(prepared["result"]["accepted"], true);
    assert_eq!(prepared["result"]["diff"]["summary"]["modified"], 1);

    let applied = tool_payload(&mcp.call(
        5,
        "tools/call",
        json!({
            "name": "composition_apply_transaction",
            "arguments": { "transaction": transaction.clone() }
        }),
    ));
    assert_eq!(applied["result"]["accepted"], true);

    let validated = tool_payload(&mcp.call(
        6,
        "tools/call",
        json!({ "name": "composition_validate", "arguments": {} }),
    ));
    assert_eq!(validated["result"]["valid"], true);

    let source = mcp.call(
        7,
        "resources/read",
        json!({ "uri": "composition://project/source" }),
    );
    let text = source["result"]["contents"][0]["text"].as_str().unwrap();
    assert!(text.contains("Agent E2E"));
    let composition = fs::read_to_string(root.join("composition/main.toml")).unwrap();
    assert!(composition.contains("title = \"Agent E2E\""));

    drop(mcp.stdin);
    let _ = mcp.child.wait();
    let _ = fs::remove_dir_all(&root);
}
