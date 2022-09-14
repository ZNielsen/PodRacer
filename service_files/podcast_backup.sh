#!/bin/bash

if [[ "$#" -lt "2" ]]; then
    echo "Error: need destination and port"
    exit 1
fi

dest="$1"
port="$2"

podcast_dir="/etc/podracer/podcasts"

# Zip up podcast dir
tmp_dir="$(/bin/mktemp -d)"
bak="$tmp_dir/podcast.bak.zip"
/bin/zip -r "$bak"  "$podcast_dir"

# Push that zip to the backup location
echo "Pushing $bak to $dest on port $port"
/bin/scp -P $port "$bak" "$dest"
