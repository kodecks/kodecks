#!/bin/bash

find kodecks-bevy -iname "*.png" | while read -r file; do
  optipng -quiet -i 0 -strip all -zc1-9 -zm1-9 -zs0-3 -f0-5 "$file"
  zopflipng --lossy_transparent -m -y "$file" "$file"
done