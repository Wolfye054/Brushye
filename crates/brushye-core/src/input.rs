//! Raw pen/mouse input samples in canvas space, and their canonical encoding (D-011,
//! `docs/command-format.md` §7). Strokes record samples before any stabilization.

/// One input sample in canvas space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    /// Canvas pixels (sub-pixel).
    pub x: f64,
    pub y: f64,
    /// Microseconds since the stroke's first sample.
    pub t_us: u32,
    /// 0..=1; `None` for devices without pressure (mouse).
    pub pressure: Option<f32>,
    /// Degrees, -90..=90.
    pub tilt: Option<[f32; 2]>,
    /// Degrees, 0..360.
    pub twist: Option<f32>,
    pub eraser: bool,
    pub barrel: [bool; 2],
}

#[derive(Debug, PartialEq, Eq)]
pub struct SampleError(pub String);

impl std::fmt::Display for SampleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "stroke samples: {}", self.0)
    }
}

impl std::error::Error for SampleError {}

const FORMAT: u8 = 1;
const F_PRESSURE: u8 = 1;
const F_TILT: u8 = 2;
const F_TWIST: u8 = 4;
const F_FLAGS: u8 = 8;
/// Positions in 1/256 px.
const POS: f64 = 256.0;

/// Quantized sample: the exact values the brush engine uses (live and on replay).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
struct Q {
    x: i64,
    y: i64,
    t: i64,
    p: i64,
    tx: i64,
    ty: i64,
    tw: i64,
    flags: i64,
}

fn round(v: f64) -> i64 {
    // Round half away from zero; deterministic for every finite input.
    v.round() as i64
}

fn quantize(s: &Sample) -> Q {
    let flags = i64::from(s.eraser) | (i64::from(s.barrel[0]) << 1) | (i64::from(s.barrel[1]) << 2);
    Q {
        x: round(s.x * POS).clamp(i64::from(i32::MIN), i64::from(i32::MAX)),
        y: round(s.y * POS).clamp(i64::from(i32::MIN), i64::from(i32::MAX)),
        t: i64::from(s.t_us),
        p: s.pressure
            .map_or(0, |p| round(f64::from(p.clamp(0.0, 1.0)) * 65535.0)),
        tx: s
            .tilt
            .map_or(0, |t| round(f64::from(t[0].clamp(-90.0, 90.0)) * 100.0)),
        ty: s
            .tilt
            .map_or(0, |t| round(f64::from(t[1].clamp(-90.0, 90.0)) * 100.0)),
        tw: s
            .twist
            .map_or(0, |w| round(f64::from(w.rem_euclid(360.0)) * 100.0) % 36000),
        flags,
    }
}

fn dequantize(q: &Q, mask: u8) -> Sample {
    Sample {
        x: q.x as f64 / POS,
        y: q.y as f64 / POS,
        t_us: q.t as u32,
        pressure: (mask & F_PRESSURE != 0).then(|| (q.p as f64 / 65535.0) as f32),
        tilt: (mask & F_TILT != 0)
            .then(|| [(q.tx as f64 / 100.0) as f32, (q.ty as f64 / 100.0) as f32]),
        twist: (mask & F_TWIST != 0).then(|| (q.tw as f64 / 100.0) as f32),
        eraser: q.flags & 1 != 0,
        barrel: [q.flags & 2 != 0, q.flags & 4 != 0],
    }
}

fn mask_of(samples: &[Sample]) -> u8 {
    let mut m = 0;
    if samples.iter().any(|s| s.pressure.is_some()) {
        m |= F_PRESSURE;
    }
    if samples.iter().any(|s| s.tilt.is_some()) {
        m |= F_TILT;
    }
    if samples.iter().any(|s| s.twist.is_some()) {
        m |= F_TWIST;
    }
    if samples
        .iter()
        .any(|s| s.eraser || s.barrel[0] || s.barrel[1])
    {
        m |= F_FLAGS;
    }
    m
}

fn put_varint(out: &mut Vec<u8>, v: i64) {
    let mut z = ((v << 1) ^ (v >> 63)) as u64;
    loop {
        let b = (z & 0x7f) as u8;
        z >>= 7;
        if z == 0 {
            out.push(b);
            return;
        }
        out.push(b | 0x80);
    }
}

