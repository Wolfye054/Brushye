# .sut (CSP 1.13.2 sub tool export)

- Status: **draft**: container and the alpha brush settings mapped; open items below.
- Samples: `specs/formats/samples.md` (S-01..S-62, automated-Wine, D-020).
- Method: black-box differential analysis with `tools/format-study/` (`dump.py`,
  `sut_sweep.py`). Each sample is diffed against S-01; a finding cites the samples that show it.
  Nothing else was used.

## Container
- **SQLite 3 database** (magic `SQLite format 3\0`, all samples). Tables: `Manager` (1 row),
  `Node` (1 row per sub tool), `Variant` (2 rows per sub tool), `sqlite_sequence`, and
  `MaterialFile` when the sub tool uses image materials (S-55, S-59..S-62).
- Settings are **named columns** of `Variant` (185 columns in S-01), plain SQLite integers,
  reals or blobs. Unknown columns are kept by the importer and reported, never an error.

## Identity and volatile fields
- `Node.NodeVariantID` names the **current** settings row of `Variant`.
  `Node.NodeInitVariantID` names a second row. In every sample that row equals the sub tool's
  settings at creation; it is probably what "Reset to default" restores (to be confirmed).
- Between two exports of one sub tool only `Manager.SavedCount` changes (S-01 vs S-02).
  Between sub tools, `Manager.RootUuid` = `Node.NodeUuid` (16-byte UUID),
  `Manager.MaxVariantID`, and the two variant ids change too (S-25, S-29 and S-33 have the same
  settings as S-01 and differ only in these).
