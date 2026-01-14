param(
  [string]$AssetsDir = "$(Split-Path -Parent $PSScriptRoot)\assets"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

New-Item -ItemType Directory -Force -Path $AssetsDir | Out-Null
$sourcesDir = Join-Path $AssetsDir "_sources"
New-Item -ItemType Directory -Force -Path $sourcesDir | Out-Null

$fontUrl = "https://raw.githubusercontent.com/googlefonts/noto-cjk/main/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf"
$licenseUrl = "https://raw.githubusercontent.com/notofonts/noto-fonts/main/LICENSE"

Invoke-WebRequest -Uri $fontUrl -OutFile (Join-Path $sourcesDir "NotoSansCJKsc-Regular.otf")
Invoke-WebRequest -Uri $licenseUrl -OutFile (Join-Path $AssetsDir "NotoSansCJK-LICENSE.txt")

Get-ChildItem $AssetsDir | Where-Object Name -like "NotoSansCJK*" | Sort-Object Name | Format-Table Name, Length
Get-ChildItem $sourcesDir | Where-Object Name -like "NotoSansCJK*" | Sort-Object Name | Format-Table Name, Length
