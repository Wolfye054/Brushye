# B-011 Colour mixing: Blend and Running colour
- Status: draft
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK

## Setup
Canvas 512×128 with a generated background: left half #ff0000, right half #0000ff, and a
transparent strip at the bottom. Brush size 40, colour #ffff00, density 100.

## Steps
For Blend and Running colour, with amount of paint {0, 30, 70, 100} × density of paint {100, 50}
× colour stretch {0, 50}: one scripted horizontal stroke across the boundary (fixed timing).
Export each.

## What to measure
Colour along the stroke centre: pickup rate, carry distance, how transparency is picked up.
This is the hardest brush behaviour to match, so expect several rounds.
