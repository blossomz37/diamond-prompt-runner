#!/bin/bash
# Script: test-sample-project-3-repetition-rerun.sh
# Purpose: Reruns only the repetition audit for sample-project-3 chapters 2 and 3.
# Last modified: 2026-04-12

echo "============================================================"
echo "DIAMOND RUNNER: sample-project-3 repetition audit rerun"
echo "============================================================"

cd "$(dirname "$0")/../src-tauri" || exit 1

export RUSTUP_HOME="$PWD/../.rustup"
export CARGO_HOME="$PWD/../.cargo"

if [ -f "../.env" ]; then
  set -a
  . "../.env"
  set +a
fi

BIN=./target/debug/diamond-runner

if [ ! -x "$BIN" ]; then
  cargo build >/dev/null || exit 1
fi

for chapter in 2 3
do
  echo ">>> RERUNNING REPETITION AUDIT FOR CHAPTER $chapter <<<"
  "$BIN" cli run-pipeline "../docs/sample-projects/sample-project-3" "repetition-audit-only" "{\"chapter\": \"$chapter\"}"

  if [ $? -ne 0 ]; then
    echo "FAILED ON CHAPTER $chapter."
    exit 1
  fi
done

echo ""
echo "[*] Repetition audit rerun complete!"
