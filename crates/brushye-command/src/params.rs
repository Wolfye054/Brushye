//! Typed command parameters. A command's parameters are declared once with [`params!`]; the
//! text codec, the JSON form and the schema all derive from that declaration (D-011).

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use brushye_core::{BlendMode, LayerId, LayerKind, Rgba8};
use serde_json::Value as Json;

use crate::syntax::Value;

/// A value type usable as a command parameter.
pub trait Arg: Sized {
    /// Schema kind name.
    const KIND: &'static str;
    fn from_value(v: &Value) -> Result<Self, String>;
    /// Canonical text value.
    fn to_value(&self) -> Value;
    fn to_json(&self) -> Json;
    fn from_json(j: &Json) -> Result<Self, String>;
}

fn want(kind: &str, v: &Value) -> String {
    format!("expected {kind}, found {} ", v.kind())
}

fn num_token<'a>(v: &'a Value, kind: &str) -> Result<&'a str, String> {
    match v {
        Value::Num(s) => Ok(s),
        other => Err(want(kind, other)),
    }
}

/// Canonical decimal text (shortest round-trip, always with '.' or 'e'; -0 → 0.0).
fn canon_f64(v: f64) -> String {
    if v == 0.0 {
        "0.0".into()
    } else {
        format!("{v:?}")
    }
}
fn canon_f32(v: f32) -> String {
    if v == 0.0 {
        "0.0".into()
    } else {
        format!("{v:?}")
    }
}

macro_rules! int_arg {
    ($t:ty, $kind:literal) => {
        impl Arg for $t {
            const KIND: &'static str = $kind;
            fn from_value(v: &Value) -> Result<Self, String> {
                let s = num_token(v, $kind)?;
                s.parse::<$t>()
                    .map_err(|_| format!("'{s}' is not a valid {}", $kind))
            }
            fn to_value(&self) -> Value {
                Value::Num(self.to_string())
            }
            fn to_json(&self) -> Json {
                Json::from(*self)
            }
            fn from_json(j: &Json) -> Result<Self, String> {
                j.as_i64()
                    .and_then(|v| <$t>::try_from(v).ok())
                    .ok_or_else(|| format!("expected {} in JSON", $kind))
            }
        }
    };
}
int_arg!(u32, "u32");
int_arg!(i64, "i64");
int_arg!(u8, "u8");

impl Arg for f64 {
    const KIND: &'static str = "f64";
    fn from_value(v: &Value) -> Result<Self, String> {
        let s = num_token(v, Self::KIND)?;
        s.parse::<f64>()
            .ok()
            .filter(|x| x.is_finite())
            .ok_or_else(|| format!("'{s}' is not a finite f64"))
    }
    fn to_value(&self) -> Value {
        Value::Num(canon_f64(*self))
    }
    fn to_json(&self) -> Json {
        serde_json::Number::from_f64(*self).map_or(Json::Null, Json::Number)
    }
    fn from_json(j: &Json) -> Result<Self, String> {
        j.as_f64().ok_or_else(|| "expected a number in JSON".into())
    }
}

impl Arg for f32 {
    const KIND: &'static str = "f32";
    fn from_value(v: &Value) -> Result<Self, String> {
        let s = num_token(v, Self::KIND)?;
        s.parse::<f32>()
            .ok()
            .filter(|x| x.is_finite())
            .ok_or_else(|| format!("'{s}' is not a finite f32"))
    }
    fn to_value(&self) -> Value {
        Value::Num(canon_f32(*self))
    }
    fn to_json(&self) -> Json {
        // Through the canonical text, so JSON shows the same digits as the text form.
        canon_f32(*self)
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map_or(Json::Null, Json::Number)
    }
    fn from_json(j: &Json) -> Result<Self, String> {
        j.as_f64()
            .map(|v| v as f32)
            .filter(|v| v.is_finite())
            .ok_or_else(|| "expected a number in JSON".into())
    }
}

impl Arg for bool {
    const KIND: &'static str = "bool";
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Bool(b) => Ok(*b),
            other => Err(want("boolean", other)),
        }
    }
    fn to_value(&self) -> Value {
        Value::Bool(*self)
    }
    fn to_json(&self) -> Json {
        Json::Bool(*self)
    }
    fn from_json(j: &Json) -> Result<Self, String> {
        j.as_bool()
            .ok_or_else(|| "expected a boolean in JSON".into())
    }
}

