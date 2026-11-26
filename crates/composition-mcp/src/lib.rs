use std::path::{Path, PathBuf};

use composition_protocol::{ProtocolSession, Request};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "composition-designer";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

pub struct McpServer {
    session: ProtocolSession,
}

impl McpServer {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            session: ProtocolSession::new(root),
        }
    }

    /// Process one newline-delimited JSON-RPC message.
    /// Notifications produce no output.
    pub fn handle_line(&mut self, line: &str) -> Option<String> {
        if line.trim().is_empty() {
            return None;
        }
        let request: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(request) => request,
            Err(error) => {
                return Some(serialize_response(error_response(
                    Value::Null,
                    -32700,
                    format!("parse error: {error}"),
                )))
            }
        };
        if request.jsonrpc != "2.0" {
            return Some(serialize_response(error_response(
                request.id.unwrap_or(Value::Null),
                -32600,
                "invalid JSON-RPC version",
            )));
        }
        let Some(id) = request.id else {
            // Notifications (including notifications/initialized) do not reply.
            return None;
        };
        let result = self.dispatch(&request.method, request.params);
        Some(serialize_response(match result {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: Some(result),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: None,
                error: Some(error),
            },
        }))
    }

    fn dispatch(&mut self, method: &str, params: Value) -> Result<Value, JsonRpcError> {
        match method {
            "initialize" => Ok(self.initialize(params)),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({ "tools": tools() })),
            "tools/call" => self.call_tool(params),
            "resources/list" => Ok(json!({ "resources": resources() })),
            "resources/read" => self.read_resource(params),
            other => Err(JsonRpcError {
                code: -32601,
                message: format!("method not found: {other}"),
                data: None,
            }),
        }
    }

    fn initialize(&self, params: Value) -> Value {
        let requested = params
            .get("protocolVersion")
            .and_then(Value::as_str)
            .unwrap_or(PROTOCOL_VERSION);
        json!({
            "protocolVersion": requested,
            "capabilities": {
                "tools": { "listChanged": false },
                "resources": { "subscribe": false, "listChanged": false }
            },
            "serverInfo": {
                "name": SERVER_NAME,
                "version": SERVER_VERSION
            }
        })
    }

    fn call_tool(&mut self, params: Value) -> Result<Value, JsonRpcError> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid_params("tools/call requires a tool name"))?;
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let (op, protocol_params) = match name {
            "composition_inspect" => ("inspect", json!({})),
            "composition_resolve" => ("resolve", json!({})),
            "composition_validate" => ("validate", json!({})),
            "composition_source" => ("source", json!({})),
            "composition_prepare_transaction" => (
                "prepare_transaction",
                json!({ "transaction": arguments.get("transaction").cloned().unwrap_or(Value::Null) }),
            ),
            "composition_apply_transaction" => (
                "apply_transaction",
                json!({ "transaction": arguments.get("transaction").cloned().unwrap_or(Value::Null) }),
            ),
            other => {
                return Err(JsonRpcError {
                    code: -32602,
                    message: format!("unknown tool: {other}"),
                    data: None,
                })
            }
        };
        let response = self.session.dispatch(Request {
            id: format!("mcp-{name}"),
            op: op.to_owned(),
            params: protocol_params,
        });
        let text = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_owned());
        let mut result = json!({
            "content": [{ "type": "text", "text": text }],
            "isError": !response.ok
        });
        if let Some(Value::Object(structured)) = response.result {
            result["structuredContent"] = Value::Object(structured);
        }
        Ok(result)
    }

    fn read_resource(&mut self, params: Value) -> Result<Value, JsonRpcError> {
        let uri = params
            .get("uri")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid_params("resources/read requires uri"))?;
        let op = match uri {
            "composition://project/inspect" => "inspect",
            "composition://project/model" => "resolve",
            "composition://project/diagnostics" => "validate",
            "composition://project/source" => "source",
            other => {
                return Err(JsonRpcError {
                    code: -32602,
                    message: format!("unknown resource: {other}"),
                    data: None,
                })
            }
        };
        let response = self.session.dispatch(Request {
            id: format!("resource-{op}"),
            op: op.to_owned(),
            params: json!({}),
        });
        let text = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_owned());
        Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": text
            }]
        }))
    }
}

