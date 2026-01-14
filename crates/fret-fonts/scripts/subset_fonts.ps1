param(
  [string]$AssetsDir = "$(Split-Path -Parent $PSScriptRoot)\assets"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not (Get-Command pyftsubset -ErrorAction SilentlyContinue)) {
  throw "pyftsubset not found. Install with: python -m pip install fonttools brotli"
}

$unicodes = @(
  "U+000A",
  "U+000D",
  "U+0020-007E",
  "U+00A0-00FF",
  "U+0100-017F",
  "U+0180-024F",
  "U+2000-206F",
  "U+20A0-20CF",
  "U+2190-21FF",
  "U+2200-22FF",
  "U+2300-23FF",
  "U+2460-24FF",
  "U+2500-257F",
  "U+2580-259F",
  "U+25A0-25FF"
) -join ","

$commonArgs = @(
  "--unicodes=$unicodes",
  "--layout-features=*",
  "--glyph-names",
  "--symbol-cmap",
  "--notdef-glyph",
  "--notdef-outline",
  "--recommended-glyphs",
  "--name-IDs=*",
  "--name-legacy",
  "--name-languages=*",
  "--drop-tables+=DSIG",
  "--no-hinting"
)

$fonts = @(
  @{ In = "Inter-roman.ttf"; Out = "Inter-roman-subset.ttf" },
  @{ In = "Inter-italic.ttf"; Out = "Inter-italic-subset.ttf" },
  @{ In = "JetBrainsMono-roman.ttf"; Out = "JetBrainsMono-roman-subset.ttf" },
  @{ In = "JetBrainsMono-italic.ttf"; Out = "JetBrainsMono-italic-subset.ttf" }
)

foreach ($f in $fonts) {
  $inPath = Join-Path $AssetsDir $f.In
  $outPath = Join-Path $AssetsDir $f.Out
  if (-not (Test-Path $inPath)) {
    throw "Missing input font: $inPath"
  }
  pyftsubset $inPath @commonArgs --output-file=$outPath | Out-Null
}

Get-ChildItem $AssetsDir -Filter "*-subset.ttf" | Sort-Object Name | Format-Table Name, Length

