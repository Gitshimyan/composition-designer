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

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
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

fn spawn(root: &Path) -> McpProcess {
    let mut child = Command::new(env!("CARGO_BIN_EXE_composition-mcp"))
        .arg("--project")
        .arg(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn composition-mcp");
    let stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap());
    McpProcess {
        child,
        stdin,
        stdout,
    }
}

fn tool_payload(response: &Value) -> Value {
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("tool text");
    serde_json::from_str(text).expect("protocol JSON")
}

fn call_tool(mcp: &mut McpProcess, id: u64, name: &str, arguments: Value) -> Value {
    tool_payload(&mcp.call(
        id,
        "tools/call",
        json!({ "name": name, "arguments": arguments }),
    ))
}

fn finish(mcp: McpProcess) {
    drop(mcp.stdin);
    let mut child = mcp.child;
    let _ = child.wait();
}

#[test]
fn skill_eval_minimal_project_workflow() {
    let root = std::env::temp_dir().join(format!("composition-skill-eval-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    copy_dir(&workspace_root().join("examples/minimal-project"), &root);

    let mut mcp = spawn(&root);
    let initialized = mcp.call(1, "initialize", json!({ "protocolVersion": "2024-11-05" }));
    assert_eq!(
        initialized["result"]["serverInfo"]["name"],
        "composition-designer"
    );

    let tools = mcp.call(2, "tools/list", json!({}));
    let names: Vec<&str> = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert_eq!(names.len(), 9);
    for required in [
        "composition_context",
        "composition_impact",
        "composition_batch",
        "composition_prepare_transaction",
        "composition_apply_transaction",
    ] {
        assert!(names.contains(&required), "missing tool {required}");
    }

    let context = call_tool(&mut mcp, 3, "composition_context", json!({}));
    assert_eq!(context["ok"], true);
    assert_eq!(context["result"]["project"]["id"], "example.minimal");
    assert_eq!(context["result"]["counts"]["instances"], 2);

    let impact = call_tool(
        &mut mcp,
        4,
        "composition_impact",
        json!({ "instance": "app" }),
    );
    assert_eq!(impact["result"]["kind"], "instance");
    assert_eq!(impact["result"]["instance"]["instance"]["id"], "app");

    let batch = call_tool(
        &mut mcp,
        5,
        "composition_batch",
        json!({
            "requests": [
                { "id": "inspect", "op": "inspect" },
                { "id": "context", "op": "context" },
                { "id": "impact", "op": "impact", "params": { "instance": "hello" } }
            ]
        }),
    );
    assert_eq!(batch["result"]["count"], 3);
    assert_eq!(batch["result"]["results"][2]["result"]["kind"], "instance");

    let resolve = call_tool(&mut mcp, 6, "composition_resolve", json!({}));
    let digest = resolve["result"]["model"]["revision"]["source_digest"]
        .as_str()
        .expect("digest")
        .to_owned();
    let seq = resolve["result"]["model"]["revision"]["seq"]
        .as_u64()
        .expect("seq");
    let transaction = json!({
        "request_id": "skill-eval-1",
        "base_revision": { "seq": seq, "source_digest": digest },
        "commands": [{
            "type": "set_config",
            "instance": "app",
            "property": "title",
            "value": "Skill Eval"
        }]
    });
    let prepared = call_tool(
        &mut mcp,
        7,
        "composition_prepare_transaction",
        json!({ "transaction": transaction.clone() }),
    );
    assert_eq!(prepared["result"]["accepted"], true);
    let applied = call_tool(
        &mut mcp,
        8,
        "composition_apply_transaction",
        json!({ "transaction": transaction }),
    );
    assert_eq!(applied["result"]["accepted"], true);

    let validated = call_tool(&mut mcp, 9, "composition_validate", json!({}));
    assert_eq!(validated["result"]["valid"], true);
    let source = call_tool(&mut mcp, 10, "composition_source", json!({}));
    assert!(source["result"]["text"]
        .as_str()
        .unwrap()
        .contains("Skill Eval"));

    finish(mcp);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn skill_eval_v2_pilot_context_and_impact() {
    let root =
        std::env::temp_dir().join(format!("composition-skill-eval-v2-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    copy_dir(
        &workspace_root().join("working/pilots/glide-task-terminal-v2"),
        &root,
    );

    let mut mcp = spawn(&root);
    mcp.call(1, "initialize", json!({ "protocolVersion": "2024-11-05" }));

    let context = call_tool(&mut mcp, 2, "composition_context", json!({}));
    assert_eq!(context["result"]["counts"]["instances"], 7);
    assert_eq!(context["result"]["counts"]["services"], 3);

    let module = call_tool(
        &mut mcp,
        3,
        "composition_impact",
        json!({
            "module": {
                "id": "glide.platform.conpty-session",
                "version": "1.0.0"
            }
        }),
    );
    assert_eq!(
        module["result"]["module"]["interfaceImplementers"][0]["interface"]["id"],
        "glide.terminal.session"
    );

    let instance = call_tool(
        &mut mcp,
        4,
        "composition_impact",
        json!({ "instance": "task-runner" }),
    );
    assert_eq!(
        instance["result"]["instance"]["instance"]["scope"],
        "service"
    );
    assert_eq!(
        instance["result"]["instance"]["requires"][0]["requirement"],
        "terminal-session"
    );

    finish(mcp);
    let _ = fs::remove_dir_all(&root);
}
