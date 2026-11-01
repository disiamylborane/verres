
use compact_str::CompactString;

use numbers::Number;

use crate::strings::{Localized, Name, Symbol};

use super::{Quantity, Prefix};


/// Base unit is always an absolute unit
pub struct BaseUnitDefinition {
    /// The textual data describing and identifying the unit
    pub names: UnitNames,
    /// None only absent prefix is allowed, otherwise the default prefix for this unit
    pub allows_prefixes: Option<Prefix>,
}


/// The textual data describing and identifying the unit
pub struct UnitNames {
    /// Main math symbol to denote unit
    pub symbol: Symbol,
    /// Unit name for documentation
    pub name: Localized<Name>,
    /// Unit description, contains the definition
    pub description: Localized<CompactString>,
}


/// The full description for unit of measurement
pub struct UnitDefinition {
    /// The textual data describing and identifying the unit
    pub names: UnitNames,
    /// Unit meaning
    pub kind: UnitKind,
}


pub struct AbsoluteUnitDefinition {
    /// Value of one unit
    pub scale: Quantity,
    /// None only absent prefix is allowed, otherwise the default prefix for this unit
    pub allows_prefixes: Option<Prefix>
}

pub struct RelativeUnitDefinition {
    /// Value representing the step of one unit
    pub scale: Quantity,
    /// Scale zero point expressed in base units
    pub offset: Number
}

pub struct LogRatioUnitDefinition {
    /// Ratio of magnitudes meant by 1 this unit
    pub power_per_unit: Number,
    /// None if disallows prefixes, otherwise the default prefix for this unit
    pub allows_prefixes: Option<Prefix>
}

pub struct LogPointUnitDefinition {
    /// Value corresponding to 0 this unit
    pub zero_unit: Quantity,
    /// Ratio of magnitudes meant by increase by 1 this unit
    pub log_base: Number,
    /// None if disallows prefixes, otherwise the default prefix for this unit
    pub allows_prefixes: Option<Prefix>
}

/// Unit kind defines the meaning of the unit
pub enum UnitKind {
    /// Just a regular unit that proportionally scales to some power expression on base units (Newtons)
    Absolute(AbsoluteUnitDefinition),

    /// Linear, but not proportional unit. Always denoted as degrees in this framework (Fahrenheit degrees)
    Relative(RelativeUnitDefinition),

    /// Units defining logarithmic ratios (decibels, octaves)
    LogDimensionless(LogRatioUnitDefinition),

    /// Units representing absolute values in log scale (dB SPL)
    LogPoint(LogPointUnitDefinition),
}



impl UnitDefinition {
    /// Must the unit have degree symbol
    pub fn is_degreed(&self) -> bool {
        match &self.kind {
            UnitKind::Absolute(_)=> false,
            UnitKind::Relative(_) => true,
            UnitKind::LogDimensionless(_) => false,
            UnitKind::LogPoint(_) => false,
        }
    }

    /// Check if the unit allows using decimal and binary prefixes
    pub fn allows_prefixes(&self) -> bool {
        match &self.kind {
            UnitKind::Absolute(AbsoluteUnitDefinition { scale:_, allows_prefixes })
            | UnitKind::LogPoint(LogPointUnitDefinition { zero_unit:_, log_base:_, allows_prefixes })
            | UnitKind::LogDimensionless(LogRatioUnitDefinition { power_per_unit:_, allows_prefixes }) =>
                allows_prefixes.is_some(),
            UnitKind::Relative(RelativeUnitDefinition { scale:_, offset:_ }) => false,
        }
    }

    pub(super) fn as_prefixed_options(&self) -> impl Iterator<Item=(Prefix, &'_ str)> {
        as_prefixed_options(&self.names.symbol, self.allows_prefixes())
    }
}



pub fn as_prefixed_options(symbol: &str, allows_prefixes: bool) -> impl Iterator<Item=(Prefix, &'_ str)> {
    let s: &str = symbol;
    let mut candidates = [(Prefix::absent(), s), (Prefix::absent(), s), (Prefix::absent(), s)];
    let mut i = 1;
    if allows_prefixes {
        'a: {
            let Some((head, tail)) = s.split_at_checked(1) else {break 'a};
            let Some(prefix) = Prefix::parse(head) else {break 'a};
            if tail.is_empty() {break 'a};
            candidates[i] = (prefix, tail);
            i += 1;
        };
        'b: {
            let Some((head, tail)) = s.split_at_checked(2) else {break 'b};
            let Some(prefix) = Prefix::parse(head) else {break 'b};
            if tail.is_empty() {break 'b};
            candidates[i] = (prefix, tail);
            i += 1;
        };
    }
    candidates.into_iter().take(i)
}
