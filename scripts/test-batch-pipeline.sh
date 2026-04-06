#!/bin/bash
# test-batch-pipeline.sh

echo "============================================================"
echo "DIAMOND RUNNER: Headless Batch Testing"
echo "============================================================"

cd "$(dirname "$0")/../src-tauri" || exit 1

echo "[*] Triggering headless compilation and execution..."
export RUSTUP_HOME="$PWD/../.rustup"
export CARGO_HOME="$PWD/../.cargo"

START_CH=${1:-1}
END_CH=${2:-1}

for (( i=START_CH; i<=END_CH; i++ ))
do
  echo ">>> PRODUCING CHAPTER $i <<<"
  cargo run -- cli run-pipeline "../Sample Projects/Neon & Nightmares" "batch-production" '{"chapter": "'$i'"}'
  
  if [ $? -ne 0 ]; then
    echo "FAILED ON CHAPTER $i. Halting batch loop."
    exit 1
  fi
done

echo ""
echo "[*] Headless Batch Run Complete!"
