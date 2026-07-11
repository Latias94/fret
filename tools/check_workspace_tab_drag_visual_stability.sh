#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

cargo_target_dir="${CARGO_TARGET_DIR:-target}"
fretboard_binary="$cargo_target_dir/debug/fretboard-dev"
binary="${1:-$cargo_target_dir/release/workspace_shell_demo}"
out_dir="target/fret-diag/workspace-tab-drag-visual-stability-check"
script="tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-text-during-dock-drag-stability.json"

cargo build -p fretboard-dev
if [[ $# -eq 0 ]]; then
  cargo build -p fret-demo --bin workspace_shell_demo --release
fi

"$fretboard_binary" diag run "$script" \
  --dir "$out_dir" \
  --session-auto \
  --timeout-ms 180000 \
  --check-pixels-unchanged workspace-shell-pane-pane-b-tab-strip \
  --launch -- "$binary"

echo "PASS: pane-b tabstrip pixels remained stable during the stationary dock drag"