- `Node.NodeName` is the sub tool name, as text (S-03).
- `Node.NodeIconColor`: 0 = no icon background; S-04 has 20145817 (`0x0133_6699`, the colour
  #336699 that was picked, with a leading 1 byte, likely an "on" flag).
- S-01: `Manager.ToolType` 0, `Manager.Version` 138; `Node.NodeInputOp` and `NodeOutputOp`
  11 (Pen / Direct draw); `Node.NodeIcon` 101. Other tool kinds aren't sampled yet.

## Brush settings (`Variant`, current row)
| Setting (Sub Tool Detail) | Column | Encoding | Samples |
|---|---|---|---|
| Brush size | `BrushSize` | real, px when `BrushSizeUnit` = 0 | S-10..S-14 (1, 2.5, 50, 500, 2000) |
| Opacity | `Opacity` | integer % (0 typed → 1 stored) | S-15..S-17 |
| Anti-aliasing | `AntiAlias` | 0 none, 1 weak, 2 middle, 3 strong | S-18..S-20, S-01 |
| Hardness | `BrushHardness` | integer **1..100** (see open items) | S-22..S-24, S-25 = 100 |
| Thickness | `BrushThickness` | integer % | S-26, S-27 |
| Thickness direction | `BrushVerticalThicknes` | 0 horizontal, 1 vertical | S-28 |
| Angle | `BrushRotation` | real, degrees | S-30, S-31 |
| Gap | `BrushAutoIntervalType` | 3 narrow, 2 normal, 1 wide, 0 fixed | S-32..S-35 |
| Fixed gap value | `BrushInterval` | real (10 in S-01; 7 in S-35) | S-35 |
| Brush density | `BrushFlow` | integer % | S-36 |
| Ink blending mode | `CompositeMode` | 0 normal, 2 multiply, 29 background, 31 replace alpha, 34 compare density | S-44..S-47 |
| Stabilization | `FlickerReduction` | integer (6 in S-01) | S-51, S-52 |
| Adjust stabilization by speed | `FlickerReductionBySpeed` | 0/1 (on in S-01) | S-53 |

## Dynamics ("effector" blobs)
`BrushSizeEffector`, `BrushOpacityEffector`, `BrushFlowEffector` (density),
`BrushThicknessEffector` and others are big-endian blobs:
- A 44-byte header of eleven u32 words:
  - w0 = 44 (header length);
  - w1 = 0xF0 in every sample (unknown);
  - w2 = **input mask**: 0x10 pen pressure, 0x20 tilt, 0x40 velocity, 0x80 random (S-37,
    S-41, S-42, S-43; S-41 is 0x50 because pressure was already on);
  - w3 = **pen pressure minimum**, % (S-37: 20);
  - w4 = 100 in every sample (unknown);
  - w5..w7 = 0 (other inputs' minimums are likely here; not sampled);
  - w8 = byte length of the **pressure curve** block, 0 when absent;
  - w9 = byte length of the **tilt curve** block (S-43);
  - w10 = 500 in every sample (unknown).
- The curve blocks follow, in the order pressure, tilt:
  - u32 12, u32 point count, u32 16, then the points as (x, y) f64 pairs, endpoints included;
  - linear = 2 points (0,0), (1,1) (S-01);
  - S-38's S-curve: (0,0), (0.2566, 0.1043), (0.75, 0.8957), (1,1);
  - the default tilt curve that CSP set in S-43: (0,1), (0.5,1), (0.8,0), (1,0).
- A blob with mask 0 and no curve is 44 bytes (S-01 opacity and density). In S-01, size has
  pressure on with minimum 0; that is a new custom sub tool's default.
- `BrushRotationEffector` is an **integer** (3 in S-01), not a blob. Its meaning is unknown.

## Present but not mapped to Brushye (yet)
| Feature | Columns seen | Samples |
|---|---|---|
| Colour mixing | `BrushUseWaterColor`, `BrushUseWaterColor2`, `BrushWaterColor` (0 blend, 1 running colour, 2 smear), `BrushMixColor`, `BrushMixAlpha`, `BrushMixColorExtension` | S-48..S-50 |
| Start and end | `BrushInOutTarget` (264-byte blob), `BrushInLength`, `BrushOutLength`, … | S-54 |
| Texture | `TextureImage` (blob that references a `MaterialFile` row), `TextureScale2`, `TextureDensity` | S-55 |
| Spray | `BrushUseSpray`, `BrushSpraySize`, `BrushSprayDensity`, `BrushSprayBias` | S-56 |
| Watercolor edge | `BrushUseWaterEdge`, `BrushWaterEdgeRadius`, `BrushWaterEdgeAlphaPower`, `BrushWaterEdgeValuePower`, `BrushWaterEdgeAfterDrag`, `BrushWaterEdgeBlur` | S-57 |
| Colour jitter | `BrushChangePatternColor`, `BrushHueChange`, `BrushSaturationChange`, `BrushValueChange` | S-58 |
| Material tips | `BrushUsePatternImage`, `BrushPatternImageArray` (blob), `BrushPatternOrderType` (3 = random) and `BrushPatternOrderType2`; image data in `MaterialFile.FileData` (an embedded material package) | S-59, S-60 |
| Dual brush | `UseDualBrush`, `DualUsePatternImage`, `DualPatternImageArray`, … | S-61 |
| Ribbon | `BrushRibbon` | S-62 |
| Velocity, random and tilt dynamics | input mask bits 0x40, 0x80, 0x20 | S-41..S-43 |

## Open items (need more samples or a behaviour check)
1. **Hardness.** CSP stores 1..100, and the 5-step bar (B-009 levels 1..5) is a display of
   it. Which value does clicking each segment store, and how do values between segments draw?
   Brushye's engine has the 5 observed levels only.
2. **Ink modes.** Only 5 of the 35 `CompositeMode` values are sampled.
3. **Angle follows the line** ("Direction of line" in the angle's dynamics) and **density by
   gap** (`BrushAdjustFlowByInterval`?) aren't sampled.
4. **Gap.** Unit of `BrushInterval` for the fixed gap (7 = 7 % of size?), and how
   narrow/normal/wide relate to Brushye's observed gaps (B-002).
5. `BrushSizeUnit` ≠ 0 (mm, …), `BrushSizeSyncViewScale`, `BrushAtLeast1Pixel`.
6. Other tool kinds (set C: eraser, airbrush, …): `ToolType`, `NodeInputOp`/`NodeOutputOp`.
7. The meaning of the unknown effector words and of `BrushRotationEffector`.
8. Behaviour cross-check (M8 exit criterion): strokes made with imported sub tools against CSP
   strokes made with the same sub tool.