fn tools() -> Value {
    let no_args = json!({
        "type": "object",
        "properties": {},
        "additionalProperties": false
    });
    let transaction = json!({
        "type": "object",
        "properties": {
            "transaction": {
                "type": "object",
                "description": "Transaction envelope from command-schema-v1"
            }
        },
        "required": ["transaction"],
        "additionalProperties": false
    });
    json!([
        {
            "name": "composition_inspect",
            "description": "Project identity, modules, counts, source digest and editability.",
            "inputSchema": no_args
        },
        {
            "name": "composition_resolve",
            "description": "Normalized CompositionModel.",
            "inputSchema": no_args
        },
        {
            "name": "composition_validate",
            "description": "Validation summary with L1-L7 diagnostics.",
            "inputSchema": no_args
        },
        {
            "name": "composition_source",
            "description": "Read-only composition TOML source for the current session.",
            "inputSchema": no_args
        },
        {
            "name": "composition_prepare_transaction",
            "description": "Validate a transaction and return its Diff without writing.",
            "inputSchema": transaction
        },
        {
            "name": "composition_apply_transaction",
            "description": "Apply a transaction atomically and return its Diff.",
            "inputSchema": transaction
        }
    ])
}

fn resources() -> Value {
    json!([
        {
            "uri": "composition://project/inspect",
            "name": "Project inspection",
            "description": "Project identity, modules and editability",
            "mimeType": "application/json"
        },
        {
            "uri": "composition://project/model",
            "name": "Composition model",
            "description": "Normalized CompositionModel",
            "mimeType": "application/json"
        },
        {
            "uri": "composition://project/diagnostics",
            "name": "Validation diagnostics",
            "description": "L1-L7 diagnostics and validity summary",
            "mimeType": "application/json"
        },
        {
            "uri": "composition://project/source",
            "name": "Composition TOML",
            "description": "Read-only composition entry source",
            "mimeType": "application/json"
        }
    ])
}

fn invalid_params(message: impl Into<String>) -> JsonRpcError {
    JsonRpcError {
        code: -32602,
        message: message.into(),
        data: None,
    }
}

fn error_response(id: Value, code: i64, message: impl Into<String>) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.into(),
            data: None,
        }),
    }
}

fn serialize_response(response: JsonRpcResponse) -> String {
    serde_json::to_string(&response).unwrap_or_else(|_| {
        r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"serialize error"}}"#
            .to_owned()
    })
}

/// Convenience for binaries/tools that need the default project root.
pub fn project_root_from_args() -> PathBuf {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--project" {
            return args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
        }
        if let Some(path) = arg.strip_prefix("--project=") {
            return PathBuf::from(path);
        }
    }
    PathBuf::from(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples")
            .join(name)
    }

    fn call(server: &mut McpServer, id: u64, method: &str, params: Value) -> Value {
        let line = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        })
        .to_string();
        let response = server.handle_line(&line).expect("response");
        serde_json::from_str(&response).expect("response JSON")
    }

    #[test]
    fn initialize_and_tools_list() {
        let mut server = McpServer::new(example("minimal-project"));
        let init = call(
            &mut server,
            1,
            "initialize",
            json!({ "protocolVersion": "2024-11-05" }),
        );
        assert_eq!(init["result"]["serverInfo"]["name"], SERVER_NAME);
        assert_eq!(
            init["result"]["capabilities"]["tools"]["listChanged"],
            false
        );

        let tools = call(&mut server, 2, "tools/list", json!({}));
        assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 6);
    }

    #[test]
    fn validate_tool_and_resource_work() {
        let mut server = McpServer::new(example("minimal-project"));
        let tool = call(
            &mut server,
            1,
            "tools/call",
            json!({ "name": "composition_validate", "arguments": {} }),
        );
        assert_eq!(tool["result"]["isError"], false);
        let text = tool["result"]["content"][0]["text"].as_str().unwrap();
        let payload: Value = serde_json::from_str(text).unwrap();
        assert_eq!(payload["result"]["valid"], true);

        let resource = call(
            &mut server,
            2,
            "resources/read",
            json!({ "uri": "composition://project/model" }),
        );
        assert_eq!(
            resource["result"]["contents"][0]["mimeType"],
            "application/json"
        );
        let text = resource["result"]["contents"][0]["text"].as_str().unwrap();
        let payload: Value = serde_json::from_str(text).unwrap();
        assert_eq!(
            payload["result"]["model"]["project"]["id"],
            "example.minimal"
        );
    }

    #[test]
    fn unknown_method_is_jsonrpc_error() {
        let mut server = McpServer::new(example("minimal-project"));
        let response = call(&mut server, 1, "nope", json!({}));
        assert_eq!(response["error"]["code"], -32601);
    }

    #[test]
    fn notifications_get_no_response() {
        let mut server = McpServer::new(example("minimal-project"));
        let line = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        assert!(server.handle_line(&line.to_string()).is_none());
    }
}
