#!/bin/bash
# Export the active CSP document to PNG via File > Export (Single Layer) > .png.
# Assumes the export dialog was configured once (preview off, RGB color) in this CSP session.
# usage: csp_export_png.sh OUT.png   (XDRIVE env = xdrive command; default python3 xdrive.py)
set -u
HERE=$(cd "$(dirname "$0")" && pwd); X="${XDRIVE:-python3 $HERE/xdrive.py}"
f=$1; rm -f "$f"; WP='Z:'$(echo "$f" | tr / '\\')
# mouse moves first: the first XTEST click after pen input is swallowed
$X move 1600 950 ';;' sleep 0.2 ';;' move 1610 955 ';;' sleep 0.2 ';;' click 28 41 ';;' sleep 0.8 \
   ';;' move 104 249 ';;' sleep 0.5 ';;' move 400 249 ';;' click 398 292 ';;' sleep 1.5 \
   ';;' key ctrl+a ';;' type "$WP" ';;' key enter ';;' sleep 2
# only press OK if the PNG settings dialog is really there (title bar text region is not canvas-white)
CHK=$(mktemp --suffix=.png)
$X shot "$CHK" 2>/dev/null
if python3 -I -c "
from PIL import Image; im=Image.open('$CHK').convert('RGB')
import sys; sys.exit(0 if im.getpixel((1170,282))!=(255,255,255) else 1)"; then $X click 1170 282; fi
rm -f "$CHK"
for _ in $(seq 15); do [ -s "$f" ] && { echo "exported $f"; exit 0; }; sleep 1; done
echo "EXPORT FAILED $f"; exit 1
