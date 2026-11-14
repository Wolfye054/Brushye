#!/bin/bash
# B-011 basic colour mixing run: stroke A (main colour 1, mixing off), pick colour 2 in the colour
# square (122, 935), reference dab (mixing off), turn mixing on, stroke B across A, export, then turn mixing
# off again and revert the canvas via the History palette (as run2_exp.sh).
# usage: b011_mix.sh FIFO OUT.png SHOTDIR       env: OX OY CROP XDRIVE as for run2_exp.sh
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
X="${XDRIVE:-python3 $HERE/xdrive.py}"
fifo=$1 out=$2 shots=$3
HX=1712 HY=607
pen() { python3 -I "$HERE/brush_run2.py" "$1" "$OX" "$OY" > "$fifo"; sleep "$2"; }
mixing() {  # toggle Ink > Color mixing and screenshot the Ink page
  "$HERE/csp_detail.sh" open "$shots/p.png" || exit 1
  $X click 750 474 ';;' sleep 0.8 ';;' click 926 579 ';;' sleep 0.6 ';;' shot "$shots/ink_$1.png"
  "$HERE/csp_detail.sh" close "$shots/p.png" || exit 1
}
$X move 1600 950 ';;' sleep 0.3
pen mixA 4
$X move 300 900 ';;' sleep 0.2 ';;' move 120 930 ';;' sleep 0.2 ';;' click 122 935 ';;' sleep 0.6 ';;' move 1600 950 ';;' sleep 0.3 ';;' shot "$shots/colour2.png"
pen mixref 2
mixing on
pen mixB 4
"$HERE/csp_export_png.sh" "$out" || exit 1
mixing off
UP=""; for _ in $(seq 80); do UP="$UP ;; click $((HX + 60)) $((HY + 150)) 4"; done
# shellcheck disable=SC2086
$X move 1600 950 ';;' sleep 0.2 ';;' move $((HX + 60)) $((HY + 150)) $UP ';;' sleep 0.4 \
   ';;' move $((HX + 5)) "$HY" ';;' sleep 0.2 ';;' click "$HX" "$HY" ';;' sleep 1.2
echo "done mix -> $out"
