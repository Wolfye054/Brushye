#!/bin/bash
# Export one PNG per (mode, layer opacity) run of the selected top layer (default workspace,
# 1920x1080 Wine desktop; see docs/observation/wine-harness.md).
# usage: blend_export_runs.sh OUT_DIR NAME:MODE_INDEX:OPACITY [...]
#   MODE_INDEX = row in the layer blend dropdown (0 Normal … 27 Brightness, see L-001 order)
#   OPACITY    = 1..100 typed into the Layer palette opacity field
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
X="${XDRIVE:-python3 $HERE/xdrive.py}"
OUT=$1; shift
for run in "$@"; do
  IFS=: read -r name idx op <<<"$run"
  f="$OUT/$name.png"
  $X move 1600 950 ';;' sleep 0.2 ';;' click 1765 605 ';;' sleep 0.8 ';;' click 1760 $((71+19*idx)) ';;' sleep 0.6 \
     ';;' dclick 1897 605 ';;' sleep 0.3 ';;' key ctrl+a ';;' type "$op" ';;' key Return ';;' sleep 0.6
  XDRIVE="$X" "$HERE/csp_export_png.sh" "$f" >/dev/null || { echo "FAIL $name"; exit 1; }
  echo "ok $name"
done
