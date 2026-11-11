#!/bin/bash
# Export one PNG per layer blending mode from CSP 1.13.2 running in the observation display.
# Preconditions (see docs/observation/wine-harness.md): CSP in a 1920x1080 Wine virtual desktop
# on $DISPLAY, default workspace layout, a 2-layer document whose top layer is selected,
# and the PNG export dialog already set once to RGB color / preview off.
# usage: blend_export_loop.sh OUT_DIR PREFIX [first_index] [last_index]
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
X="${XDRIVE:-python3 $HERE/xdrive.py}"
OUT=$1; P=$2; A=${3:-0}; B=${4:-27}
MODES=(normal darken multiply color_burn linear_burn subtract lighten screen color_dodge glow_dodge add add_glow overlay soft_light hard_light difference vivid_light linear_light pin_light hard_mix exclusion darker_color lighter_color divide hue saturation color brightness)
for i in $(seq "$A" "$B"); do
  m=${MODES[$i]}; f="$OUT/${P}_$(printf %02d "$i")_$m.png"; rm -f "$f"
  WP='Z:'$(echo "$f" | tr / '\\')
  # layer palette blend dropdown -> item i (19 px rows) -> File > Export (Single Layer) > .png
  $X click 1765 605 ';;' sleep 0.8 ';;' click 1760 $((71+19*i)) ';;' sleep 0.8 \
     ';;' click 28 41 ';;' sleep 0.6 ';;' move 104 249 ';;' sleep 0.5 ';;' move 400 249 ';;' click 398 292 ';;' sleep 1.5 \
     ';;' key ctrl+a ';;' type "$WP" ';;' key enter ';;' sleep 1.5 ';;' click 1170 282
  for _ in $(seq 10); do [ -s "$f" ] && break; sleep 1; done
  [ -s "$f" ] && echo "ok $i $m" || { echo "FAIL $i $m"; exit 1; }
done
