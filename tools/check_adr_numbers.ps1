Param(
    [string]$AdrDir = "docs/adr"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $AdrDir)) {
    throw "ADR directory not found: $AdrDir"
}

$items = Get-ChildItem -Path $AdrDir -Filter "*.md" -File
$groups = @{}

foreach ($item in $items) {
    if ($item.Name -match '^(\d+)-') {
        $id = [int]$Matches[1]
        if (-not $groups.ContainsKey($id)) {
            $groups[$id] = New-Object System.Collections.Generic.List[string]
        }
        $groups[$id].Add($item.Name)
    }
}

$dups = $groups.GetEnumerator() | Where-Object { $_.Value.Count -gt 1 } | Sort-Object Name

if ($dups.Count -gt 0) {
    Write-Host "Duplicate ADR IDs found in ${AdrDir}:" -ForegroundColor Red
    foreach ($dup in $dups) {
        $names = ($dup.Value | Sort-Object) -join ", "
        Write-Host ("  {0}: {1}" -f $dup.Name, $names) -ForegroundColor Red
    }
    exit 1
}

Write-Host "ADR ID check passed."