impl Arg for String {
    const KIND: &'static str = "string";
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Str(s) => Ok(s.clone()),
            other => Err(want("string", other)),
        }
    }
    fn to_value(&self) -> Value {
        Value::Str(self.clone())
    }
    fn to_json(&self) -> Json {
        Json::String(self.clone())
    }
    fn from_json(j: &Json) -> Result<Self, String> {
        j.as_str()
            .map(str::to_owned)
            .ok_or_else(|| "expected a string in JSON".into())
    }
}

impl Arg for LayerId {
    const KIND: &'static str = "layer id";
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Id(LayerId::PREFIX, n) => Ok(LayerId(*n)),
            Value::Id(k, n) => Err(format!("{k}{n} is not a layer id (L…)")),
            other => Err(want("layer id", other)),
        }
    }
    fn to_value(&self) -> Value {
        Value::Id(LayerId::PREFIX, self.0)
    }
    fn to_json(&self) -> Json {
        Json::String(self.to_string())
    }
    fn from_json(j: &Json) -> Result<Self, String> {
        let s = j
            .as_str()
            .ok_or("expected a layer id string like \"L2\" in JSON")?;
        text_value(s).and_then(|v| Self::from_value(&v))
    }
}

/// Colour: `#rrggbb` or `#rrggbbaa`.
impl Arg for Rgba8 {
    const KIND: &'static str = "colour";
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Color { rgba, .. } => Ok(Rgba8(*rgba)),
            other => Err(want("colour", other)),
        }
    }
    fn to_value(&self) -> Value {
        Value::Color {
            rgba: self.0,
            has_alpha: self.0[3] != 255,
        }
    }
    fn to_json(&self) -> Json {
        Json::String(value_text(&self.to_value()))
    }
    fn from_json(j: &Json) -> Result<Self, String> {
        let s = j
            .as_str()
            .ok_or("expected a colour string like \"#ff8800\" in JSON")?;
        text_value(s).and_then(|v| Self::from_value(&v))
    }
}

/// Arbitrary bytes (`b64:`).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Blob(pub Vec<u8>);

impl Arg for Blob {
    const KIND: &'static str = "blob";
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Blob(b) => Ok(Blob(b.clone())),
            other => Err(want("blob", other)),
        }
    }
    fn to_value(&self) -> Value {
        Value::Blob(self.0.clone())
    }
    fn to_json(&self) -> Json {
        Json::String(format!("b64:{}", B64.encode(&self.0)))
    }
    fn from_json(j: &Json) -> Result<Self, String> {
        let s = j.as_str().ok_or("expected a \"b64:…\" string in JSON")?;
        text_value(s).and_then(|v| Self::from_value(&v))
    }
}

/// Implements [`Arg`] for an enum with `name()`/`from_name()` (bare-word values).
macro_rules! word_arg {
    ($t:ty, $kind:literal) => {
        impl Arg for $t {
            const KIND: &'static str = $kind;
            fn from_value(v: &Value) -> Result<Self, String> {
                match v {
                    Value::Word(w) => {
                        <$t>::from_name(w).ok_or_else(|| format!("unknown {} '{w}'", $kind))
                    }
                    other => Err(want($kind, other)),
                }
            }
            fn to_value(&self) -> Value {
                Value::Word(self.name().to_owned())
            }
            fn to_json(&self) -> Json {
                Json::String(self.name().to_owned())
            }
            fn from_json(j: &Json) -> Result<Self, String> {
                let s = j
                    .as_str()
                    .ok_or(concat!("expected a ", $kind, " name in JSON"))?;
                <$t>::from_name(s).ok_or_else(|| format!("unknown {} '{s}'", $kind))
            }
        }
    };
}

word_arg!(BlendMode, "blend mode");
word_arg!(LayerKind, "layer kind");

/// Parse a single value from its text form (used for JSON strings in text syntax).
pub fn text_value(s: &str) -> Result<Value, String> {
    use crate::syntax::{Line, parse_line};
    match parse_line(&format!("x {s}")) {
        Ok(Line::Item(r)) if r.positional.len() == 1 && r.named.is_empty() => r
            .positional
            .into_iter()
            .next()
            .ok_or_else(|| "no value".into()),
        Ok(_) => Err(format!("'{s}' is not a single value")),
        Err(e) => Err(format!("'{s}': {}", e.msg)),
    }
}

