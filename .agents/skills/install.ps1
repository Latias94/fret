<# 
Fret skills installer (repo-local source -> project-local agent directory)

This script copies skill folders (each containing `SKILL.md`) from this repo's `.agents/skills/`
directory into a target project's agent skills directory:

- Claude Code: <project>/.claude/skills/
- Codex CLI:   <project>/.agents/skills/
- Gemini CLI:  <project>/.gemini/skills/

Examples:
  # Install all fret-* skills into the current project for Claude Code
  powershell -ExecutionPolicy Bypass -File .\.agents\skills\install.ps1 -Agent claude

  # Install all skills into another project for Codex
  powershell -ExecutionPolicy Bypass -File .\.agents\skills\install.ps1 -Agent codex -Target E:\Rust\my-app

  # Install only selected skills (space-separated)
  powershell -ExecutionPolicy Bypass -File .\.agents\skills\install.ps1 -Agent claude -Skills fret-diag-workflow,fret-shadcn-app-recipes -Force
#>

[CmdletBinding()]
param(
  [Parameter(Mandatory = $false)]
  [ValidateSet("claude", "claude-code", "codex", "gemini")]
  [string]$Agent = "claude",

  [Parameter(Mandatory = $false)]
  [string]$Target = ".",

  [Parameter(Mandatory = $false)]
  [string[]]$Skills,

  [Parameter(Mandatory = $false)]
  [switch]$Force,

  [Parameter(Mandatory = $false)]
  [switch]$List,

  [Parameter(Mandatory = $false)]
  [switch]$DryRun
)

$ErrorActionPreference = "Stop"

function Info([string]$Message)  { Write-Host "[INFO] $Message" -ForegroundColor Cyan }
function Warn([string]$Message)  { Write-Host "[WARN] $Message" -ForegroundColor Yellow }
function Ok([string]$Message)    { Write-Host "[OK]   $Message" -ForegroundColor Green }
function Die([string]$Message)   { Write-Host "[ERROR] $Message" -ForegroundColor Red; exit 1 }

function Normalize-Agent([string]$A) {
  switch ($A) {
    "claude" { return "claude" }
    "claude-code" { return "claude" }
    "codex" { return "codex" }
    "gemini" { return "gemini" }
    default { Die "Unknown agent: $A" }
  }
}

function Skills-Dest-Dir([string]$NormalizedAgent, [string]$TargetAbs) {
  switch ($NormalizedAgent) {
    "codex"  { return (Join-Path $TargetAbs ".agents\\skills") }
    "gemini" { return (Join-Path $TargetAbs ".gemini\\skills") }
    "claude" { return (Join-Path $TargetAbs ".claude\\skills") }
    default  { Die "Unknown agent: $NormalizedAgent" }
  }
}

$sourceRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$targetAbs = (Resolve-Path -LiteralPath $Target).Path
$agentNorm = Normalize-Agent $Agent
$destDir = Skills-Dest-Dir $agentNorm $targetAbs

if (!(Test-Path -LiteralPath $sourceRoot)) {
  Die "Cannot find skills source directory: $sourceRoot"
}

$available = Get-ChildItem -LiteralPath $sourceRoot -Directory -Force |
  Where-Object { $_.Name -like "fret-*" -and (Test-Path -LiteralPath (Join-Path $_.FullName "SKILL.md")) } |
  Sort-Object Name

if ($List) {
  Info "Available skills in $sourceRoot:"
  foreach ($s in $available) { Write-Host "  - $($s.Name)" }
  exit 0
}

if ($available.Count -eq 0) {
  Die "No skills found under $sourceRoot (expected folders like fret-*/SKILL.md)"
}

$requested = @()
if ($Skills -and $Skills.Count -gt 0) {
  # Accept both comma-separated and array inputs
  $requested = $Skills | ForEach-Object { $_ -split "," } | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne "" }
} else {
  $requested = $available | ForEach-Object { $_.Name }
}

$missing = $requested | Where-Object { $_ -notin ($available | ForEach-Object { $_.Name }) }
if ($missing.Count -gt 0) {
  Die ("Unknown skill(s): " + ($missing -join ", ") + ". Use -List to see available skills.")
}

Info "Source: $sourceRoot"
Info "Target: $targetAbs"
Info "Agent:  $agentNorm"
Info "Dest:   $destDir"
Info ("Skills: " + ($requested -join ", "))

if (!(Test-Path -LiteralPath $destDir)) {
  if ($DryRun) {
    Info "Dry run: would create $destDir"
  } else {
    New-Item -ItemType Directory -Force -Path $destDir | Out-Null
  }
}

foreach ($name in $requested) {
  $src = Join-Path $sourceRoot $name
  $dst = Join-Path $destDir $name

  if (Test-Path -LiteralPath $dst) {
    if ($Force) {
      if ($DryRun) {
        Warn "Dry run: would remove existing $dst"
      } else {
        Remove-Item -LiteralPath $dst -Recurse -Force
      }
    } else {
      Warn "Skip (already exists): $dst (use -Force to overwrite)"
      continue
    }
  }

  if ($DryRun) {
    Info "Dry run: would copy $src -> $dst"
  } else {
    Copy-Item -LiteralPath $src -Destination $dst -Recurse -Force
    Ok "Installed: $name"
  }
}

Ok "Done."
