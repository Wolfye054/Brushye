//! Text syntax of one command line (`docs/command-format.md`): tokens, values, parse, print.
//!
//! This layer is untyped: numbers keep their exact token, and the command's typed parameters
//! (see `params`) decide how to read and canonically print them.

use std::fmt::Write as _;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;

/// One argument value as written.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// A validated integer or decimal token, kept verbatim.
    Num(String),
    Bool(bool),
    /// `#rrggbb` (alpha 255, `has_alpha` false) or `#rrggbbaa`.
    Color {
        rgba: [u8; 4],
        has_alpha: bool,
    },
    /// Object id: kind letter + number, e.g. `L2`.
    Id(char, u32),
    /// `@word`
    Ref(String),
    /// Bare lowercase word (enum value).
    Word(String),
    Str(String),
    Blob(Vec<u8>),
    /// Rows of number tokens.
    Matrix(Vec<Vec<String>>),
}

impl Value {
    pub fn kind(&self) -> &'static str {
        match self {
            Value::Num(_) => "number",
            Value::Bool(_) => "boolean",
            Value::Color { .. } => "colour",
            Value::Id(..) => "id",
            Value::Ref(_) => "reference",
            Value::Word(_) => "word",
            Value::Str(_) => "string",
            Value::Blob(_) => "blob",
            Value::Matrix(_) => "matrix",
        }
    }
}

/// A parsed line, before typing.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct RawLine {
    /// Journal sequence number (`@n`); `None` in scripts and for queries.
    pub seq: Option<u64>,
    /// `@n+`: joins the previous undo step (D-013).
    pub merge: bool,
    /// `?name …`
    pub query: bool,
    pub name: String,
    /// Command version; 1 when not written.
    pub version: u32,
    pub positional: Vec<Value>,
    pub named: Vec<(String, Value)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxError {
    /// 1-based column (in characters).
    pub col: usize,
    pub msg: String,
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "column {}: {}", self.col, self.msg)
    }
}

/// What a line is.
#[derive(Clone, Debug, PartialEq)]
pub enum Line {
    Blank,
    Comment(String),
    Item(RawLine),
}

struct Cur<'a> {
    s: &'a [char],
    i: usize,
}

fn is_word_start(c: char) -> bool {
    c.is_ascii_lowercase()
}
fn is_word_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'
}

