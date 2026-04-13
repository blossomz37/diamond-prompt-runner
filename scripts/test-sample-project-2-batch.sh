#!/bin/bash
# Script: test-sample-project-2-batch.sh
# Purpose: Runs the sample-project-2 flash-fiction pipeline five times with padded count values.
# Last modified: 2026-04-12

echo "============================================================"
echo "DIAMOND RUNNER: sample-project-2 flash-fiction batch test"
echo "============================================================"

cd "$(dirname "$0")/../src-tauri" || exit 1

echo "[*] Triggering headless compilation and execution..."
export RUSTUP_HOME="$PWD/../.rustup"
export CARGO_HOME="$PWD/../.cargo"

if [ -f "../.env" ]; then
  set -a
  . "../.env"
  set +a
fi

START_COUNT=${1:-1}
END_COUNT=${2:-5}

cargo build >/dev/null || exit 1

for (( i=START_COUNT; i<=END_COUNT; i++ ))
do
  printf -v COUNT_PADDED "%02d" "$i"
  echo ">>> PRODUCING FLASH FICTION $COUNT_PADDED <<<"
  ./target/debug/diamond-runner cli run-pipeline "../docs/sample-projects/sample-project-2" "flash-fiction-batch" "{\"count\": \"$COUNT_PADDED\"}"

  if [ $? -ne 0 ]; then
    echo "FAILED ON LOOP $COUNT_PADDED. Halting batch loop."
    exit 1
  fi
done

echo ""
echo "[*] Headless Batch Run Complete!"
