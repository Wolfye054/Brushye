# H-002 History steps for folders, clipping and layer masks
- Status: **observed**
- CSP version: 1.13.2 (PRO, 64-bit, build 202301210208)
- Needs: automated-Wine OK
- Date / method / observer: 2026-11-14 / automated-Wine (Xvfb :77, XTEST) / Claude
- Screenshots: `obs-out/m10/h002_*.png` on the observation machine (not committed).

## Setup (exact)
The same session as U-014: an 800×800 canvas with no paper, and one stroke on Layer 1. After
groups of actions, the History tab (Layer palette group, tab 2) was opened and read.

## Observations
Each action below adds **one** History entry, with the label shown, unless marked otherwise:

| Action | History entry |
|---|---|
| Create folder and insert layer (Ctrl+G) | **Change order of layer** |
| Folder blending mode → Through | Change layer blending mode |
| Collapse or expand a folder (disclosure arrow) | **none**: not a History step |
| New Raster Layer (inside a folder) | New Raster Layer |
| Clip to Layer Below on | **Switching clipping** |
| Create layer mask (palette button, or Mask Outside Selection) | Create layer mask |
| Eraser stroke on the mask | Eraser/Eraser/Eraser (the tool path, as for any stroke, H-001) |
| Show Mask Area on | **Switch showing mask area** |
| Enable Mask off, then on | **Switch enabling mask** (one entry each time) |
| Link Mask to Layer off, then on | **Switch link for mask** (one entry each time) |
| Apply Mask to Layer | Apply layer mask |
| Ungroup Layer Folder | Ungroup Layer Folder |
| New Layer Folder (menu) | New Layer Folder |
| Delete Layer (a folder with contents) | Delete Layer |

- Unlike layer visibility (H-001: not a History step), **Show Mask Area, Enable Mask and Link
  are History steps**.
- **Same-kind changes in a row merge** (H-001):
  - Enable Mask off then on, with nothing in between, gave **one** "Switch enabling mask"
    entry (L-008 run 1).
  - Blend Multiply then Normal gave one "Change layer blending mode".
  - Opacity 50 then 100 gave one "Change layer opacity".
  - With other actions in between (Show Mask Area, Link), each toggle is its own entry.
- Clicking an entry in the History palette moves to that state. A new action then discards the
  later entries (linear history, H-001). This was observed by accident during the run.

## Open questions
- Consecutive clip or Show Mask Area toggles: expected to merge like mask enable (not run).
- Delete Mask, and layer moves into or out of folders by drag: their labels.
