param(
  [string]$AssetsDir = "$(Split-Path -Parent $PSScriptRoot)\assets"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

New-Item -ItemType Directory -Force -Path $AssetsDir | Out-Null

$fontUrl = "https://raw.githubusercontent.com/googlefonts/noto-emoji/main/fonts/NotoColorEmoji.ttf"
$licenseUrl = "https://raw.githubusercontent.com/googlefonts/noto-emoji/main/LICENSE"

Invoke-WebRequest -Uri $fontUrl -OutFile (Join-Path $AssetsDir "NotoColorEmoji.ttf")
Invoke-WebRequest -Uri $licenseUrl -OutFile (Join-Path $AssetsDir "NotoEmoji-LICENSE.txt")

Get-ChildItem $AssetsDir | Where-Object Name -like "Noto*" | Sort-Object Name | Format-Table Name, Length

