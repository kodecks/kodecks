param (
    [string]$name,
    [string]$target
)

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$env:BEVY_ASSET_PATH = Join-Path -Path $scriptDir -ChildPath "..\kodecks-bevy\assets"
if ("" -ne $target) {
    cross build --release --features embed_assets --target $target
    $exePath = Join-Path -Path $scriptDir -ChildPath "..\target\$target\release\kodecks-bevy.exe"
} else {
    cargo build --release --features embed_assets
    $exePath = Join-Path -Path $scriptDir -ChildPath "..\target\release\kodecks-bevy.exe"
}

$copiedExePath = Join-Path -Path $scriptDir -ChildPath "..\target\kodecks.exe"
$zipPath = Join-Path -Path $scriptDir -ChildPath "kodecks-$name.zip"

Copy-Item -Path $exePath -Destination $copiedExePath
Compress-Archive -Update -Path $copiedExePath -DestinationPath $zipPath