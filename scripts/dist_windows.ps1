$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$env:BEVY_ASSET_PATH = Join-Path -Path $scriptDir -ChildPath "..\kodecks-bevy\assets"
cargo build --release --features embed_assets

$exePath = Join-Path -Path $scriptDir -ChildPath "..\target\release\kodecks-bevy.exe"
$copiedExePath = Join-Path -Path $scriptDir -ChildPath "..\target\release\kodecks.exe"
$zipPath = "kodecks-x86_64-pc-windows-msvc.zip"

Copy-Item -Path $exePath -Destination $copiedExePath
Compress-Archive -Update -Path $copiedExePath -DestinationPath $zipPath
