param(
    [string[]]$Paths = @(
        "ecosystem/fret-ui-shadcn/src",
        "ecosystem/fret-ui-kit/src"
    )
)

$ErrorActionPreference = "Stop"
$pattern = "theme\\.colors\\.|theme\\.metrics\\."

function Assert-NoMatches {
    param(
        [string[]]$Matches
    )

    if ($Matches.Count -gt 0) {
        Write-Host "Found forbidden typed theme reads:" -ForegroundColor Red
        $Matches | ForEach-Object { Write-Host $_ }
        exit 1
    }
}

$rg = Get-Command rg -ErrorAction SilentlyContinue
if ($null -ne $rg) {
    $out = & $rg.Path "-n" $pattern @Paths 2>$null
    $code = $LASTEXITCODE

    if ($code -eq 0) {
        Assert-NoMatches $out
    }
    if ($code -eq 1) {
        exit 0
    }

    Write-Host "rg failed with exit code $code" -ForegroundColor Red
    exit $code
}

$matches = Get-ChildItem -Recurse -File -Path $Paths |
    Select-String -Pattern $pattern |
    ForEach-Object { "$($_.Path):$($_.LineNumber):$($_.Line.Trim())" }

Assert-NoMatches $matches
exit 0

