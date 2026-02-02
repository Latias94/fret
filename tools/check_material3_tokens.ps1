param(
    [string]$MaterialWebDir = "",
    [switch]$Update
)

$ErrorActionPreference = "Stop"

function Assert-ExitCode {
    param(
        [string]$Name,
        [int]$ExitCode
    )

    if ($ExitCode -ne 0) {
        Write-Host "$Name failed with exit code $ExitCode" -ForegroundColor Red
        exit $ExitCode
    }
}

$repoRoot = & git rev-parse --show-toplevel
Assert-ExitCode "git rev-parse --show-toplevel" $LASTEXITCODE

Set-Location $repoRoot

Write-Host "Repo: $repoRoot"

$materialWeb = $MaterialWebDir
if ([string]::IsNullOrWhiteSpace($materialWeb)) {
    $commonDir = & git rev-parse --git-common-dir
    Assert-ExitCode "git rev-parse --git-common-dir" $LASTEXITCODE
    $repoRootFromCommon = Split-Path -Parent $commonDir
    $materialWeb = Join-Path $repoRootFromCommon "repo-ref\\material-web"
}

if (Test-Path $materialWeb) {
    $materialWebRev = & git -C $materialWeb rev-parse HEAD 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Material Web: $materialWeb"
        Write-Host "Material Web rev: $materialWebRev"
    } else {
        Write-Host "Material Web: $materialWeb (rev unknown)"
    }
} else {
    Write-Host "Material Web: not found at $materialWeb (token tools will attempt auto-discovery or env overrides)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Token import check..."
& cargo run -p fret-ui-material3 --bin material3_token_import -- --check
$importCode = $LASTEXITCODE

if ($importCode -ne 0 -and $Update) {
    Write-Host ""
    Write-Host "Updating generated token output..."
    & cargo run -p fret-ui-material3 --bin material3_token_import
    Assert-ExitCode "material3_token_import (update)" $LASTEXITCODE

    Write-Host ""
    Write-Host "Re-checking token import..."
    & cargo run -p fret-ui-material3 --bin material3_token_import -- --check
    Assert-ExitCode "material3_token_import --check (after update)" $LASTEXITCODE
} else {
    Assert-ExitCode "material3_token_import --check" $importCode
}

Write-Host ""
Write-Host "Token audit check..."
& cargo run -p fret-ui-material3 --bin material3_token_audit -- --check
Assert-ExitCode "material3_token_audit --check" $LASTEXITCODE

Write-Host ""
Write-Host "OK"
exit 0

