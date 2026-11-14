#!/bin/bash
# Type a value into a CSP Tool property number field (double-click, select all, type, Enter),
# then screenshot-check nothing else. usage: csp_set_field.sh X Y VALUE   (XDRIVE env as usual)
# Fields (1.13.2 default workspace): Brush Size (212, 256), Opacity (212, 291),
# Stabilization (212, 361).
set -u
HERE=$(cd "$(dirname "$0")" && pwd); X="${XDRIVE:-python3 $HERE/xdrive.py}"
$X move 600 900 ';;' sleep 0.2 ';;' move "$(( $1 - 6 ))" "$2" ';;' sleep 0.2 ';;' dclick "$1" "$2" ';;' sleep 0.3 \
   ';;' key ctrl+a ';;' type "$3" ';;' key Return ';;' sleep 0.6 \
   ';;' move 900 500 ';;' sleep 0.4 ';;' move 1300 600 ';;' sleep 0.3 ';;' move 1712 607 ';;' sleep 0.6
# (a brush-size change leaves a size-preview circle on the canvas until the mouse crosses it)
