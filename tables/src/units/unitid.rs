use std::fmt::Debug;

use nimi_pona::NimiPonaConvert;


/// Max amount of base physical quantities in [`UnitsEngine`]
pub const MAX_BASE_UNITS: u8 = 7;


/// Unique ID of a measurement unit defined in [`UnitsEngine`]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct UnitId(pub(super) u16);


impl NimiPonaConvert for UnitId {
    fn display_nimipona(self) -> impl std::fmt::Display {
        self.0.display_nimipona()
    }

    fn parse_nimipona(str: &str) -> Result<Self, nimi_pona::NimiPonaParseError> {
        Ok(Self(u16::parse_nimipona(str)?))
    }
}


impl UnitId {
    /// Create an identifier for a base unit
    pub(super) const fn base(id: u8) -> Self {
        assert!(id < MAX_BASE_UNITS);
        Self(id as u16)
    }

    pub(super) const fn first_derived() -> Self {
        Self(MAX_BASE_UNITS as u16)
    }

    pub(super) const fn next_derived(self) -> Option<Self> {
        let Some(id) = self.0.checked_add(1) else {return None};
        Some(Self(id))
    }
}
