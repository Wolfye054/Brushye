# U-005 Colour palettes
- Status: **observed** (default tab = Color Wheel; others opened by clicking their tabs)
- See README.

## Group and tabs (left dock group 4, y 854–1079)
- Title strip y 854–870: "≡" menu, then **icon-only tabs** in this order: Color Wheel (active),
  Color Set, Color Slider, Color Mixing, Color History, Approximate Color; a ▾ overflow button at
  the far right lists all tabs (Color Wheel ✓, Color Set, Color Slider, Color Mixing,
  Color History, Approximate Color — separator — Intermediate Color). Intermediate Color is
  only reachable from that list at this dock width.
- Content background (78,78,78).

## Color Wheel (default)
- A hue **ring** ~190 px across (x 56–245, y 876–1065), ring thickness ~16 px, red at the
  left (9 o'clock), hue running yellow → green going clockwise over the top, cyan at the right,
  blue at the bottom, magenta lower-left. The current hue is a small rectangular marker on the
  ring.
- Inside the ring a **saturation/value square** ~100×100 (x ~101–202, y ~920–1021): saturation
  left → right, value top → bottom; a small circle marks the current colour.
- Bottom-left: **main/sub colour chips** (main ~20 px square top-left with a selection frame,
  sub square offset to the bottom-right) and a **transparent** chip (checker) under them.
- Bottom: numeric readout "H <h> S <s> V <v>" with small mode icons (here H 0, S 53, V 70).
- Bottom-right: a round button "Switch color wheel to HLS color space" (toggles HSV/HLS).

## Color Set
- Top: dropdown with the current set name ("Standard color set") and two small icon buttons.
- A grid of swatches (~16 px cells, 12 per row at this width, 1 px gaps), first row greys
  black → white and a transparent cell, then hue rows at several lightness levels.
- Three icon buttons at the bottom-right (not identified).

## Color Slider
- Vertical mode tabs at the left edge: **RGB** (active), **HSV**, **CMY(K)**.
- Three sliders (R, G, B) each with a gradient track showing the channel's range at the current
  colour, a small triangle marker under the track, a numeric field (0–255) and a spinner.
- Under them: three large chips: main colour (framed), sub colour, transparent.

## Color Mixing
- A small mixing canvas (checker when empty), a row of paint swatches (red, orange, yellow,
  green, teal, blue, …) and a bottom toolbar of icon buttons with the main colour chip at its
  left (buttons not identified).

## Color History
- A strip of recently used colours (newest first), small squares.

## Approximate Color
- A colour grid (~13×13 cells) around the current colour, with a **V** slider + percentage on
  top and an **S** slider + percentage down the left side; the current colour is framed.

## Intermediate Color
- Not opened (only in the overflow list at this width).

## What Brushye should match
- A colour group at the bottom of the left dock with icon tabs in the same order and an
  overflow ▾ list.
- Color Wheel as the default tab: hue ring + SV square (same hue orientation), main/sub/
  transparent chips bottom-left, H/S/V readout, HSV/HLS switch.
- Color Slider with RGB/HSV/CMY(K) vertical tabs and value fields; Color Set as a swatch grid
  with a set dropdown; Color History strip. Mixing / Approximate / Intermediate can follow.
- The same main/sub/transparent chips also at the bottom of the tool column (U-004); clicking
  a chip makes it the active drawing colour.
