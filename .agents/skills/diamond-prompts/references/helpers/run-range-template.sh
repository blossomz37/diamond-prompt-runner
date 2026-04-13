#!/bin/bash
# Illustrative helper template for Diamond Prompt Runner workflows.
#
# Do not run this unchanged on another machine or repo.
# First map:
# - project path
# - pipeline id
# - loop variable name
# - payload shape
# - output file expectations
# - local build assumptions

set -euo pipefail

echo "============================================================"
echo "DIAMOND RUNNER: illustrative range-run template"
echo "============================================================"

PROJECT_PATH="${1:-path/to/project}"
PIPELINE_ID="${2:-pipeline-id}"
START_VALUE="${3:-1}"
END_VALUE="${4:-3}"
PAYLOAD_KEY="${5:-count}"
BIN="${6:-./src-tauri/target/debug/diamond-runner}"

for (( i=START_VALUE; i<=END_VALUE; i++ ))
do
  printf -v PADDED "%02d" "$i"
  echo ">>> RUNNING ${PIPELINE_ID} WITH ${PAYLOAD_KEY}=${PADDED} <<<"
  "$BIN" cli run-pipeline "$PROJECT_PATH" "$PIPELINE_ID" "{\"${PAYLOAD_KEY}\": \"${PADDED}\"}"
done

echo ""
echo "[*] Illustrative range run complete."
