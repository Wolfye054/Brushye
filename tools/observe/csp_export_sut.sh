#!/bin/bash
# Export the selected sub tool through Sub Tool palette ≡ > Export sub tool… to
# ~/brushye-samples/sut/NAME.sut (typed as a Z:\ path into the Wine file dialog), then check
# the file appeared. UI-level only. usage: csp_export_sut.sh NAME   (XDRIVE env as usual)
# Coordinates: CSP 1.13.2 default workspace, 1920x1080 Wine desktop (≡ at 56,84; Export at 130,352).
set -u
HERE=$(cd "$(dirname "$0")" && pwd); X="${XDRIVE:-python3 $HERE/xdrive.py}"
name=$1; out=~/brushye-samples/sut/$name.sut
[ -e "$out" ] && { echo "exists: $out"; exit 1; }
$X move 600 900 ';;' sleep 0.2 ';;' click 56 84 ';;' sleep 1 ';;' click 130 352 ';;' sleep 2
shot=${SHOT:-/tmp/csp_export_sut_check.png}; $X shot "$shot"
# the Wine file dialog opens at the top-left with a blue title bar: (200,16) = (50,150,250)
if [ "$(${PY:-python3} -I -c "from PIL import Image; import sys; print(Image.open(sys.argv[1]).convert('RGB').getpixel((200,16)))" "$shot")" != "(50, 150, 250)" ]; then echo "export dialog did not open"; $X key Escape; exit 1; fi
$X key ctrl+a ';;' type "Z:\\home\\geoffrey\\brushye-samples\\sut\\$name.sut" ';;' key Return ';;' sleep 2.5
for _ in 1 2 3 4 5; do [ -s "$out" ] && { sha256sum "$out"; exit 0; }; sleep 1; done
echo "export failed: $out not written"; exit 1
