# Brushye command format (text form) — specification v1

This is the exact grammar for DECISIONS D-011. The parser and printer in `brushye-command`
implement this document; any change here is a format change (bump the journal format version).

## 1. Files and lines

- UTF-8, one command per line. The printer writes `\n`; the parser also accepts `\r\n`.
- A **journal** (`.bcmd` written by Brushye) starts with a header line:
  `#brushye-journal v<format> app=<app-version>`; format version is `1`.
- A **script** (hand-written `.bcmd`) may omit the header and the `@<seq>` prefixes.
- Lines that are empty or contain only spaces/tabs are ignored.
- A line whose first non-space character is `#` is a comment (the header is the first line only).
  Comments are not allowed at the end of a command line.

## 2. Command lines

```
line     = [ seq-prefix SP ] command
         | query
seq-prefix = "@" uint [ "+" ]          ; "+" = joins the previous undo step (D-013)
command  = name [ "/" uint ] *( SP arg )
query    = "?" name [ "/" uint ] *( SP arg )
name     = word *( "." word )           ; e.g. layer.set, history.undo
word     = %x61-7A *( %x61-7A / DIGIT / "_" )   ; [a-z][a-z0-9_]*
arg      = value                        ; positional
         | key "=" value
key      = word *( "." word2 )          ; e.g. opacity, brush.size
word2    = 1*( %x61-7A / DIGIT / "_" )
uint     = "0" / ( %x31-39 *DIGIT )     ; no leading zeros
SP       = 1*( " " / %x09 )             ; parser: spaces or tabs; printer: one space
```

- In journals, `seq` starts at 1 and increases by exactly 1 per line. Scripts have no `seq`.
- A missing `/<version>` means version 1. The printer always writes the version when it is
  greater than 1, and never writes `/1`.
- All positional arguments come before all `key=value` arguments. Each key appears at most once.
- The command's typed parameters (registry) decide how many positionals there are and which
  parameter each one fills (declared order). The printer writes a parameter positionally if the
  command declares it positional, otherwise as `key=value`, in declared order. Optional
  parameters equal to their default are omitted.

## 3. Values

| Kind | Syntax | Notes |
|---|---|---|
| integer | `-?(0|[1-9][0-9]*)` | i64 range |
| decimal | `-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?` with a `.` or an exponent | f32/f64 per parameter; NaN and ±∞ are not representable |
| boolean | `true` / `false` | |
| colour | `#` 6 or 8 hex digits | `rrggbb` or `rrggbbaa`, straight alpha; printed lowercase |
| id | `[A-Z][0-9]+` (no leading zeros) | object ids, e.g. `L2` (layer); the letter is the object kind |
| reference | `@` word | e.g. `@current` |
| enum | word | bare lowercase word that isn't `true`/`false`, e.g. `multiply` |
| string | `"` … `"` | escapes below |
| blob | `b64:` base64 | standard alphabet with `=` padding; the empty blob is `b64:` |
| matrix | `[` row *( `;` row ) `]`, row = number *( `,` number ) | numbers only (integer/decimal tokens), e.g. readable stroke samples `[x,y,p,t;…]`; no spaces inside; `[]` is the empty matrix |

**Strings:** `\"` `\\` `\n` `\t` `\r` and `\u{X…}` (1–6 hex digits, a Unicode scalar value).
Other characters below U+0020 and U+007F must be written as `\u{…}`. The printer uses `\n`, `\t`,
`\r` where possible and `\u{…}` (lowercase hex, no leading zeros) for the rest.

**Numbers in the printer (canonical form):**
- integers: decimal, no `+`, no leading zeros, `-0` is written `0`;
- decimals: Rust's shortest round-trip representation (`{:?}` for the declared type), which always
  contains a `.` or an `e` (e.g. `1.0`, `0.8`, `12.5`, `1e-7`, `1e21`). `-0.0` is written `0.0`.
- The parser accepts an integer token where a decimal is declared (hand-written scripts);
  printing it back gives the canonical decimal form.

## 4. Round-trip rule

For every command value `c`: `parse(print(c)) == c` and `print(parse(print(c))) == print(c)`
(byte-identical). This is property-tested over generated commands.

## 5. JSON form (MCP, scripting hosts)

Each line maps one-to-one to:
```json
{"seq": 3, "merge": false, "cmd": "layer.set", "v": 1, "args": {"layer": "L2", "opacity": 0.8}}
```
- `seq` and `merge` are present only for journal lines; queries use `"query": "layer.list"` instead
  of `"cmd"`.
- Positional parameters appear under their parameter names. Ids, references, enums and colours
  are JSON strings in their text syntax (`"L2"`, `"@current"`, `"multiply"`, `"#ff8800"`); blobs
  are `"b64:…"` strings; matrices are arrays of arrays of numbers.

## 6. Errors

The parser reports the line number, column and a message. Unknown commands, unknown keys,
duplicate keys, positionals after a key, wrong value kinds, out-of-range values and malformed
tokens are errors. Nothing is ignored silently.

## 7. Stroke samples (`samples=` of the stroke command)

A stroke records the raw input stream before any stabilization (D-011). Two forms:

**Blob (canonical, written by Brushye):** `b64:` of
```
u8  format version (1)
u8  field mask: bit0 pressure, bit1 tilt (x and y), bit2 twist, bit3 flags
varint  sample count n
then n samples, each field as a zigzag-LEB128 varint of the delta to the previous sample
(the first sample's delta is from 0), in this order:
  x, y        canvas position in 1/256 px (i32)
  t           time since the stroke's first sample in µs (u32; first sample 0, never decreasing)
  pressure    0..=65535 (u16)                  if bit0
  tilt_x, tilt_y  1/100 degree, -9000..=9000   if bit1
  twist       1/100 degree, 0..36000           if bit2
  flags       u8: bit0 eraser end, bit1 barrel button 1, bit2 barrel button 2   if bit3
```
Zigzag: `(v << 1) ^ (v >> 63)` on i64. LEB128: 7 bits per byte, low bits first, high bit = more.
Values are quantized when a stroke is created; the quantized values are what the brush engine
uses, live and on replay, so both give identical pixels.

**Readable (scripts and tests):** a matrix with one row per sample:
`[x,y,p,t;…]` or `[x,y,p,t,tilt_x,tilt_y,twist;…]` — x, y in canvas px, p in 0..1, t in µs,
tilt and twist in degrees. It is quantized exactly like the blob. Brushye always writes the blob.
