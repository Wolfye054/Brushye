# U-013 Colour chips, HSV/HLS wheel, Color History
- Status: **observed**
- See README. Date 2026-11-13, automated-Wine, observer Claude.

## Chips (tool column and Color Wheel; the two sets stay in sync)
- Clicking the **sub** chip makes sub the active drawing colour, and clicking **main** makes main
  active. The active chip gets a blue-grey accent frame; the colours themselves don't change.
- Clicking the **transparent** chip (checker strip) makes the transparent colour active. The
  strip gets the frame.
- Pressing **X** (default shortcut) **switches the active chip between main and sub**; the two
  colours are not swapped. Pressing it again switches back.
- No swap button was seen on the chips.

## Color Wheel: HSV / HLS switch (button at the bottom-right of the wheel)
- HSV (default): the hue ring with a saturation/value square inside. The readout reads "H … S …
  V …" and the button's tooltip offers to switch the wheel to HLS.
- HLS: the square is replaced by a **triangle** inside the ring:
  - white at the top-left corner, black at the bottom-left, the pure hue at the right;
  - the corner sits at the right regardless of the hue's position on the ring.

  The readout becomes "H … L … S …", and the button's icon and tooltip change to switching
  back to HSV. Switched back after the test.
- The colour group's ≡ menu has the same choice (HSV color space / HLS color space, U-012).

## Color History
- A row of small swatches, newest first; the current drawing colour is framed in red.
- **Picking a colour (from Color Set) did not add it to the history.** Drawing a stroke with it
  did, and the colour went to the front. The test stroke was undone.

## What Brushye should match
- Main / sub / transparent chips in both places, kept in sync, with the active one framed;
  clicking selects.
- An X shortcut that switches the active chip between main and sub.
- The HLS triangle mode, switched from the wheel's button and the palette menu, with the
  readout following the mode.
- Color History records a colour when it's used to draw, newest first, with the current colour
  marked.
