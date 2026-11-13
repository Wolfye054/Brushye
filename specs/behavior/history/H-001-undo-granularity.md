# H-001 Undo granularity: which actions create history entries, and the undo limit
- Status: **observed** (2026-11-13)
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK (virtual stylus for the stroke steps)
- Date / method / observer: 2026-11-13 / automated-Wine + virtual stylus / Claude

## Purpose
Decide what is an undo step in Brushye (DECISIONS D-013) by observing CSP: which actions add
an entry to the History palette, how continuous edits (slider drags, strokes) are grouped, what
happens to redo after a new action, and the default/range of the undo limit.

## Setup (exact)
1. File > New: 800×600 px, 72 dpi, Color, white paper. One raster layer ("Layer 1") is created.
2. Show the History palette (right dock, tab next to Layer). Note its entries (expect one
   starting entry for the new document).
3. Tool: the "Obs pen" sub tool (B-004 setup), size 20.

## Steps (exact)
After **each** step, screenshot the History palette and record the entry count and whether
the newest entry is highlighted (current state).
1. Pen stroke with the virtual stylus (one press, ~60 events).
2. Layer > New Raster Layer (via the Layer palette's new-layer button).
3. Click "Layer 1" in the Layer palette (change the selected layer).
4. Toggle Layer 1's visibility off, then on (two clicks).
5. Set Layer 1's opacity by typing 50 + Enter in the Layer palette opacity field.
6. Drag the opacity slider across several values in one drag (mouse down, 20 moves, up).
7. Change Layer 1's blending mode to Multiply.
8. Rename Layer 1 (double-click name, type "Renamed", Enter).
9. Change the brush size in Tool Property (type 40 + Enter).
10. Change the main colour (click a point in the colour wheel's square).
11. Zoom in/out (Navigator +/−), and rotate the view one step.
12. Rectangle selection drag on the canvas; then Select > Deselect.
13. Lock Layer 1 (lock button), then unlock.
14. Undo ×2 (Ctrl+Z): note which entries become inactive. Then draw a new stroke: are the
    undone entries discarded (redo no longer possible)?
15. File > Preferences > Performance: record the undo settings (count / memory) and their
    ranges.

## What to record
Per step: entries added (0, 1, n), grouping of continuous edits (step 6), undo/redo behaviour
(step 14), and the preference values (step 15). Entry labels are paraphrased in the spec
(no CSP UI text is copied).

## Observations (2026-11-13)
The History palette was captured after every step (screenshots kept locally, not in the repo).
Entry labels are paraphrased. Canvas "undo" 800×600; the palette starts with one entry for
the new document.

| Step | Action | New entries |
|---|---|---|
| 1 | Pen stroke (virtual stylus) | **1** |
| 2 | New raster layer | **1** |
| 3 | Select another layer | 0 |
| 4 | Visibility off, then on (verified: the layer's pixels vanish and return) | 0 |
| 5 | Opacity typed (50) | **1** |
| 6a | Opacity slider drag right after step 5 (50→93) | 0, merged into step 5's entry |
| 6b | Opacity slider drag after an unrelated entry (rename) | **1** for the whole drag (not one per move) |
| 6c | Opacity typed (40) right after 6b | 0, merged into 6b's entry |
| 7 | Blending mode → Multiply | **1** |
| 8 | Rename layer | **1** |
| 9 | Brush size typed in Tool Property | 0 |
| 10 | Main colour changed (colour wheel) | 0 |
| 11 | Zoom in/out and rotate view | 0 |
| 12 | Rectangle selection; Deselect | **1** each |
| 13 | Lock layer; then unlock (verified unlocked) | **1** for lock; unlock merged into it |
| 16 | Two strokes 0.5 s apart, undo interval 0 ms | **2** |
| 16 | Two strokes 0.5 s apart, undo interval 2000 ms | **1** (merged) |

Findings:
1. **Document changes are undo steps; application/view state is not.** Strokes, new layer,
   layer properties (opacity, blend, name, lock), selection and deselection each make an entry.
   Layer selection, visibility, tool settings, colour, zoom, pan and rotation make none.
   (Visibility is the notable one: it changes the rendered document but isn't undoable.)
2. **Consecutive changes of the same kind merge into one entry.** A continuous opacity drag is
   one entry. A further opacity change (drag or typed) right after it merges into the same entry,
   as do lock → unlock. **Undo of a merged entry restores the value from before the whole run**
   (opacity went back to 93, its value before the drag, not to an intermediate value). Redo
   restores the final value.
3. **Strokes with an empty result still make an entry.** Strokes drawn entirely outside an active
   selection (clipped to nothing) were each recorded.
4. **Linear history.** Undone entries stay listed but greyed (redoable, Ctrl+Y). A new action after
   undo discards them. Undoing a Deselect restores the previous selection, so selection is part of
   the undoable document state.
5. **Undo limit and merge window** (File > Preferences > Performance > Undo):
   - "Undo levels": 200 in this prefix (a copy of the maintainer's install, so it may not be the
     factory default), range **1–200** (values typed outside the range are clamped).
   - "Undo interval between actions": 0 ms here, range **0–2000 ms**. When > 0, consecutive
     actions within the interval merge into one entry (two strokes 0.5 s apart → 1 entry at
     2000 ms, 2 entries at 0 ms).
   - Memory settings sit on the same page (virtual memory path, % allocated to the application);
     the undo limit itself is a step count, not a memory size.
6. Side note for B-001: Preferences > Tool "Reduce pen pressure when drawing with finger/mouse"
   is **off** in this prefix, which fits the full-size dabs from mouse clicks.

## Open questions
- Factory default of "Undo levels" (needs a fresh install or the manual).
- Does the same-kind merge (finding 2) depend on the target layer (opacity of layer A, then of
  layer B)? Does it also apply to blend mode or other properties?
- Merge-window rule with the interval: measured from the end of the previous action or from its start?
- Are other application settings (e.g. canvas flip, workspace changes) also outside history?
