#!/bin/bash

# Configuration
DEST_VOLUME="corpus_corpus-data"
SOURCE_DIR="$HOME/Dropbox/corpus+"

# Check if source directory exists
if [ ! -d "$SOURCE_DIR" ]; then
    echo "Error: Source directory $SOURCE_DIR does not exist."
    exit 1
fi

echo "[$(date)] Starting manual restore from $SOURCE_DIR to $DEST_VOLUME..."
echo "WARNING: This will overwrite data in the volume '$DEST_VOLUME'. Continue? (y/n)"
read -r response
if [[ ! "$response" =~ ^[Yy]$ ]]; then
    echo "Restore cancelled."
    exit 0
fi

# Use a temporary container to rsync data from the host path back to the volume
docker run --rm \
    -v "$DEST_VOLUME":/dest \
    -v "$SOURCE_DIR":/source:ro \
    alpine sh -c "apk add --no-cache rsync > /dev/null && rsync -av --delete /source/ /dest/"

if [ $? -eq 0 ]; then
    echo "[$(date)] Restore completed successfully."
else
    echo "[$(date)] Restore failed!"
    exit 1
fi
