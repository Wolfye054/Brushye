//! Object ids. Ids are assigned by commands (D-011), never by the executor, so replay can't
//! diverge. Text form: a kind letter and a number, e.g. `L2` for a layer.

use std::fmt;

/// A layer id; text form `L<n>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerId(pub u32);

impl LayerId {
    pub const PREFIX: char = 'L';
}

impl fmt::Display for LayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", Self::PREFIX, self.0)
    }
}
