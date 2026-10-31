
use compact_str::CompactString;

use numbers::Number;

use crate::strings::{Localized, Name, Symbol};

use super::{Quantity, Prefix, Dimensionality};


/// Base unit is always an absolute unit
pub struct BaseUnitDefinition {
    /// Main math symbol to denote unit
    pub symbol: Symbol,
    /// Unit name for documentation
    pub name: Localized<Name>,
    /// Unit description, contains the definition
    pub description: Localized<CompactString>,
    /// None only absent prefix is allowed, otherwise the default prefix for this unit
    pub allows_prefixes: Option<Prefix>,
}



/// The full description for unit of measurement
pub struct UnitDefinition {
    /// Main math symbol to denote unit
    pub symbol: Symbol,
    /// Unit name for documentation
    pub name: Localized<Name>,
    /// Unit description, contains the definition
    pub description: Localized<CompactString>,
    /// Unit meaning
    pub kind: UnitKind,
}


/// Unit kind defines the meaning of the unit
pub enum UnitKind {
    /// Just a regular unit that proportionally scales to some power expression on base units (Newtons)
    Absolute {
        /// Value of one unit
        scale: Quantity,
        /// None only absent prefix is allowed, otherwise the default prefix for this unit
        allow_prefixes: Option<Prefix>
    },

    /// Linear, but not proportional unit. Always denoted as degrees in this framework (Fahrenheit degrees)
    Relative {
        /// Value representing the step of one unit
        scale: Quantity,
        /// Distance between absolute zero and this scale zero expressed in this scale units
        offset: Number
    },

    /// Percents, Permilles, ppms and other units defining the fraction of some value
    LinearRatio {
        /// What part of a value is meant (1/100 for percent)
        multiplier: Number
    },

    /// Units defining logarithmic ratios (decibels, octaves)
    LogDimensionless {
        /// Ratio of magnitudes meant by 1 this unit
        power_per_unit: Number,
        /// None if disallows prefixes, otherwise the default prefix for this unit
        allow_prefixes: Option<Prefix>
    },

    /// Units representing absolute values in log scale (dB SPL)
    LogPoint {
        /// Value corresponding to 0 this unit
        zero_unit: Quantity,
        /// Ratio of magnitudes meant by increase by 1 this unit
        log_base: Number,
        /// None if disallows prefixes, otherwise the default prefix for this unit
        allow_prefixes: Option<Prefix>
    }
}



impl UnitDefinition {
    /// Get dimensionality of the unit
    pub fn dimensionality(&self) -> Dimensionality {
        match &self.kind {
            UnitKind::Absolute { scale, allow_prefixes:_ } => scale.dimensionality.clone(),
            UnitKind::Relative { scale, offset:_ } => scale.dimensionality.clone(),
            UnitKind::LinearRatio { multiplier:_ } => Dimensionality::DIMENSIONLESS,
            UnitKind::LogDimensionless { power_per_unit:_, allow_prefixes:_ } => Dimensionality::DIMENSIONLESS,
            UnitKind::LogPoint { zero_unit, log_base:_, allow_prefixes:_ } => zero_unit.dimensionality.clone(),
        }
    }

    /// Check if the unit may be used in unit power expressions (kg m2 s-1)
    pub fn composable(&self) -> bool {
        match &self.kind {
            UnitKind::Absolute { scale:_, allow_prefixes:_ } => true,
            UnitKind::Relative { scale:_, offset:_ } => true,
            UnitKind::LinearRatio { multiplier:_ } => true,
            UnitKind::LogDimensionless { power_per_unit:_, allow_prefixes:_ } => false,
            UnitKind::LogPoint { zero_unit:_, log_base:_, allow_prefixes:_ } => false,
        }
    }

    /// Must the unit have degree symbol
    pub fn is_degreed(&self) -> bool {
        match &self.kind {
            UnitKind::Absolute { scale:_, allow_prefixes:_ } => false,
            UnitKind::Relative { scale:_, offset:_ } => true,
            UnitKind::LinearRatio { multiplier:_ } => false,
            UnitKind::LogDimensionless { power_per_unit:_, allow_prefixes:_ } => false,
            UnitKind::LogPoint { zero_unit:_, log_base:_, allow_prefixes:_ } => false,
        }
    }

    /// Get the default prefix for the unit
    pub fn base_prefix(&self) -> Prefix {
        match &self.kind {
            UnitKind::Absolute { scale:_, allow_prefixes } => allow_prefixes.unwrap_or(Prefix::absent()),
            UnitKind::Relative { scale:_, offset:_ } => Prefix::absent(),
            UnitKind::LinearRatio { multiplier:_ } => Prefix::absent(),
            UnitKind::LogDimensionless { power_per_unit:_, allow_prefixes } => allow_prefixes.unwrap_or(Prefix::absent()),
            UnitKind::LogPoint { zero_unit:_, log_base:_, allow_prefixes } => allow_prefixes.unwrap_or(Prefix::absent()),
        }
    }

    /// Check if the unit allows using decimal and binary prefixes
    pub fn allows_prefixes(&self) -> bool {
        match &self.kind {
            UnitKind::Absolute { scale:_, allow_prefixes } => allow_prefixes.is_some(),
            UnitKind::Relative { scale:_, offset:_ } => false,
            UnitKind::LinearRatio { multiplier:_ } => false,
            UnitKind::LogDimensionless { power_per_unit:_, allow_prefixes } => allow_prefixes.is_some(),
            UnitKind::LogPoint { zero_unit:_, log_base:_, allow_prefixes } => allow_prefixes.is_some(),
        }
    }

    pub(super) fn as_prefixed_options(&self) -> impl Iterator<Item=(Prefix, &'_ str)> {
        as_prefixed_options(&self.symbol, self.allows_prefixes())
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
