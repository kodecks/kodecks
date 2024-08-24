#!/bin/bash

ASSETS_DIR="./kodecks-bevy/assets"

for txt_file in "$ASSETS_DIR"/*.txt; do
    base_name=$(basename "$txt_file" .txt)
    mkdir -p "$ASSETS_DIR/$base_name"

    while IFS= read -r url; do
        file_name=$(basename "$url")
        curl -L -o "$ASSETS_DIR/$base_name/$file_name" "$url"
    done < "$txt_file"
done

echo "Download complete!"