impl<'a> Cur<'a> {
    fn peek(&self) -> Option<char> {
        self.s.get(self.i).copied()
    }
    fn err<T>(&self, msg: impl Into<String>) -> Result<T, SyntaxError> {
        Err(SyntaxError {
            col: self.i + 1,
            msg: msg.into(),
        })
    }
    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn at_end(&self) -> bool {
        self.i >= self.s.len()
    }
    fn skip_space(&mut self) -> bool {
        let start = self.i;
        while matches!(self.peek(), Some(' ' | '\t')) {
            self.i += 1;
        }
        self.i > start
    }
    fn take_while(&mut self, f: impl Fn(char) -> bool) -> String {
        let start = self.i;
        while self.peek().is_some_and(&f) {
            self.i += 1;
        }
        self.s[start..self.i].iter().collect()
    }
    fn word(&mut self) -> Result<String, SyntaxError> {
        if !self.peek().is_some_and(is_word_start) {
            return self.err("expected a lowercase word");
        }
        Ok(self.take_while(is_word_char))
    }
    /// `word(.word)*`
    fn dotted(&mut self, later_may_start_with_digit: bool) -> Result<String, SyntaxError> {
        let mut out = self.word()?;
        while self.peek() == Some('.') {
            self.i += 1;
            let part = if later_may_start_with_digit {
                let p = self.take_while(is_word_char);
                if p.is_empty() {
                    return self.err("expected a word after '.'");
                }
                p
            } else {
                self.word()?
            };
            out.push('.');
            out.push_str(&part);
        }
        Ok(out)
    }
    fn uint(&mut self) -> Result<u64, SyntaxError> {
        let start = self.i;
        let d = self.take_while(|c| c.is_ascii_digit());
        if d.is_empty() {
            return self.err("expected a number");
        }
        if d.len() > 1 && d.starts_with('0') {
            self.i = start;
            return self.err("leading zeros are not allowed");
        }
        d.parse().or_else(|_| {
            self.i = start;
            self.err("number too large")
        })
    }
    /// Number token per the grammar; returns the verbatim text.
    fn number(&mut self) -> Result<String, SyntaxError> {
        let start = self.i;
        self.eat('-');
        let int = self.take_while(|c| c.is_ascii_digit());
        if int.is_empty() {
            self.i = start;
            return self.err("expected a number");
        }
        if int.len() > 1 && int.starts_with('0') {
            self.i = start;
            return self.err("leading zeros are not allowed");
        }
        if self.eat('.') && self.take_while(|c| c.is_ascii_digit()).is_empty() {
            return self.err("expected digits after '.'");
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.i += 1;
            if !self.eat('+') {
                self.eat('-');
            }
            if self.take_while(|c| c.is_ascii_digit()).is_empty() {
                return self.err("expected exponent digits");
            }
        }
        Ok(self.s[start..self.i].iter().collect())
    }
    fn string(&mut self) -> Result<String, SyntaxError> {
        // opening quote already consumed
        let mut out = String::new();
        loop {
            let Some(c) = self.peek() else {
                return self.err("unterminated string");
            };
            self.i += 1;
            match c {
                '"' => return Ok(out),
                '\\' => {
                    let Some(e) = self.peek() else {
                        return self.err("unterminated escape");
                    };
                    self.i += 1;
                    match e {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        'n' => out.push('\n'),
                        't' => out.push('\t'),
                        'r' => out.push('\r'),
                        'u' => {
                            if !self.eat('{') {
                                return self.err("expected '{' after \\u");
                            }
                            let h = self.take_while(|c| c.is_ascii_hexdigit());
                            if h.is_empty() || h.len() > 6 || !self.eat('}') {
                                return self.err("expected 1-6 hex digits and '}' in \\u{…}");
                            }
                            let v = u32::from_str_radix(&h, 16).unwrap_or(u32::MAX);
                            match char::from_u32(v) {
                                Some(ch) => out.push(ch),
                                None => return self.err("\\u{…} is not a Unicode scalar value"),
                            }
                        }
                        _ => return self.err(format!("unknown escape \\{e}")),
                    }
                }
                c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                    self.i -= 1;
                    return self.err("control characters must be escaped");
                }
                c => out.push(c),
            }
        }
    }
    fn value(&mut self) -> Result<Value, SyntaxError> {
        let Some(c) = self.peek() else {
            return self.err("expected a value");
        };
        match c {
            '"' => {
                self.i += 1;
                Ok(Value::Str(self.string()?))
            }
            '#' => {
                self.i += 1;
                let h = self.take_while(|c| c.is_ascii_hexdigit());
                let byte = |k: usize| u8::from_str_radix(&h[k..k + 2], 16).unwrap_or(0);
                match h.len() {
                    6 => Ok(Value::Color {
                        rgba: [byte(0), byte(2), byte(4), 255],
                        has_alpha: false,
                    }),
                    8 => Ok(Value::Color {
                        rgba: [byte(0), byte(2), byte(4), byte(6)],
                        has_alpha: true,
                    }),
                    _ => self.err("colour must be #rrggbb or #rrggbbaa"),
                }
            }
            '@' => {
                self.i += 1;
                Ok(Value::Ref(self.word()?))
            }
            '[' => {
                self.i += 1;
                let mut rows = Vec::new();
                if self.eat(']') {
                    return Ok(Value::Matrix(rows));
                }
                loop {
                    let mut row = vec![self.number()?];
                    while self.eat(',') {
                        row.push(self.number()?);
                    }
                    rows.push(row);
                    if self.eat(']') {
                        return Ok(Value::Matrix(rows));
                    }
                    if !self.eat(';') {
                        return self.err("expected ',', ';' or ']' in matrix");
                    }
                }
            }
            c if c == '-' || c.is_ascii_digit() => Ok(Value::Num(self.number()?)),
            c if c.is_ascii_uppercase() => {
                self.i += 1;
                let n = self.uint()?;
                match u32::try_from(n) {
                    Ok(n) => Ok(Value::Id(c, n)),
                    Err(_) => self.err("id number too large"),
                }
            }
            c if is_word_start(c) => {
                let start = self.i;
                let w = self.take_while(is_word_char);
                if w == "b64" && self.eat(':') {
                    let data = self
                        .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '='));
                    return match B64.decode(data.as_bytes()) {
                        Ok(b) => Ok(Value::Blob(b)),
                        Err(e) => {
                            self.i = start;
                            self.err(format!("invalid base64: {e}"))
                        }
                    };
                }
                Ok(match w.as_str() {
                    "true" => Value::Bool(true),
                    "false" => Value::Bool(false),
                    _ => Value::Word(w),
                })
            }
            _ => self.err(format!("unexpected character '{c}'")),
        }
    }
}

