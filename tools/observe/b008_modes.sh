#!/bin/bash
# B-008: one export per brush ink blending mode (Sub Tool Detail > Ink > Blending mode).
# Layout (2026-11-13 harness, 1920x1080 Wine desktop, pen display :78):
#   document = make_b008_input.py base image (256x256), canvas (0,0) at screen (821, 435), 100 %;
#   sub tool "Obs pen": size 400, AA None, hardness 5, density 100, colour (64,128,192),
#   opacity set beforehand in Tool property; Sub Tool Detail palette left on its Ink page.
# Per mode: open Sub Tool Detail (toggle 256,504), pick the mode in the dropdown (1268,544;
# 28 items visible, one item per wheel click; picking closes the popup), reopen it to crop the
# mode label into SHOTDIR/md_<i>.png, close it with the toggle, one dab at canvas
# (128.5, 128.5), export, undo, check the canvas is back to the clean reference.
# usage: b008_modes.sh FIFO OUTDIR OPACITY CLEAN_CANVAS.png SHOTDIR IDX...
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
X="${XDRIVE:-python3 $HERE/xdrive.py}"
fifo=$1 out=$2 op=$3 clean=$4 shots=$5; shift 5
NAMES=(normal darken multiply color_burn linear_burn black_burn subtract lighten screen
  color_dodge glow_dodge white_burn add add_glow overlay soft_light hard_light difference
  erase background replace_alpha compare_density erase_compare vivid_light linear_light
  pin_light hard_mix exclusion darker_color lighter_color divide hue saturation color brightness)

px() { # px SHOT X Y -> "r,g,b"
  python3 -I -c "from PIL import Image; import sys; p=Image.open(sys.argv[1]).convert('RGB').getpixel((int(sys.argv[2]),int(sys.argv[3]))); print(','.join(map(str,p)))" "$@"
}

palette_open() { # 0 if Sub Tool Detail is open on the Ink page
  $X shot "$shots/chk.png"; [ "$(px "$shots/chk.png" 760 474)" = "96,106,128" ]
}

open_palette() { # the first click after pen input or a popup closing can be swallowed: retry
  for _ in 1 2 3 4; do
    $X move 600 900 ';;' sleep 0.2 ';;' move 610 905 ';;' sleep 0.2 ';;' click 256 504 ';;' sleep 1.2
    palette_open && return 0
  done
  return 1
}

canvas_clean() {
  $X shot "$shots/after.png"
  python3 -I -c "
from PIL import Image, ImageChops; import sys
a=Image.open(sys.argv[1]).convert('RGB').crop((821,435,1077,691)); b=Image.open(sys.argv[2]).convert('RGB')
sys.exit(0 if ImageChops.difference(a,b).getbbox() is None else 1)" "$shots/after.png" "$clean"
}

for i in "$@"; do
  name=${NAMES[$i]}
  canvas_clean || { echo "CANVAS NOT CLEAN before $name"; exit 1; }
  f=$(printf "%s/B008_op%s_%02d_%s.png" "$out" "$op" "$i" "$name")
  open_palette || { echo "PALETTE DID NOT OPEN ($name)"; exit 1; }
  if [ "$i" -lt 28 ]; then down=0; row=$i; else down=7; row=$((i - 7)); fi
  cmd="click 1268 544 ;; sleep 0.8 ;; move 1200 300"
  for _ in $(seq 10); do cmd="$cmd ;; click 1200 300 4"; done
  for _ in $(seq $down); do cmd="$cmd ;; click 1200 300 5"; done
  cmd="$cmd ;; sleep 0.4 ;; click 1100 $((10 + 19 * row)) ;; sleep 0.8"
  # shellcheck disable=SC2086
  $X $cmd
  # picking an item closes the popup; reopen it to record the selected mode, then close it
  if palette_open; then echo "PALETTE STILL OPEN after pick ($name)"; exit 1; fi
  open_palette || { echo "PALETTE DID NOT REOPEN ($name)"; exit 1; }
  python3 -I -c "from PIL import Image; import sys; Image.open(sys.argv[1]).convert('RGB').crop((1000,536,1540,552)).save(sys.argv[2])" "$shots/chk.png" "$shots/md_$i.png"
  $X click 256 504 ';;' sleep 1
  if palette_open; then echo "PALETTE DID NOT CLOSE ($name)"; exit 1; fi
  # one dab at canvas centre
  printf '%s\n' "in" "move 949.5 563.5 0" "wait 60" "down" "move 949.5 563.5 1" "wait 60" \
    "up" "wait 60" "out" "wait 200" > "$fifo"
  sleep 1.5
  "$HERE/csp_export_png.sh" "$f" || exit 1
  # undo via the document tab (safe to click), then verify the canvas is clean again
  $X move 420 88 ';;' sleep 0.2 ';;' click 412 88 ';;' sleep 0.3 ';;' key ctrl+z ';;' sleep 1
  canvas_clean || { echo "CANVAS NOT CLEAN after $name"; exit 1; }
  echo "done $i $name"
done
