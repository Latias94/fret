param(
    [Parameter(Mandatory = $false)]
    [string]$SkillsRoot = ".agents/skills",
    [Parameter(Mandatory = $false)]
    [switch]$Strict
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Unquote([string]$s) {
    $s = $s.Trim()
    if (($s.StartsWith('"') -and $s.EndsWith('"')) -or ($s.StartsWith("'") -and $s.EndsWith("'"))) {
        return $s.Substring(1, $s.Length - 2)
    }
    return $s
}

function Parse-Frontmatter([string[]]$frontmatterLines) {
    $name = $null
    $description = $null

    for ($i = 0; $i -lt $frontmatterLines.Length; $i++) {
        $line = $frontmatterLines[$i]

        if ($line -match '^\s*name:\s*(.+)\s*$') {
            $name = Unquote($Matches[1])
            continue
        }

        if ($line -match '^\s*description:\s*(.+)\s*$') {
            $raw = $Matches[1].Trim()
            if ($raw -eq "|" -or $raw -eq ">") {
                $block = New-Object System.Collections.Generic.List[string]
                for ($j = $i + 1; $j -lt $frontmatterLines.Length; $j++) {
                    $l = $frontmatterLines[$j]
                    if ($l -match '^\s*[\w-]+\s*:') {
                        break
                    }
                    $block.Add(($l -replace '^\s{0,2}', ''))
                    $i = $j
                }
                $description = ($block -join "`n").Trim()
            } else {
                $description = Unquote($raw)
            }
            continue
        }
    }

    return [PSCustomObject]@{
        name        = $name
        description = $description
    }
}

$errors = New-Object System.Collections.Generic.List[string]
$warnings = New-Object System.Collections.Generic.List[string]

if (!(Test-Path $SkillsRoot)) {
    throw "SkillsRoot does not exist: $SkillsRoot"
}

$skillDirs = Get-ChildItem -Path $SkillsRoot -Directory
if ($skillDirs.Count -eq 0) {
    Write-Host "No skills found under $SkillsRoot"
    exit 0
}

$nameRegex = '^[a-z0-9]+(?:-[a-z0-9]+)*$'
$recommendedHeadings = @(
    "## When to use",
    "## Quick start",
    "## Workflow",
    "## Evidence anchors",
    "## Common pitfalls",
    "## Related skills"
)

foreach ($dir in $skillDirs) {
    $skillDirName = $dir.Name
    $skillPath = $dir.FullName
    $skillFile = Join-Path $skillPath "SKILL.md"

    if (!(Test-Path $skillFile)) {
        $errors.Add("Missing SKILL.md: $skillDirName")
        continue
    }

    $lines = Get-Content -Path $skillFile
    if ($lines.Length -lt 3 -or $lines[0].Trim() -ne "---") {
        $errors.Add("Invalid frontmatter (missing leading ---): $skillDirName")
        continue
    }

    $endIndex = -1
    for ($i = 1; $i -lt $lines.Length; $i++) {
        if ($lines[$i].Trim() -eq "---") {
            $endIndex = $i
            break
        }
    }
    if ($endIndex -lt 0) {
        $errors.Add("Invalid frontmatter (missing closing ---): $skillDirName")
        continue
    }

    $frontmatter = $lines[1..($endIndex - 1)]
    $props = Parse-Frontmatter $frontmatter

    if ([string]::IsNullOrWhiteSpace($props.name)) {
        $errors.Add("Frontmatter missing name: $skillDirName")
    } elseif ($props.name -ne $skillDirName) {
        $errors.Add("Skill name mismatch: dir='$skillDirName' frontmatter.name='$($props.name)'")
    } elseif ($props.name.Length -gt 64) {
        $errors.Add("Skill name too long (>64): $skillDirName")
    } elseif ($props.name -notmatch $nameRegex) {
        $errors.Add("Skill name invalid (expected lowercase-hyphen): $skillDirName")
    }

    if ([string]::IsNullOrWhiteSpace($props.description)) {
        $errors.Add("Frontmatter missing description: $skillDirName")
    } elseif ($props.description.Length -gt 1024) {
        $errors.Add("Description too long (>1024 chars): $skillDirName")
    }

    $body = ($lines[($endIndex + 1)..($lines.Length - 1)] -join "`n")
    foreach ($h in $recommendedHeadings) {
        if ($body -notmatch [regex]::Escape($h)) {
            $warnings.Add("[$skillDirName] Missing recommended heading: $h")
        }
    }
}

if ($errors.Count -gt 0) {
    Write-Host "Skill validation errors:" -ForegroundColor Red
    $errors | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
}

if ($warnings.Count -gt 0) {
    Write-Host "Skill validation warnings:" -ForegroundColor Yellow
    $warnings | ForEach-Object { Write-Host "  - $_" -ForegroundColor Yellow }
}

if ($errors.Count -gt 0) {
    exit 1
}

if ($Strict -and $warnings.Count -gt 0) {
    exit 1
}

Write-Host "Skills OK ($($skillDirs.Count) checked)."
exit 0

