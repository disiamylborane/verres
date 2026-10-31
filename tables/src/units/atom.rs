
use crate::strings::Symbol;
use super::{Prefix, UnitId, UnitSystem};

/// Textual representation of [`AtomicUnit`]
#[derive(Debug)]
pub struct SymbolicUnitElement {
    /// Prefix + unit symbol without degree sign
    pub symbol: Symbol,

    /// Degree sign is (true=before, false=after, none=absent) the main symbol
    pub degree_is_prefix: Option<bool>,
}


/// A unit that is expressed as a single math symbol, includes prefix and base value, and may be used in unit expressions
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AtomicUnit {
    pub(super) prefix: Prefix,
    pub(super) root: UnitId,
    pub(super) degree_is_prefix: Option<bool>,
}


impl AtomicUnit {
    /// The UnitId this unit belongs to
    pub fn root(self) -> UnitId {self.root}

    /// Create AtomicUnit from its symbolic representation
    pub fn parse(symbol: &SymbolicUnitElement, units: &UnitSystem) -> Option<Self> {
        if symbol.degree_is_prefix == Some(true) {
            return if let Some((root, def)) = units.get_by_symbol(&symbol.symbol, true) {
                if def.is_degreed() {
                    Some(Self{prefix: Prefix::absent(), root, degree_is_prefix: Some(true)})
                }
                else {None}
            }
            else {None}
        }

        let (symbol, degreed) =
            if symbol.degree_is_prefix == Some(false) {
                (&symbol.symbol, true)
            } else {(&symbol.symbol, false)};

        for (prefix, sym) in crate::units::definition::as_prefixed_options(symbol, true) {
            if let Some((root, def)) = units.get_by_symbol(sym, degreed) {
                return if def.allows_prefixes() || prefix == Prefix::absent() {
                    Some(Self{prefix, root, degree_is_prefix: if degreed {Some(false)} else {None}})
                }
                else {None};
            }
        }
        None
    }

    printer!{
        /// Represent as expression
        repr(&AtomicUnit { prefix, root, degree_is_prefix } = &AtomicUnit, units: &UnitSystem, f) {
            if degree_is_prefix == Some(true) {
                write!(f, "°")?;
            }
            write!(f, "{}{}", prefix.name(), units[root].symbol)?;
            if degree_is_prefix == Some(false) {
                write!(f, "°")?;
            }
            Ok(())
        }
    }
}


/// An [`AtomicUnit`] that may be used in unit expressions
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AtomicComposeableUnit {
    pub(super) prefix: Prefix,
    pub(super) root: UnitId,
}

impl AtomicComposeableUnit {
    pub(super) fn from_atomic(at: AtomicUnit, units: &UnitSystem) -> Option<Self> {
        let AtomicUnit{prefix, root, degree_is_prefix} = at;
        match degree_is_prefix {
            Some(true) => None,
            Some(false) => Some(Self{prefix, root}),
            None => {
                let def = units.get(root)?;
                if def.composable() {Some(Self{prefix, root})} else {None}
            }
        }
    }

    pub(super) fn parse(symbol: &SymbolicUnitElement, units: &UnitSystem) -> Option<Self> {
        Self::from_atomic(AtomicUnit::parse(symbol, units)?, units)
    }

    printer!{
        /// Represent as expression
        repr(&AtomicComposeableUnit { prefix, root } = &AtomicComposeableUnit, units: &UnitSystem, f) {
            let unit = &units[root];
            write!(f, "{}{}{}", prefix.name(), unit.symbol, if unit.is_degreed() {"°"} else {""})?;
            Ok(())
        }
    }
}


