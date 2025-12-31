$ErrorActionPreference = "Continue"
Set-StrictMode -Version Latest

function Write-RuleViolation {
    param(
        [Parameter(Mandatory = $true)][string]$Rule,
        [Parameter(Mandatory = $true)][string]$From,
        [Parameter(Mandatory = $true)][string]$To
    )
    Write-Error -ErrorAction Continue ("Layering violation ({0}): {1} must not depend on {2}" -f $Rule, $From, $To)
    $script:HadErrors = $true
}

function MatchesAnyPattern {
    param(
        [Parameter(Mandatory = $true)][string]$Value,
        [Parameter(Mandatory = $true)][string[]]$Patterns
    )
    foreach ($pattern in $Patterns) {
        if ($Value -match $pattern) {
            return $true
        }
    }
    return $false
}

function AssertNoWorkspaceDepsMatching {
    param(
        [Parameter(Mandatory = $true)][string]$From,
        [Parameter(Mandatory = $true)][string]$Rule,
        [string[]]$Deps = @(),
        [Parameter(Mandatory = $true)][string[]]$ForbiddenPatterns
    )

    foreach ($to in $Deps) {
        if (MatchesAnyPattern -Value $to -Patterns $ForbiddenPatterns) {
            Write-RuleViolation -Rule $Rule -From $From -To $to
        }
    }
}

$script:HadErrors = $false

$metadataJson = & cargo metadata --format-version 1
$metadata = $metadataJson | ConvertFrom-Json

$workspaceIds = @{}
foreach ($id in $metadata.workspace_members) {
    $workspaceIds[$id] = $true
}

$idToName = @{}
$nameToPackage = @{}
foreach ($pkg in $metadata.packages) {
    $idToName[$pkg.id] = $pkg.name
    $nameToPackage[$pkg.name] = $pkg
}

$depsByFrom = @{}
foreach ($node in $metadata.resolve.nodes) {
    if (-not $workspaceIds.ContainsKey($node.id)) {
        continue
    }
    $from = $idToName[$node.id]
    if (-not $depsByFrom.ContainsKey($from)) {
        $depsByFrom[$from] = New-Object System.Collections.Generic.List[string]
    }

    foreach ($dep in $node.deps) {
        $toId = $dep.pkg
        if (-not $workspaceIds.ContainsKey($toId)) {
            continue
        }
        $to = $idToName[$toId]
        $depsByFrom[$from].Add($to)
    }
}

# Workspace-level crate boundary checks (ADR 0037 / docs/dependency-policy.md).
#
# Notes:
# - We only validate workspace->workspace edges here (fast + stable).
# - Cargo already prevents cycles; this script focuses on "do not leak backend crates into contracts/components".

$platformPatterns = @("^fret-platform($|-)")
$rendererPatterns = @("^fret-render($|-)")
$runnerPatterns = @("^fret-runner($|-)")
$componentsPatterns = @("^fret-components-")

# 1) `fret-core` must not depend on any other workspace crate.
if ($depsByFrom.ContainsKey("fret-core")) {
    foreach ($to in $depsByFrom["fret-core"]) {
        Write-RuleViolation -Rule "core-is-leaf" -From "fret-core" -To $to
    }
}

# 2) Runtime + UI substrate must not depend on backend crates.
foreach ($from in @("fret-runtime", "fret-app", "fret-ui")) {
    if (-not $depsByFrom.ContainsKey($from)) {
        continue
    }
    AssertNoWorkspaceDepsMatching -From $from -Rule "portable-no-backends" -Deps $depsByFrom[$from] -ForbiddenPatterns ($platformPatterns + $rendererPatterns + $runnerPatterns)
}

# 3) Component crates must not depend on platform/render/runner crates.
foreach ($kv in $depsByFrom.GetEnumerator()) {
    $from = [string]$kv.Key
    if (-not (MatchesAnyPattern -Value $from -Patterns $componentsPatterns)) {
        continue
    }
    AssertNoWorkspaceDepsMatching -From $from -Rule "components-no-backends" -Deps $kv.Value -ForbiddenPatterns ($platformPatterns + $rendererPatterns + $runnerPatterns)
}

# 4) Backend crates must not depend on UI/component crates.
foreach ($from in @("fret-render", "fret-platform", "fret-platform-winit")) {
    if (-not $depsByFrom.ContainsKey($from)) {
        continue
    }
    AssertNoWorkspaceDepsMatching -From $from -Rule "backends-no-ui" -Deps $depsByFrom[$from] -ForbiddenPatterns (@("^fret-ui$") + $componentsPatterns)
}

# 5) Runner may depend on everything; no checks here (it is the wiring crate).

# External dependency checks for portable crates (cheap sanity guards).
function AssertNoExternalDeps {
    param(
        [Parameter(Mandatory = $true)][string]$Crate,
        [Parameter(Mandatory = $true)][string]$Rule,
        [Parameter(Mandatory = $true)][string[]]$ForbiddenDepNames
    )

    if (-not $nameToPackage.ContainsKey($Crate)) {
        return
    }

    $pkg = $nameToPackage[$Crate]
    foreach ($dep in $pkg.dependencies) {
        # Only enforce normal dependencies; allow dev/build deps to vary without breaking portability.
        if ($null -ne $dep.kind -and $dep.kind -ne "normal") {
            continue
        }
        if ($ForbiddenDepNames -contains $dep.name) {
            Write-Error -ErrorAction Continue ("Layering violation ({0}): {1} must not depend on external crate {2}" -f $Rule, $Crate, $dep.name)
            $script:HadErrors = $true
        }
    }
}

$forbiddenInPortable = @("wgpu", "winit", "taffy", "accesskit", "accesskit_winit", "cosmic-text", "lyon", "resvg", "usvg", "arboard", "rfd", "webbrowser")

AssertNoExternalDeps -Crate "fret-core" -Rule "core-portable-deps" -ForbiddenDepNames $forbiddenInPortable
AssertNoExternalDeps -Crate "fret-runtime" -Rule "runtime-portable-deps" -ForbiddenDepNames $forbiddenInPortable
AssertNoExternalDeps -Crate "fret-app" -Rule "app-portable-deps" -ForbiddenDepNames $forbiddenInPortable
AssertNoExternalDeps -Crate "fret-platform" -Rule "platform-contracts-portable-deps" -ForbiddenDepNames $forbiddenInPortable

if ($script:HadErrors) {
    exit 1
}

Write-Host "Layering check passed."