fn put_uvarint(out: &mut Vec<u8>, mut z: u64) {
    loop {
        let b = (z & 0x7f) as u8;
        z >>= 7;
        if z == 0 {
            out.push(b);
            return;
        }
        out.push(b | 0x80);
    }
}

struct Rd<'a> {
    b: &'a [u8],
    i: usize,
}

impl Rd<'_> {
    fn byte(&mut self) -> Result<u8, SampleError> {
        let v = *self
            .b
            .get(self.i)
            .ok_or_else(|| SampleError("truncated".into()))?;
        self.i += 1;
        Ok(v)
    }
    fn uvarint(&mut self) -> Result<u64, SampleError> {
        let mut z = 0u64;
        for shift in (0..64).step_by(7) {
            let b = self.byte()?;
            z |= u64::from(b & 0x7f) << shift;
            if b & 0x80 == 0 {
                return Ok(z);
            }
        }
        Err(SampleError("varint too long".into()))
    }
    fn varint(&mut self) -> Result<i64, SampleError> {
        let z = self.uvarint()?;
        Ok(((z >> 1) as i64) ^ -((z & 1) as i64))
    }
}

/// Encode samples (blob form). Samples are quantized; times must not decrease.
pub fn encode(samples: &[Sample]) -> Result<Vec<u8>, SampleError> {
    if samples.windows(2).any(|w| w[1].t_us < w[0].t_us) {
        return Err(SampleError("sample times must not decrease".into()));
    }
    let mask = mask_of(samples);
    let mut out = vec![FORMAT, mask];
    put_uvarint(&mut out, samples.len() as u64);
    let mut prev = Q::default();
    for s in samples {
        let q = quantize(s);
        for (v, p) in [(q.x, prev.x), (q.y, prev.y), (q.t, prev.t)] {
            put_varint(&mut out, v - p);
        }
        if mask & F_PRESSURE != 0 {
            put_varint(&mut out, q.p - prev.p);
        }
        if mask & F_TILT != 0 {
            put_varint(&mut out, q.tx - prev.tx);
            put_varint(&mut out, q.ty - prev.ty);
        }
        if mask & F_TWIST != 0 {
            put_varint(&mut out, q.tw - prev.tw);
        }
        if mask & F_FLAGS != 0 {
            put_varint(&mut out, q.flags - prev.flags);
        }
        prev = q;
    }
    Ok(out)
}

/// Decode a blob back to (quantized) samples, validating every field.
pub fn decode(blob: &[u8]) -> Result<Vec<Sample>, SampleError> {
    let mut r = Rd { b: blob, i: 0 };
    let fmt = r.byte()?;
    if fmt != FORMAT {
        return Err(SampleError(format!("unsupported sample format {fmt}")));
    }
    let mask = r.byte()?;
    if mask & !(F_PRESSURE | F_TILT | F_TWIST | F_FLAGS) != 0 {
        return Err(SampleError(format!("unknown field bits {mask:#x}")));
    }
    let n = r.uvarint()?;
    if n > (blob.len() as u64) {
        return Err(SampleError("sample count larger than the data".into()));
    }
    let mut q = Q::default();
    let mut out = Vec::with_capacity(n as usize);
    let add = |a: i64, d: i64| {
        a.checked_add(d)
            .ok_or_else(|| SampleError("value overflow".into()))
    };
    for _ in 0..n {
        q.x = add(q.x, r.varint()?)?;
        q.y = add(q.y, r.varint()?)?;
        q.t = add(q.t, r.varint()?)?;
        if mask & F_PRESSURE != 0 {
            q.p = add(q.p, r.varint()?)?;
        }
        if mask & F_TILT != 0 {
            q.tx = add(q.tx, r.varint()?)?;
            q.ty = add(q.ty, r.varint()?)?;
        }
        if mask & F_TWIST != 0 {
            q.tw = add(q.tw, r.varint()?)?;
        }
        if mask & F_FLAGS != 0 {
            q.flags = add(q.flags, r.varint()?)?;
        }
        let ok = (i64::from(i32::MIN)..=i64::from(i32::MAX)).contains(&q.x)
            && (i64::from(i32::MIN)..=i64::from(i32::MAX)).contains(&q.y)
            && (0..=i64::from(u32::MAX)).contains(&q.t)
            && (0..=65535).contains(&q.p)
            && (-9000..=9000).contains(&q.tx)
            && (-9000..=9000).contains(&q.ty)
            && (0..36000).contains(&q.tw)
            && (0..8).contains(&q.flags)
            && out.last().is_none_or(|s: &Sample| q.t >= i64::from(s.t_us));
        if !ok {
            return Err(SampleError(format!("sample {} out of range", out.len())));
        }
        out.push(dequantize(&q, mask));
    }
    if r.i != blob.len() {
        return Err(SampleError("trailing bytes".into()));
    }
    Ok(out)
}

