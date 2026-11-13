#!/bin/bash
# Start the pen-capable observation display (DECISIONS D-003, option 1):
#   virtual stylus (vpen.py) -> rootless Xorg on seat-brushye (dummy video, libinput for that one device)
#   -> CSP under Wine.
# Requires the maintainer's udev rule /etc/udev/rules.d/70-brushye-virtual-pen.rules and
# xorg-x11-server-Xorg, xorg-x11-drv-dummy, xorg-x11-drv-libinput.
# usage: start_pen_display.sh WORKDIR [DISPLAY_NUM]
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
WD=$1; DN=${2:-78}; mkdir -p "$WD"
python3 -I "$HERE/vpen.py" "$WD/fifo" > "$WD/vpen.log" 2>&1 &
for _ in $(seq 20); do [ -s "$WD/vpen.log" ] && break; sleep 0.25; done
NODE=$(head -1 "$WD/vpen.log"); [ -c "$NODE" ] || { echo "no pen device: $(cat "$WD/vpen.log")"; exit 1; }
cp "$HERE/xorg-pen.conf.in" "$WD/xorg.conf"
mkdir -p "$WD/empty.d"
Xorg ":$DN" -seat seat-brushye -dumbSched -config "$WD/xorg.conf" -configdir "$WD/empty.d" -logfile "$WD/Xorg.log" \
     -nolisten tcp -noreset -novtswitch -sharevts > "$WD/xorg.out" 2>&1 &
sleep 3
grep -q 'Adding extended input device "Brushye Virtual Pen"' "$WD/Xorg.log" \
  || { echo "Xorg did not add the pen; see $WD/Xorg.log"; exit 1; }
# xf86-input-libinput creates the stylus XInput device ("Brushye Virtual Pen Pen (0)") on the
# first proximity-in, and only while a client is talking to the server. Wine enumerates tablet
# devices when wintab loads, so make the stylus exist before CSP starts.
printf 'in\nmove 960 540 0\nwait 200\nout\nwait 200\n' > "$WD/fifo"
for _ in $(seq 20); do
  DISPLAY=":$DN" xprop -root >/dev/null 2>&1   # any request lets the server run its work queue
  grep -q 'Adding extended input device "Brushye Virtual Pen Pen' "$WD/Xorg.log" && break; sleep 0.25
done
grep -q 'Adding extended input device "Brushye Virtual Pen Pen' "$WD/Xorg.log" \
  || { echo "stylus subdevice not created; see $WD/Xorg.log"; exit 1; }
W=~/.local/share/cspenguin/wine-11.4
export LD_LIBRARY_PATH="$W/lib/freetype2-2.13.2/lib64:$W/lib/freetype2-2.13.2/lib32"
export PATH="$W/bin:$PATH" WINESERVER="$W/bin/wineserver" WINEPREFIX=~/.wine-csp-observe \
       WINEDEBUG=-all DISPLAY=":$DN" WINEDLLPATH=~/.local/share/cspenguin
wine explorer /desktop=csp,1920x1080 \
  "$WINEPREFIX/drive_c/Program Files/CELSYS/CLIP STUDIO 1.5/CLIP STUDIO PAINT/CLIPStudioPaint.exe" \
  > "$WD/csp.log" 2>&1 &
echo "pen=$NODE display=:$DN fifo=$WD/fifo"
