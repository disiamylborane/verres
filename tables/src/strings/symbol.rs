
use std::{fmt::Display, ops::Deref, str::FromStr};

use compact_str::CompactString;

/// A short string containing only alphabetic characters, used for math expressions
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Symbol(CompactString);

impl Symbol {
    /// Create a symbol from string, checking that the format is allowed
    pub fn new(s: &str) -> Option<Self> {
        let mut chars = s.chars();
        if chars.all(|c| c.is_alphabetic() || c == '%') {
            Some(Self(s.into()))
        } else {None}
    }
}

impl Deref for Symbol {
    type Target = str;
    fn deref(&self) -> &Self::Target {&self.0}
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {self.0.fmt(f)}
}

pub struct SymbolIsIncorrect;

impl core::fmt::Display for SymbolIsIncorrect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Symbol is incorrect")
    }
}

impl FromStr for Symbol {
    type Err = SymbolIsIncorrect;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or(SymbolIsIncorrect)
    }
}
