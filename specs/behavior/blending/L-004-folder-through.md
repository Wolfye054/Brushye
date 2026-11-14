# L-004 Layer folders: Normal vs Through (pass-through), folder opacity
- Status: **observed** (run 2, 2026-11-14: several children, child opacity, folder masks)
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

## Run 2 (2026-11-14): several children, child opacity, folder masks
Method: automated-Wine (Xvfb :77), the same harness. Data: `data/observed3/folder_*.png`.
Checks: `tools/observe/l004b_check.py`. Predictions are rendered by Brushye's CPU compositor
(`brushye run`).

Setup:
1. base.png (opaque).
2. Folder 1 holding `top Copy` (top.png, opaque, **Multiply**) with `top_alpha Copy`
   (top_alpha.png, **Screen**) above it.
3. The folder was built with Ctrl+G on top Copy, then a paste with top Copy selected.
4. The folder mask uses the L-008 eraser strokes (hard 30 at y 40/330, soft 60 at y 150/420),
   so its M equals `mask2_readout_alone`'s alpha.

All results are **bit-exact**, 131 072 px:
- `folder_through`: the children go straight onto the backdrop in order (base, then Multiply
  top, then Screen top_alpha). It is identical to the flat stack.
- `folder_normal`: the children composite in isolation onto transparent (the first child is
  copied, L-005), and the result composites Normal onto base.
- `folder_through_child_op50`: child opacity inside Through works as on a top-level layer.
- `folder_normal_mask`: the folder's isolated result gets alpha `floor(a·M/255)`, then the
  usual Normal composite. This is the L-008 layer rule; the result here is opaque, so
  `lerp(base, result, a15(M))`.
- `folder_through_mask`: `lerp15(backdrop, through_result, w)` with
  **`w = floor(M·32768/255)`**. This is not the layers' rounded a15(M), which is off by 1 on
  43 px.
  - Applying the mask to each child instead is wrong (39 728 px off).
  - Like folder opacity (run 1), a Through folder's mask blends the backdrop toward the
    folder's result.

## Open questions
- Nested folders (rendering). The palette nests (U-014); compositing is expected to follow
  recursively. Not run.
- Through folder opacity and mask together: their order.