/// Quantize samples to the exact values the engine uses (= decode(encode(s))).
pub fn canonical(samples: &[Sample]) -> Result<Vec<Sample>, SampleError> {
    decode(&encode(samples)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: f64, y: f64, t: u32, p: f32) -> Sample {
        Sample {
            x,
            y,
            t_us: t,
            pressure: Some(p),
            tilt: None,
            twist: None,
            eraser: false,
            barrel: [false; 2],
        }
    }

    #[test]
    fn round_trip_is_canonical_and_stable() {
        let v = vec![
            s(100.3, 200.7, 0, 0.5),
            s(101.0, 199.0, 4000, 0.51),
            s(-3.25, 1e6, 8000, 1.0),
        ];
        let c = canonical(&v).unwrap_or_else(|e| unreachable!("{e}"));
        assert_eq!(canonical(&c).unwrap_or_else(|e| unreachable!("{e}")), c);
        assert_eq!(c[0].x, 25677.0 / 256.0);
        assert_eq!(c[2].y, 1e6);
        assert!((c[1].pressure.unwrap_or(0.0) - 0.51).abs() < 1e-4);
    }

    #[test]
    fn mouse_strokes_omit_pressure_and_are_small() {
        let v: Vec<Sample> = (0..200)
            .map(|i| Sample {
                x: 10.0 + f64::from(i),
                y: 10.0,
                t_us: i * 4000,
                pressure: None,
                tilt: None,
                twist: None,
                eraser: false,
                barrel: [false; 2],
            })
            .collect();
        let b = encode(&v).unwrap_or_else(|e| unreachable!("{e}"));
        assert_eq!(b[1], 0, "no optional fields");
        // ~ 2 bytes (x delta 256) + 1 (y) + 3 (t delta 4000) per sample.
        assert!(b.len() < 200 * 7, "{} bytes", b.len());
        assert!(
            decode(&b)
                .unwrap_or_else(|e| unreachable!("{e}"))
                .iter()
                .all(|s| s.pressure.is_none())
        );
    }

    #[test]
    fn all_fields_round_trip() {
        let v = vec![Sample {
            x: 1.5,
            y: 2.5,
            t_us: 0,
            pressure: Some(0.25),
            tilt: Some([-45.5, 30.0]),
            twist: Some(359.99),
            eraser: true,
            barrel: [false, true],
        }];
        let c = canonical(&v).unwrap_or_else(|e| unreachable!("{e}"));
        assert_eq!(c[0].tilt, Some([-45.5, 30.0]));
        assert_eq!(c[0].twist, Some(359.99));
        assert!(c[0].eraser && c[0].barrel[1] && !c[0].barrel[0]);
    }

    #[test]
    fn rejects_bad_blobs() {
        assert!(decode(&[]).is_err());
        assert!(decode(&[2, 0, 0]).is_err()); // format
        assert!(decode(&[1, 0x10, 0]).is_err()); // unknown field
        assert!(decode(&[1, 0, 1, 0]).is_err()); // truncated sample
        assert!(decode(&[1, 0, 0, 7]).is_err()); // trailing byte
        let backwards = vec![s(0.0, 0.0, 10, 0.5), s(0.0, 0.0, 5, 0.5)];
        assert!(encode(&backwards).is_err());
    }
}