/// Text form of a single value.
pub fn value_text(v: &Value) -> String {
    use crate::syntax::{RawLine, print_line};
    let line = print_line(&RawLine {
        name: "x".into(),
        version: 1,
        positional: vec![v.clone()],
        ..Default::default()
    });
    line.strip_prefix("x ").unwrap_or(&line).to_owned()
}

/// Schema kind of an optional parameter (`Option<T>` → `T::KIND`).
pub trait OptArg {
    const KIND: &'static str;
}

impl<T: Arg> OptArg for Option<T> {
    const KIND: &'static str = T::KIND;
}

/// How a parameter is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Required, positional, in declared order.
    Pos,
    /// Required `key=value`.
    Key,
    /// Optional `key=value`, omitted when unset.
    Opt,
}

/// Schema entry for one parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParamSpec {
    pub name: &'static str,
    pub kind: &'static str,
    pub mode: Mode,
    pub doc: &'static str,
}

/// Collects arguments while reading a [`crate::syntax::RawLine`].
pub struct Reader<'a> {
    positional: std::slice::Iter<'a, Value>,
    named: Vec<(&'a str, &'a Value, bool)>,
}

impl<'a> Reader<'a> {
    pub fn new(positional: &'a [Value], named: &'a [(String, Value)]) -> Self {
        Self {
            positional: positional.iter(),
            named: named.iter().map(|(k, v)| (k.as_str(), v, false)).collect(),
        }
    }
    pub fn pos<T: Arg>(&mut self, name: &str) -> Result<T, String> {
        let v = self
            .positional
            .next()
            .ok_or_else(|| format!("missing positional argument <{name}>"))?;
        T::from_value(v).map_err(|e| format!("<{name}>: {e}"))
    }
    fn take(&mut self, name: &str) -> Option<&'a Value> {
        self.named
            .iter_mut()
            .find(|(k, _, used)| *k == name && !*used)
            .map(|e| {
                e.2 = true;
                e.1
            })
    }
    pub fn key<T: Arg>(&mut self, name: &str) -> Result<T, String> {
        let v = self.take(name).ok_or_else(|| format!("missing {name}=…"))?;
        T::from_value(v).map_err(|e| format!("{name}: {e}"))
    }
    pub fn opt<T: Arg>(&mut self, name: &str) -> Result<Option<T>, String> {
        self.take(name)
            .map(|v| T::from_value(v).map_err(|e| format!("{name}: {e}")))
            .transpose()
    }
    pub fn finish(mut self) -> Result<(), String> {
        if self.positional.next().is_some() {
            return Err("too many positional arguments".into());
        }
        match self.named.iter().find(|(_, _, used)| !used) {
            Some((k, _, _)) => Err(format!("unknown key '{k}'")),
            None => Ok(()),
        }
    }
}

/// Collects arguments while reading the JSON form.
pub struct JsonReader<'a> {
    obj: &'a serde_json::Map<String, Json>,
    used: Vec<&'a str>,
}

