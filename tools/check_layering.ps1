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

# Ecosystem crates are incubated in-tree (see docs/repo-structure.md) and are expected to be
# extractable. As a default rule, they should remain backend-agnostic (no platform/render/runner
# deps), with a small explicit allowlist for wiring-heavy crates.
$ecosystemAllowBackendDeps = @(
    # `fret-bootstrap` is intentionally "golden path wiring" and may need direct renderer hooks.
    "fret-bootstrap"
)

$ecosystemNameSet = @{}
foreach ($pkg in $metadata.packages) {
    if (-not $workspaceIds.ContainsKey($pkg.id)) {
        continue
    }
    $manifest = [string]$pkg.manifest_path
    if ($manifest -match "\\\\ecosystem\\\\" -or $manifest -match "/ecosystem/") {
        $ecosystemNameSet[$pkg.name] = $true
    }
}

$kernelNameSet = @{}
foreach ($pkg in $metadata.packages) {
    if (-not $workspaceIds.ContainsKey($pkg.id)) {
        continue
    }
    $manifest = [string]$pkg.manifest_path
    if ($manifest -match "\\\\crates\\\\" -or $manifest -match "/crates/") {
        $kernelNameSet[$pkg.name] = $true
    }
}

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

# 3.5) Ecosystem crates must not depend on platform/render/runner crates (unless explicitly allowlisted).
foreach ($kv in $depsByFrom.GetEnumerator()) {
    $from = [string]$kv.Key
    if (-not $ecosystemNameSet.ContainsKey($from)) {
        continue
    }
    if ($ecosystemAllowBackendDeps -contains $from) {
        continue
    }
    AssertNoWorkspaceDepsMatching -From $from -Rule "ecosystem-no-backends" -Deps $kv.Value -ForbiddenPatterns ($platformPatterns + $rendererPatterns + $runnerPatterns)
}

# 3.75) Kernel (`crates/*`) must not depend on ecosystem crates.
#
# Rationale: ecosystem crates are incubated in-tree but should remain extractable (docs/repo-structure.md).
foreach ($kv in $depsByFrom.GetEnumerator()) {
    $from = [string]$kv.Key
    if (-not $kernelNameSet.ContainsKey($from)) {
        continue
    }
    foreach ($to in $kv.Value) {
        if ($ecosystemNameSet.ContainsKey($to)) {
            Write-RuleViolation -Rule "kernel-no-ecosystem" -From $from -To $to
        }
    }
}

# 4) Backend crates must not depend on UI/component crates.
foreach ($from in @(
    "fret-render",
    "fret-render-core",
    "fret-render-wgpu",
    "fret-platform",
    "fret-platform-native",
    "fret-platform-web"
)) {
    if (-not $depsByFrom.ContainsKey($from)) {
        continue
    }
    foreach ($to in $depsByFrom[$from]) {
        if ($to -eq "fret-ui" -or (MatchesAnyPattern -Value $to -Patterns $componentsPatterns) -or $ecosystemNameSet.ContainsKey($to)) {
            Write-RuleViolation -Rule "backends-no-ui" -From $from -To $to
        }
    }
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

# Feature usage policy: retained bridge must remain explicitly opt-in and tightly scoped.
$unstableRetainedBridgeAllowlist = @(
    "fret-chart",
    "fret-docking",
    "fret-node",
    "fret-plot",
    "fret-plot3d"
)
foreach ($pkg in $metadata.packages) {
    if (-not $workspaceIds.ContainsKey($pkg.id)) {
        continue
    }
    foreach ($dep in $pkg.dependencies) {
        if ($dep.name -ne "fret-ui") {
            continue
        }
        if ($dep.features -contains "unstable-retained-bridge") {
            if (-not ($unstableRetainedBridgeAllowlist -contains $pkg.name)) {
                Write-Error -ErrorAction Continue ("Layering violation (unstable-retained-bridge-allowlist): {0} must not enable fret-ui/unstable-retained-bridge" -f $pkg.name)
                $script:HadErrors = $true
            }
        }
    }
}

if ($script:HadErrors) {
    exit 1
}

Write-Host "Layering check passed."
