#!/bin/bash
# Create a new custom sub tool in the current tool group via Sub Tool palette ≡ >
# "Create custom sub tool…" (Direct draw / Pen defaults, tool icon Pen, "Background color of
# icon" forced OFF because the dialog inherits it from the selected sub tool), named NAME.
# If CSP asks "Sub tool with same name exists. Do you want to proceed?", answer OK.
# Then set Brush Size to 10 in the Sub Tool Detail palette that CSP opens right after creation,
# and verify it by comparing the Tool property Brush Size value (180,433)-(222,450) with REF10
# (a crop of the same area showing "10.0"). UI-level only (clicks, keys, screenshots).
# usage: REF10=ref.png csp_new_custom_subtool.sh NAME   (XDRIVE, PY, SHOT env as usual)
# Coordinates: CSP 1.13.2 default workspace, 1920x1080 Wine desktop.
set -u
HERE=$(cd "$(dirname "$0")" && pwd); X="${XDRIVE:-python3 $HERE/xdrive.py}"; PY=${PY:-python3}
name=$1; shot=${SHOT:-/tmp/csp_new_subtool_check.png}
px() { $PY -I -c "from PIL import Image; import sys; print(Image.open(sys.argv[1]).convert('RGB').getpixel((int(sys.argv[2]),int(sys.argv[3]))))" "$shot" "$1" "$2"; }
$X move 600 900 ';;' sleep 0.2 ';;' move 60 90 ';;' sleep 0.2 ';;' click 56 84 ';;' sleep 1 ';;' click 160 187 ';;' sleep 1.5 ';;' shot "$shot"
# dialog: title bar (955,436) = (56,56,56), OK button (1144,478) = (104,121,160)
if [ "$(px 955 436)" != "(56, 56, 56)" ] || [ "$(px 1144 478)" != "(104, 121, 160)" ]; then
  echo "create dialog did not open"; exit 1; fi
$X key ctrl+a ';;' type "$name" ';;' sleep 0.3
if [ "$(px 783 627)" = "(218, 218, 218)" ]; then $X click 783 627 ';;' sleep 0.4; fi
$X shot "$shot"; [ "$(px 783 627)" = "(218, 218, 218)" ] && { echo "icon background still on"; exit 1; }
$X click 1144 478 ';;' sleep 1.5 ';;' shot "$shot"
# same-name confirmation: title bar at (900,335) = (56,56,56) and OK at (990,408)
if [ "$(px 900 335)" = "(56, 56, 56)" ] && [ "$(px 990 408)" = "(104, 121, 160)" ]; then
  $X click 995 408 ';;' sleep 1.5; fi
$X sleep 1.5 ';;' shot "$shot"
# Sub Tool Detail open? (800,900) is the empty category-list background (63,63,63) when open
if [ "$(px 800 900)" != "(63, 63, 63)" ]; then
  for _ in 1 2 3; do
    $X move 600 900 ';;' sleep 0.2 ';;' move 240 590 ';;' sleep 0.3 ';;' click 244 596 ';;' sleep 2 ';;' shot "$shot"
    [ "$(px 800 900)" = "(63, 63, 63)" ] && break
  done
  [ "$(px 800 900)" = "(63, 63, 63)" ] || { echo "Sub Tool Detail not open"; exit 1; }
fi
for _ in 1 2 3; do
  $X click 765 455 ';;' sleep 1 ';;' dclick 1512 510 ';;' sleep 0.4 ';;' key ctrl+a ';;' type 10 \
     ';;' key Return ';;' sleep 1 ';;' shot "$shot"
  if $PY -I -c "
from PIL import Image, ImageChops; import sys
a=Image.open(sys.argv[1]).convert('RGB').crop((180,433,222,450)); b=Image.open(sys.argv[2]).convert('RGB')
sys.exit(0 if ImageChops.difference(a,b).getbbox() is None else 1)" "$shot" "$REF10"; then exit 0; fi
done
echo "brush size 10 not confirmed"; exit 1
