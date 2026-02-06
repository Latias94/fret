param(
  [Parameter(Mandatory = $false)]
  [ValidateSet("lucide", "radix", "all")]
  [string]$Pack = "all",

  [Parameter(Mandatory = $false)]
  [switch]$Clean
)

$ErrorActionPreference = "Stop"

function Find-Python() {
  $candidates = @(
    [pscustomobject]@{ Cmd = "python"; Args = @() },
    [pscustomobject]@{ Cmd = "python3"; Args = @() },
    [pscustomobject]@{ Cmd = "py"; Args = @("-3") }
  )

  foreach ($candidate in $candidates) {
    $cmd = Get-Command $candidate.Cmd -ErrorAction SilentlyContinue
    if ($null -ne $cmd) {
      return $candidate
    }
  }

  return $null
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

$py = Find-Python
if ($null -eq $py) {
  throw "No Python interpreter found (tried: python, python3, py -3)."
}

$args = @(
  (Join-Path $repoRoot "tools/sync_icons.py"),
  "--pack", $Pack
)
if ($Clean) {
  $args += "--clean"
}

& $py.Cmd @($py.Args + $args)
exit $LASTEXITCODE

