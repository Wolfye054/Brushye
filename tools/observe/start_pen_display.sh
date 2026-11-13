#!/bin/bash
# Start the pen-capable observation display (DECISIONS D-003, option 1):
#   virtual stylus (vpen.py) -> rootless Xorg (dummy video + evdev for that one device) -> CSP under Wine.
# Requires the maintainer's udev rule /etc/udev/rules.d/70-brushye-virtual-pen.rules and
# xorg-x11-server-Xorg, xorg-x11-drv-dummy, xorg-x11-drv-evdev.
# usage: start_pen_display.sh WORKDIR [DISPLAY_NUM]
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
WD=$1; DN=${2:-78}; mkdir -p "$WD"
python3 -I "$HERE/vpen.py" "$WD/fifo" > "$WD/vpen.log" 2>&1 &
for _ in $(seq 20); do [ -s "$WD/vpen.log" ] && break; sleep 0.25; done
NODE=$(head -1 "$WD/vpen.log"); [ -c "$NODE" ] || { echo "no pen device: $(cat "$WD/vpen.log")"; exit 1; }
sed "s|@NODE@|$NODE|" "$HERE/xorg-pen.conf.in" > "$WD/xorg.conf"
mkdir -p "$WD/empty.d"
Xorg ":$DN" -config "$WD/xorg.conf" -configdir "$WD/empty.d" -logfile "$WD/Xorg.log" \
     -nolisten tcp -noreset -novtswitch -sharevts > "$WD/xorg.out" 2>&1 &
sleep 3
grep -q 'Adding extended input device "Brushye Virtual Pen" (type: TABLET' "$WD/Xorg.log" \
  || { echo "Xorg did not add the pen; see $WD/Xorg.log"; exit 1; }
W=~/.local/share/cspenguin/wine-11.4
export LD_LIBRARY_PATH="$W/lib/freetype2-2.13.2/lib64:$W/lib/freetype2-2.13.2/lib32"
export PATH="$W/bin:$PATH" WINESERVER="$W/bin/wineserver" WINEPREFIX=~/.wine-csp-observe \
       WINEDEBUG=-all DISPLAY=":$DN" WINEDLLPATH=~/.local/share/cspenguin
wine explorer /desktop=csp,1920x1080 \
  "$WINEPREFIX/drive_c/Program Files/CELSYS/CLIP STUDIO 1.5/CLIP STUDIO PAINT/CLIPStudioPaint.exe" \
  > "$WD/csp.log" 2>&1 &
echo "pen=$NODE display=:$DN fifo=$WD/fifo"