/// Parse one line (without its line terminator; a trailing '\r' is accepted).
pub fn parse_line(line: &str) -> Result<Line, SyntaxError> {
    let line = line.strip_suffix('\r').unwrap_or(line);
    let chars: Vec<char> = line.chars().collect();
    let mut c = Cur { s: &chars, i: 0 };
    c.skip_space();
    if c.at_end() {
        return Ok(Line::Blank);
    }
    if c.peek() == Some('#') {
        return Ok(Line::Comment(chars[c.i + 1..].iter().collect()));
    }
    let mut raw = RawLine {
        version: 1,
        ..Default::default()
    };
    if c.eat('@') {
        raw.seq = Some(c.uint()?);
        raw.merge = c.eat('+');
        if !c.skip_space() {
            return c.err("expected a space after the sequence number");
        }
    }
    if c.eat('?') {
        if raw.seq.is_some() {
            return c.err("queries have no sequence number");
        }
        raw.query = true;
    }
    raw.name = c.dotted(false)?;
    if c.eat('/') {
        let v = c.uint()?;
        raw.version = u32::try_from(v).or_else(|_| c.err("version too large"))?;
        if raw.version == 0 {
            return c.err("versions start at 1");
        }
    }
    loop {
        let spaced = c.skip_space();
        if c.at_end() {
            break;
        }
        if !spaced {
            return c.err("expected a space between arguments");
        }
        // key=value?
        let save = c.i;
        if c.peek().is_some_and(is_word_start) {
            if let Ok(key) = c.dotted(true) {
                if c.eat('=') {
                    let v = c.value()?;
                    if raw.named.iter().any(|(k, _)| *k == key) {
                        c.i = save;
                        return c.err(format!("duplicate key '{key}'"));
                    }
                    raw.named.push((key, v));
                    continue;
                }
            }
            c.i = save;
        }
        if !raw.named.is_empty() {
            return c.err("positional arguments must come before key=value arguments");
        }
        raw.positional.push(c.value()?);
    }
    Ok(Line::Item(raw))
}

fn print_value(out: &mut String, v: &Value) {
    match v {
        Value::Num(s) => out.push_str(s),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Color { rgba, has_alpha } => {
            let _ = write!(out, "#{:02x}{:02x}{:02x}", rgba[0], rgba[1], rgba[2]);
            if *has_alpha {
                let _ = write!(out, "{:02x}", rgba[3]);
            }
        }
        Value::Id(k, n) => {
            let _ = write!(out, "{k}{n}");
        }
        Value::Ref(r) => {
            out.push('@');
            out.push_str(r);
        }
        Value::Word(w) => out.push_str(w),
        Value::Str(s) => {
            out.push('"');
            for ch in s.chars() {
                match ch {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\t' => out.push_str("\\t"),
                    '\r' => out.push_str("\\r"),
                    c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                        let _ = write!(out, "\\u{{{:x}}}", c as u32);
                    }
                    c => out.push(c),
                }
            }
            out.push('"');
        }
        Value::Blob(b) => {
            out.push_str("b64:");
            out.push_str(&B64.encode(b));
        }
        Value::Matrix(rows) => {
            out.push('[');
            for (i, r) in rows.iter().enumerate() {
                if i > 0 {
                    out.push(';');
                }
                out.push_str(&r.join(","));
            }
            out.push(']');
        }
    }
}

