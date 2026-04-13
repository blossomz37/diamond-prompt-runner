#!/bin/bash
# Script: test-sample-project-3-first3.sh
# Purpose: Runs sample-project-3 chapter loop for chapters 1 through 3.
# Last modified: 2026-04-12

echo "============================================================"
echo "DIAMOND RUNNER: sample-project-3 first-three chapter test"
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

START_CHAPTER=${1:-1}
END_CHAPTER=${2:-3}
BIN=./target/debug/diamond-runner

if [ ! -x "$BIN" ]; then
  cargo build >/dev/null || exit 1
fi

for (( i=START_CHAPTER; i<=END_CHAPTER; i++ ))
do
  echo ">>> PRODUCING CHAPTER $i <<<"
  "$BIN" cli run-pipeline "../docs/sample-projects/sample-project-3" "chapter-loop" "{\"chapter\": \"$i\"}"

  if [ $? -ne 0 ]; then
    echo "FAILED ON CHAPTER $i. Halting batch loop."
    exit 1
  fi
done

echo ""
echo "[*] Chapter loop run complete!"
