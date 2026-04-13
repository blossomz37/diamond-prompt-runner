#!/bin/bash
# Script: test-sample-project-4-cascade.sh
# Purpose: Runs sample-project-4 dossier cascade over a section range.
# Last modified: 2026-04-13

echo "============================================================"
echo "DIAMOND RUNNER: sample-project-4 dossier cascade test"
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

START_SECTION=${1:-1}
END_SECTION=${2:-17}
TOTAL_CHAPTERS=${3:-20}
BIN=./target/debug/diamond-runner
DOSSIER_PATH="../docs/sample-projects/sample-project-4/documents/story-dossier.md"
BACKUP_PATH="../docs/sample-projects/sample-project-4/documents/story-dossier.backup.$(date +%Y%m%d%H%M%S).md"

SECTIONS=(
  "section_1_required_data_layer"
  "section_2_story_concept"
  "section_3_protagonist_operating_systems"
  "section_4_supporting_cast"
  "section_5_story_world"
  "section_6_narrative_physics_engine_axes_and_vectors"
  "section_7_narrative_physics_engine_thresholds_and_entropy"
  "section_8_writing_style_rules"
  "section_9_genre_lens"
  "section_10_story_summary"
  "section_11_structure_breakdown"
  "section_12_chapter_outlines_setup"
  "section_13_chapter_outlines_rising_action"
  "section_14_chapter_outlines_complications"
  "section_15_chapter_outlines_climax"
  "section_16_chapter_outlines_resolution"
  "section_17_continuity_check"
)

if [ ! -x "$BIN" ]; then
  cargo build >/dev/null || exit 1
fi

if [ "$START_SECTION" -eq 1 ]; then
  if [ -f "$DOSSIER_PATH" ] && [ -s "$DOSSIER_PATH" ]; then
    cp "$DOSSIER_PATH" "$BACKUP_PATH"
    echo "[*] Existing story-dossier.md backed up to $(basename "$BACKUP_PATH")"
  fi
  : > "$DOSSIER_PATH"
fi

for (( i=START_SECTION; i<=END_SECTION; i++ ))
do
  SECTION_KEY="${SECTIONS[$((i-1))]}"
  echo ">>> PRODUCING SECTION $i: $SECTION_KEY <<<"
  "$BIN" cli run-pipeline "../docs/sample-projects/sample-project-4" "build-dossier-section" "{\"section_key\": \"$SECTION_KEY\", \"total_chapters\": \"$TOTAL_CHAPTERS\"}"

  if [ $? -ne 0 ]; then
    echo "FAILED ON SECTION $i ($SECTION_KEY). Halting cascade."
    exit 1
  fi
done

echo ""
echo "[*] Dossier cascade run complete!"