/// Canonical text of a line (no line terminator).
pub fn print_line(raw: &RawLine) -> String {
    let mut out = String::new();
    if let Some(s) = raw.seq {
        let _ = write!(out, "@{s}");
        if raw.merge {
            out.push('+');
        }
        out.push(' ');
    }
    if raw.query {
        out.push('?');
    }
    out.push_str(&raw.name);
    if raw.version > 1 {
        let _ = write!(out, "/{}", raw.version);
    }
    for v in &raw.positional {
        out.push(' ');
        print_value(&mut out, v);
    }
    for (k, v) in &raw.named {
        out.push(' ');
        out.push_str(k);
        out.push('=');
        print_value(&mut out, v);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(s: &str) -> RawLine {
        match parse_line(s) {
            Ok(Line::Item(r)) => r,
            other => unreachable!("{s:?} -> {other:?}"),
        }
    }

    #[test]
    fn parses_spec_examples() {
        let r = item("@3 layer.set L2 blend=multiply opacity=0.80");
        assert_eq!(r.seq, Some(3));
        assert_eq!(r.name, "layer.set");
        assert_eq!(r.positional, vec![Value::Id('L', 2)]);
        assert_eq!(r.named[0], ("blend".into(), Value::Word("multiply".into())));
        assert_eq!(r.named[1], ("opacity".into(), Value::Num("0.80".into())));
        let r = item("@4 tool.set brush.size=12.5 color=#1a1a1aff");
        assert_eq!(
            r.named[1].1,
            Value::Color {
                rgba: [0x1a, 0x1a, 0x1a, 0xff],
                has_alpha: true
            }
        );
        let r = item("@7+ layer.set/2 L2 name=\"Ink \\\"A\\\"\\n\"");
        assert!(r.merge);
        assert_eq!(r.version, 2);
        assert_eq!(r.named[0].1, Value::Str("Ink \"A\"\n".into()));
        let r = item("?pixel L2 x=10 y=-20");
        assert!(r.query);
        let r = item("stroke L2 tool=@current samples=[1,2.5,0.3,0;4,5,1e-3,8] data=b64:AAEC");
        assert_eq!(r.named[0].1, Value::Ref("current".into()));
        assert_eq!(
            r.named[1].1,
            Value::Matrix(vec![
                vec!["1".into(), "2.5".into(), "0.3".into(), "0".into()],
                vec!["4".into(), "5".into(), "1e-3".into(), "8".into()]
            ])
        );
        assert_eq!(r.named[2].1, Value::Blob(vec![0, 1, 2]));
    }

    #[test]
    fn blank_comment_and_crlf() {
        assert_eq!(parse_line("   \t"), Ok(Line::Blank));
        assert_eq!(parse_line("# hello"), Ok(Line::Comment(" hello".into())));
        assert!(matches!(parse_line("history.undo\r"), Ok(Line::Item(_))));
    }

    #[test]
    fn rejects_malformed_input() {
        for bad in [
            "layer.set L2 opacity=1 L3",     // positional after key
            "layer.set opacity=1 opacity=2", // duplicate key
            "Layer.set",                     // uppercase name
            "layer.set L02",                 // leading zeros in id
            "x n=007",                       // leading zeros in number
            "x n=1.",                        // missing fraction digits
            "x s=\"abc",                     // unterminated string
            "x s=\"\\q\"",                   // unknown escape
            "x c=#12345",                    // bad colour length
            "x b=b64:!!!",                   // bad base64
            "x/0",                           // version 0
            "@1 ?q",                         // query with seq
            "x m=[1,a]",                     // non-number in matrix
            "x a=1b=2",                      // missing space
        ] {
            assert!(parse_line(bad).is_err(), "should reject {bad:?}");
        }
    }

    #[test]
    fn printing_is_canonical_and_round_trips() {
        let src = "@12+ layer.set/3 L2 name=\"a\\u{1}b\\\\\" blend=multiply on=true c=#ff8800 r=@current d=b64: m=[1,2;3,-4.5e2] e=[]";
        let r = item(src);
        let printed = print_line(&r);
        assert_eq!(printed, src);
        assert_eq!(item(&printed), r);
    }
}
