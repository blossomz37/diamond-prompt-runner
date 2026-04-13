#!/bin/bash
# Illustrative helper template for Diamond Prompt Runner workflows.
#
# Do not run this unchanged on another machine or repo.
# First map:
# - which outputs are generated artifacts
# - which files need backup vs deletion
# - whether append behavior is in use
# - whether reruns should clear one file or many files

set -euo pipefail

TARGET_FILE="${1:-path/to/generated-output.md}"
TIMESTAMP="$(date +%Y%m%d%H%M%S)"
BACKUP_FILE="${TARGET_FILE}.backup.${TIMESTAMP}"

if [ -f "$TARGET_FILE" ] && [ -s "$TARGET_FILE" ]; then
  cp "$TARGET_FILE" "$BACKUP_FILE"
  echo "[*] Backed up $TARGET_FILE -> $BACKUP_FILE"
fi

: > "$TARGET_FILE"
echo "[*] Cleared $TARGET_FILE"
