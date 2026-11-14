#!/bin/bash
# Open or close CSP's Sub Tool Detail popup (toggle at 256,504) and verify the result with a
# page-independent pixel: (800,900) is the empty category-list background (63,63,63) when open.
# The first click after pen input or a popup closing can be swallowed, so it retries.
# usage: csp_detail.sh open|close SHOT_PATH      (XDRIVE env as usual)
set -u
HERE=$(cd "$(dirname "$0")" && pwd); X="${XDRIVE:-python3 $HERE/xdrive.py}"
want=$1 shot=$2
is_open() {
  $X shot "$shot"
  [ "$(python3 -I -c "from PIL import Image; import sys; print(Image.open(sys.argv[1]).convert('RGB').getpixel((800,900)))" "$shot")" = "(63, 63, 63)" ]
}
for _ in 1 2 3 4; do
  if is_open; then [ "$want" = open ] && exit 0; else [ "$want" = close ] && exit 0; fi
  $X move 600 900 ';;' sleep 0.2 ';;' move 610 905 ';;' sleep 0.2 ';;' click 256 504 ';;' sleep 1.2
done
echo "SUB TOOL DETAIL did not $want"; exit 1
