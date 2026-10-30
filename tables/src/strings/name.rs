
use std::ops::Deref;

use compact_str::CompactString;

/// A refined string allowed to be used as an entity name in experiment engine
#[derive(Clone, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct Name(CompactString);

impl Name {
    /// Create a name from string, checking that the format is allowed
    pub fn new(s: &str) -> Option<Self> {
        let mut chars = s.chars();
        let Some(first) = chars.next() else {None?};
        if first.is_alphabetic() && chars.all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-') {
            Some(Self(s.into()))
        } else {None}
    }
}

impl Deref for Name {
    type Target = str;
    fn deref(&self) -> &Self::Target {&self.0}
}

#[derive(Debug)]
pub struct NameIsIncorrect;

impl core::fmt::Display for NameIsIncorrect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Name is incorrect")
    }
}

impl core::str::FromStr for Name {
    type Err = NameIsIncorrect;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or(NameIsIncorrect)
    }
}
