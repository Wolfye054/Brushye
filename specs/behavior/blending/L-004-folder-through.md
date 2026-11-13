# L-004 Layer folders: Normal vs Through (pass-through), folder opacity
- Status: **observed**
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK
- Date / method / observer: 2026-11-13 / automated-Wine (Xvfb :77) / Claude (background run)

## Setup (exact)
1. File > Open `data/inputs/top.png`, Select All, Copy, Deselect; File > Open `data/inputs/base.png`;
   Paste → "top Copy" over "base".
2. Set "top Copy" to Multiply (Layer palette dropdown).
3. Layer > Create folder and insert layer (Ctrl+G) → "Folder 1" containing "top Copy".
   A new folder's blending mode is **Normal**.
4. With the folder selected, export at: folder Normal 100 %; Through 100 %; Through 50 %; Normal 50 %.
   For a folder the blend dropdown has **Through** as an extra first entry above Normal.

## Observations (2026-11-13)
Data: `data/observed2/folder_{normal,through}_op{100,50}.png`. All four are **bit-exact**
against predictions built from L-001 data and the L-003 opacity model:
- **Normal folder** composites its children in isolation (onto transparent), then blends the
  result. A Multiply child over the empty folder is just the child, so the output equals `top`.
- **Through** blends the children straight into the backdrop. The output equals L-001's
  Multiply(base, top).
- **Folder opacity** is a lerp of the folder's composited result with the backdrop, using
  `o8 = floor(op·255/100)` and the 15-bit lerp (L-003), **for Through too**. Through at 50 % =
  lerp(base, Multiply(base, top), o8), *not* Multiply with a partially transparent source (which
  follows a different, still-open rule, see L-003).

## Open questions
- Several children (e.g. Screen over Multiply) inside a Through folder, and child opacity inside
  folders: not yet run.
- Nested folders.
