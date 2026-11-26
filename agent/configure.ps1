param(
    [Parameter(Mandatory = $true)]
    [string]$Project,
    [string]$Destination = (Join-Path $PSScriptRoot "mcp.local.json")
)

$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root "target\debug\composition-mcp.exe"
if (-not (Test-Path $exe)) {
    throw "composition-mcp is not built. Run: cargo build -p composition-mcp"
}
if (-not (Test-Path $Project)) {
    throw "Project directory not found: $Project"
}

$config = @{
    mcpServers = @{
        "composition-designer" = @{
            command = $exe
            args = @("--project", (Resolve-Path $Project).Path)
        }
    }
} | ConvertTo-Json -Depth 6

Set-Content -Path $Destination -Value $config -Encoding utf8 -NoNewline
Write-Output "Wrote $Destination"
Write-Output "Point your MCP client at that config; stdout of composition-mcp is JSON-RPC only."
