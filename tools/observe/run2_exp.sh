#!/bin/bash
# Run one brush_run2.py experiment on the pen display, export it, and revert the document.
# The canvas must be clean (= CLEAN reference) before and is reverted after via the History
# palette's top entry (History tab active, first entry at HIST_X, HIST_Y; the list is scrolled
# to the top with the wheel first).
# usage: run2_exp.sh FIFO OUT.png CLEAN.png SHOTDIR EXPERIMENT [ARGS...]
#   env: OX OY = screen position of canvas (0,0); CROP = "x0,y0,x1,y1" screen box of the canvas
#        HIST_X HIST_Y (default 1712 607); XDRIVE as for csp_export_png.sh
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
X="${XDRIVE:-python3 $HERE/xdrive.py}"
fifo=$1 out=$2 clean=$3 shots=$4; shift 4
HX=${HIST_X:-1712} HY=${HIST_Y:-607}

canvas_clean() {
  $X shot "$shots/rc.png"
  python3 -I -c "
from PIL import Image, ImageChops; import sys
box=tuple(int(v) for v in sys.argv[3].split(','))
a=Image.open(sys.argv[1]).convert('RGB').crop(box); b=Image.open(sys.argv[2]).convert('RGB')
sys.exit(0 if ImageChops.difference(a,b).getbbox() is None else 1)" "$shots/rc.png" "$clean" "$CROP"
}

# park the mouse off the canvas and let tooltips / size previews disappear
$X move 1600 950 ';;' sleep 0.3 ';;' move "$HX" "$HY" ';;' sleep 1.2
canvas_clean || { echo "CANVAS NOT CLEAN before $*"; exit 1; }
script=$(mktemp)
exp=$1; shift
python3 -I "$HERE/brush_run2.py" "$exp" "$OX" "$OY" "$@" > "$script" || exit 1
# time the pen needs: all waits plus ~1.5 ms per event
secs=$(awk '/^wait/ {w += $2} {n++} END {printf "%.1f", w / 1000 + n * 0.0015 + 1.5}' "$script")
cat "$script" > "$fifo"
rm -f "$script"
sleep "$secs"
"$HERE/csp_export_png.sh" "$out" || exit 1
# the History list scrolls down as steps are added: scroll it to the top first
UP=""; for _ in $(seq 80); do UP="$UP ;; click $((HX + 60)) $((HY + 150)) 4"; done
for _ in 1 2 3; do
  # shellcheck disable=SC2086
  $X move 1600 950 ';;' sleep 0.2 ';;' move $((HX + 60)) $((HY + 150)) $UP ';;' sleep 0.4 \
     ';;' move $((HX + 5)) "$HY" ';;' sleep 0.2 ';;' click "$HX" "$HY" ';;' sleep 1.2
  canvas_clean && { echo "done $exp -> $out"; exit 0; }
done
echo "CANVAS NOT CLEAN after $exp"; exit 1