impl<'a> JsonReader<'a> {
    pub fn new(obj: &'a serde_json::Map<String, Json>) -> Self {
        Self {
            obj,
            used: Vec::new(),
        }
    }
    fn get(&mut self, name: &'a str) -> Option<&'a Json> {
        let v = self.obj.get(name);
        if v.is_some() {
            self.used.push(name);
        }
        v
    }
    pub fn req<T: Arg>(&mut self, name: &'a str) -> Result<T, String> {
        let j = self
            .get(name)
            .ok_or_else(|| format!("missing \"{name}\""))?;
        T::from_json(j).map_err(|e| format!("{name}: {e}"))
    }
    pub fn opt<T: Arg>(&mut self, name: &'a str) -> Result<Option<T>, String> {
        match self.get(name) {
            None | Some(Json::Null) => Ok(None),
            Some(j) => T::from_json(j)
                .map(Some)
                .map_err(|e| format!("{name}: {e}")),
        }
    }
    pub fn finish(self) -> Result<(), String> {
        match self.obj.keys().find(|k| !self.used.contains(&k.as_str())) {
            Some(k) => Err(format!("unknown key \"{k}\"")),
            None => Ok(()),
        }
    }
}

/// Declares a parameter struct and derives its text codec, JSON form and schema.
///
/// ```ignore
/// params! {
///     /// Set layer properties.
///     pub struct LayerSet {
///         layer: LayerId = pos "Target layer",
///         name: Option<String> = opt "New name",
///     }
/// }
/// ```
/// Modes: `pos` (required positional), `key` (required key=value), `opt` (`Option<T>`, omitted
/// when `None`).
#[macro_export]
macro_rules! params {
    (
        $(#[$meta:meta])*
        pub struct $S:ident {
            $( $f:ident : $T:ty = $mode:ident $doc:literal ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct $S { $( pub $f: $T ),* }

        impl $S {
            pub const PARAMS: &'static [$crate::params::ParamSpec] = &[
                $( $crate::params::ParamSpec {
                    name: stringify!($f),
                    kind: $crate::params!(@kind $mode $T),
                    mode: $crate::params!(@mode $mode),
                    doc: $doc,
                } ),*
            ];

            pub fn read(r: &mut $crate::params::Reader<'_>) -> Result<Self, String> {
                let _ = &r;
                Ok(Self { $( $f: $crate::params!(@read $mode r $f) ),* })
            }

            pub fn write(&self, pos: &mut Vec<$crate::syntax::Value>, named: &mut Vec<(String, $crate::syntax::Value)>) {
                let _ = (&pos, &named);
                $( $crate::params!(@write $mode self pos named $f); )*
            }

            pub fn read_json(r: &mut $crate::params::JsonReader<'_>) -> Result<Self, String> {
                let _ = &r;
                Ok(Self { $( $f: $crate::params!(@readj $mode r $f) ),* })
            }

            pub fn write_json(&self) -> serde_json::Map<String, serde_json::Value> {
                #[allow(unused_mut)]
                let mut m = serde_json::Map::new();
                $( $crate::params!(@writej $mode self m $f); )*
                m
            }
        }
    };

    (@kind opt $T:ty) => { <$T as $crate::params::OptArg>::KIND };
    (@kind $mode:ident $T:ty) => { <$T as $crate::params::Arg>::KIND };
    (@mode pos) => { $crate::params::Mode::Pos };
    (@mode key) => { $crate::params::Mode::Key };
    (@mode opt) => { $crate::params::Mode::Opt };

    (@read pos $r:ident $f:ident) => { $r.pos(stringify!($f))? };
    (@read key $r:ident $f:ident) => { $r.key(stringify!($f))? };
    (@read opt $r:ident $f:ident) => { $r.opt(stringify!($f))? };

    (@write pos $s:ident $p:ident $n:ident $f:ident) => {
        $p.push($crate::params::Arg::to_value(&$s.$f));
    };
    (@write key $s:ident $p:ident $n:ident $f:ident) => {
        $n.push((stringify!($f).to_owned(), $crate::params::Arg::to_value(&$s.$f)));
    };
    (@write opt $s:ident $p:ident $n:ident $f:ident) => {
        if let Some(v) = &$s.$f {
            $n.push((stringify!($f).to_owned(), $crate::params::Arg::to_value(v)));
        }
    };

    (@readj pos $r:ident $f:ident) => { $r.req(stringify!($f))? };
    (@readj key $r:ident $f:ident) => { $r.req(stringify!($f))? };
    (@readj opt $r:ident $f:ident) => { $r.opt(stringify!($f))? };

    (@writej opt $s:ident $m:ident $f:ident) => {
        if let Some(v) = &$s.$f {
            $m.insert(stringify!($f).to_owned(), $crate::params::Arg::to_json(v));
        }
    };
    (@writej $mode:ident $s:ident $m:ident $f:ident) => {
        $m.insert(stringify!($f).to_owned(), $crate::params::Arg::to_json(&$s.$f));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimals_print_canonically() {
        assert_eq!(0.8f64.to_value(), Value::Num("0.8".into()));
        assert_eq!(1.0f64.to_value(), Value::Num("1.0".into()));
        assert_eq!((-0.0f64).to_value(), Value::Num("0.0".into()));
        assert_eq!(1e21f64.to_value(), Value::Num("1e21".into()));
        assert_eq!(0.1f32.to_value(), Value::Num("0.1".into()));
        // An integer token is accepted for a decimal parameter.
        assert_eq!(f64::from_value(&Value::Num("3".into())), Ok(3.0));
        assert!(f64::from_value(&Value::Num("1e999".into())).is_err());
    }

    #[test]
    fn typed_values_reject_wrong_kinds() {
        assert!(u32::from_value(&Value::Num("-1".into())).is_err());
        assert!(LayerId::from_value(&Value::Id('S', 2)).is_err());
        assert!(BlendMode::from_value(&Value::Word("multiplyy".into())).is_err());
        assert_eq!(
            BlendMode::from_value(&Value::Word("color_dodge".into())),
            Ok(BlendMode::ColorDodge)
        );
    }

    #[test]
    fn samples_accept_matrix_and_blob_and_print_blob() {
        let m =
            text_value("[10,20.5,0.5,0;12,21,0.75,4000]").unwrap_or_else(|e| unreachable!("{e}"));
        let s = Samples::from_value(&m).unwrap_or_else(|e| unreachable!("{e}"));
        assert_eq!(s.0.len(), 2);
        assert_eq!(s.0[1].x, 12.0);
        let blob = s.to_value();
        assert!(matches!(blob, Value::Blob(_)));
        assert_eq!(Samples::from_value(&blob), Ok(s));
        assert!(
            Samples::from_value(&text_value("[1,2,3]").unwrap_or_else(|e| unreachable!("{e}")))
                .is_err()
        );
        assert!(
            Samples::from_value(
                &text_value("[1,2,0.5,1.5]").unwrap_or_else(|e| unreachable!("{e}"))
            )
            .is_err()
        );
        assert!(
            Samples::from_value(
                &text_value("[1,2,0.5,10;1,2,0.5,5]").unwrap_or_else(|e| unreachable!("{e}"))
            )
            .is_err()
        );
    }

    #[test]
    fn single_value_text_round_trip() {
        for s in [
            "L2",
            "#ff8800",
            "#ff880080",
            "\"a b\"",
            "b64:AAEC",
            "multiply",
            "@current",
            "-1.5e-3",
        ] {
            let v = text_value(s).unwrap_or_else(|e| unreachable!("{e}"));
            assert_eq!(value_text(&v), s);
        }
    }
}

/// Stroke input samples (`docs/command-format.md` §7): accepted as a blob or a readable
/// matrix, always printed as the canonical blob. Values are quantized either way.
#[derive(Clone, Debug, PartialEq)]
pub struct Samples(pub Vec<brushye_core::input::Sample>);

impl Samples {
    fn from_matrix(rows: &[Vec<String>]) -> Result<Self, String> {
        use brushye_core::input::{Sample, canonical};
        let num = |s: &str| {
            s.parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| format!("'{s}' is not a number"))
        };
        let mut out = Vec::with_capacity(rows.len());
        for (i, r) in rows.iter().enumerate() {
            if r.len() != 4 && r.len() != 7 {
                return Err(format!(
                    "sample row {i} has {} values; expected [x,y,p,t] or [x,y,p,t,tilt_x,tilt_y,twist]",
                    r.len()
                ));
            }
            let v: Vec<f64> = r.iter().map(|s| num(s)).collect::<Result<_, _>>()?;
            if !(0.0..=f64::from(u32::MAX)).contains(&v[3]) || v[3].fract() != 0.0 {
                return Err(format!("sample row {i}: t must be whole microseconds"));
            }
            out.push(Sample {
                x: v[0],
                y: v[1],
                t_us: v[3] as u32,
                pressure: Some(v[2] as f32),
                tilt: (v.len() == 7).then(|| [v[4] as f32, v[5] as f32]),
                twist: (v.len() == 7).then(|| v[6] as f32),
                eraser: false,
                barrel: [false; 2],
            });
        }
        canonical(&out).map(Samples).map_err(|e| e.to_string())
    }
}

impl Arg for Samples {
    const KIND: &'static str = "stroke samples";
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Blob(b) => brushye_core::input::decode(b)
                .map(Samples)
                .map_err(|e| e.to_string()),
            Value::Matrix(rows) => Samples::from_matrix(rows),
            other => Err(format!(
                "expected stroke samples (b64:… or [x,y,p,t;…]), found {}",
                other.kind()
            )),
        }
    }
    fn to_value(&self) -> Value {
        // Samples held here are always canonical (decoded/quantized), so encoding can't fail.
        Value::Blob(brushye_core::input::encode(&self.0).unwrap_or_default())
    }
    fn to_json(&self) -> Json {
        Json::String(format!(
            "b64:{}",
            B64.encode(brushye_core::input::encode(&self.0).unwrap_or_default())
        ))
    }
    fn from_json(j: &Json) -> Result<Self, String> {
        let s = j.as_str().ok_or("expected a \"b64:…\" string in JSON")?;
        text_value(s).and_then(|v| Self::from_value(&v))
    }
}
