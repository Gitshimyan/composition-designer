#!/usr/bin/env node
import { existsSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "..");

function fail(message) {
  console.error(message);
  process.exit(2);
}

const args = process.argv.slice(2);
let project = "";
let destination = join(here, "mcp.local.json");
let profile = "";
for (let index = 0; index < args.length; index += 1) {
  const arg = args[index];
  if (arg === "--project") {
    project = args[++index] ?? "";
  } else if (arg === "--profile") {
    profile = args[++index] ?? "";
  } else if (arg === "--dest") {
    destination = args[++index] ?? "";
  } else if (arg === "-h" || arg === "--help") {
    console.log(
      "Usage: node agent/configure.mjs --project <dir> [--profile <name>] [--dest <file>]",
    );
    process.exit(0);
  } else {
    fail(`unknown argument ${arg}`);
  }
}

if (!project) {
  fail("missing --project <dir>");
}
if (!existsSync(project)) {
  fail(`project directory not found: ${project}`);
}

const executable = join(
  root,
  "target",
  "debug",
  process.platform === "win32" ? "composition-mcp.exe" : "composition-mcp",
);
if (!existsSync(executable)) {
  fail("composition-mcp is not built. Run: cargo build -p composition-mcp");
}

const config = {
  mcpServers: {
    "composition-designer": {
      command: executable,
      args: profile
        ? ["--project", resolve(project), "--profile", profile]
        : ["--project", resolve(project)],
    },
  },
};
writeFileSync(destination, `${JSON.stringify(config, null, 2)}\n`, "utf8");
console.log(`Wrote ${destination}`);
console.log(
  "Point your MCP client at that config; stdout of composition-mcp is JSON-RPC only.",
);
