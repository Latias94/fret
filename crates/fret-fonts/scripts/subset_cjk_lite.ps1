param(
  [string]$AssetsDir = "$(Split-Path -Parent $PSScriptRoot)\assets"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not (Get-Command pyftsubset -ErrorAction SilentlyContinue)) {
  throw "pyftsubset not found. Install with: python -m pip install fonttools brotli"
}

$sourcesDir = Join-Path $AssetsDir "_sources"
$inPath = Join-Path $sourcesDir "NotoSansCJKsc-Regular.otf"
if (-not (Test-Path $inPath)) {
  throw "Missing input font: $inPath (run scripts/fetch_cjk_font.ps1 first)"
}

$textPath = Join-Path $AssetsDir "cjk-lite-text.txt"
if (-not (Test-Path $textPath)) {
  $genScript = Join-Path $PSScriptRoot "generate_cjk_lite_text.py"
  if (-not (Test-Path $genScript)) {
    throw "Missing generator script: $genScript"
  }
  python $genScript --out $textPath
}

$outPath = Join-Path $AssetsDir "NotoSansCJKsc-Regular-cjk-lite-subset.otf"

$commonArgs = @(
  "--text-file=$textPath",
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
  "--no-hinting",
  "--output-file=$outPath"
)

pyftsubset $inPath @commonArgs | Out-Null

Get-ChildItem $AssetsDir | Where-Object Name -like "NotoSansCJKsc-Regular-cjk-lite-subset.otf" | Format-Table Name, Length
