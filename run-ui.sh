#!/bin/bash
cd "$(dirname "$0")/corpus-ui"
echo "Starting Corpus+ frontend dev server..."
npm run --host dev
