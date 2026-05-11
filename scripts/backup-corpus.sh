#!/bin/bash

# Configuration
SOURCE_VOLUME="corpus_corpus-data"
DEST_DIR="$HOME/Dropbox/corpus+"

# Ensure destination exists
mkdir -p "$DEST_DIR"

echo "[$(date)] Starting manual backup of $SOURCE_VOLUME to $DEST_DIR..."

# Use a temporary container to rsync data from the volume to the host path
# -a: archive mode (preserves permissions, etc.)
# -v: verbose
# --delete: remove files in dest that are no longer in source
docker run --rm \
    -v "$SOURCE_VOLUME":/source:ro \
    -v "$DEST_DIR":/dest \
    alpine sh -c "apk add --no-cache rsync > /dev/null && rsync -av --delete /source/ /dest/"

if [ $? -eq 0 ]; then
    echo "[$(date)] Backup completed successfully."
else
    echo "[$(date)] Backup failed!"
    exit 1
fi
