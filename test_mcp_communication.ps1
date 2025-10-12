# MCP Communication Test Script
# Tests JSON-RPC communication with Codex MCP Server

Write-Host "=== MCP Communication Test ===" -ForegroundColor Cyan
Write-Host ""

# Test 1: Start MCP server in background
Write-Host "1. Starting MCP server in background..." -ForegroundColor Yellow
$mcpProcess = Start-Process -FilePath "codex" -ArgumentList "mcp-server" -NoNewWindow -PassThru -RedirectStandardInput "mcp_input.txt" -RedirectStandardOutput "mcp_output.txt" -RedirectStandardError "mcp_error.txt"

Start-Sleep -Seconds 3

if ($mcpProcess.HasExited) {
    Write-Host "   [ERROR] MCP server exited unexpectedly" -ForegroundColor Red
    Get-Content mcp_error.txt | Write-Host -ForegroundColor Red
    exit 1
} else {
    Write-Host "   [OK] MCP server running (PID: $($mcpProcess.Id))" -ForegroundColor Green
}

# Test 2: Send initialize request
Write-Host "`n2. Sending initialize request..." -ForegroundColor Yellow

$initRequest = @{
    jsonrpc = "2.0"
    id = 1
    method = "initialize"
    params = @{
        protocolVersion = "2024-11-05"
        capabilities = @{
            roots = @{
                listChanged = $true
            }
        }
        clientInfo = @{
            name = "test-client"
            version = "1.0.0"
        }
    }
} | ConvertTo-Json -Depth 10

$initRequest | Out-File -FilePath "mcp_input.txt" -Encoding UTF8 -NoNewline

Start-Sleep -Seconds 2

if (Test-Path mcp_output.txt) {
    $response = Get-Content mcp_output.txt -Raw
    if ($response) {
        Write-Host "   [OK] Received response" -ForegroundColor Green
        Write-Host "   Response length: $($response.Length) bytes" -ForegroundColor Gray
    } else {
        Write-Host "   [WARN] No response received (server may be waiting for complete message)" -ForegroundColor Yellow
    }
}

# Test 3: Check error log
Write-Host "`n3. Checking error log..." -ForegroundColor Yellow
if (Test-Path mcp_error.txt) {
    $errors = Get-Content mcp_error.txt
    if ($errors) {
        Write-Host "   [WARN] Errors found:" -ForegroundColor Yellow
        $errors | Select-Object -First 10 | ForEach-Object { Write-Host "   $_" -ForegroundColor Red }
    } else {
        Write-Host "   [OK] No errors" -ForegroundColor Green
    }
}

# Cleanup
Write-Host "`n4. Cleanup..." -ForegroundColor Yellow
Stop-Process -Id $mcpProcess.Id -Force -ErrorAction SilentlyContinue
Remove-Item mcp_input.txt -ErrorAction SilentlyContinue
Remove-Item mcp_output.txt -ErrorAction SilentlyContinue
Remove-Item mcp_error.txt -ErrorAction SilentlyContinue
Write-Host "   [OK] Cleanup complete" -ForegroundColor Green

Write-Host "`n=== Test Complete ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Summary:" -ForegroundColor Yellow
Write-Host "  - MCP server can be started: YES" -ForegroundColor Green
Write-Host "  - Server accepts stdio input: NEEDS VERIFICATION" -ForegroundColor Yellow
Write-Host "  - JSON-RPC communication: NEEDS FULL TEST" -ForegroundColor Yellow
Write-Host ""
Write-Host "Note: Full MCP testing requires a proper MCP client." -ForegroundColor Gray
Write-Host "Consider using MCP Inspector or custom test client for comprehensive testing." -ForegroundColor Gray

