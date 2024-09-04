$ASSETS_DIR = "./kodecks-bevy/assets"

$txt_files = Get-ChildItem -Path "$ASSETS_DIR" -Filter *.txt

foreach ($txt_file in $txt_files) {
    $base_name = [System.IO.Path]::GetFileNameWithoutExtension($txt_file)
    $base_dir = Join-Path -Path $ASSETS_DIR -ChildPath $base_name

    if (-not (Test-Path -Path $base_dir)) {
        New-Item -ItemType Directory -Path $base_dir | Out-Null
    }

    $urls = Get-Content -Path $txt_file.FullName
    foreach ($url in $urls) {
        $url = $url.Trim()
        $file_name = [System.IO.Path]::GetFileName($url)
        $output_path = Join-Path -Path $base_dir -ChildPath $file_name
        Invoke-WebRequest -Uri $url -OutFile $output_path
    }
}