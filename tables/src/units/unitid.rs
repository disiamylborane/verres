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


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct AbsoluteUnitId(UnitId);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct RelativeUnitId(UnitId);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct LogRatioUnitId(UnitId);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct LogPointUnitId(UnitId);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ExpressionUnitId(UnitId);
//#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
//pub(super) struct GeometricUnitId(UnitId);

impl AbsoluteUnitId {pub fn unitid(self) -> UnitId {self.0} pub fn new_proven(u: UnitId) -> Self {Self(u)}}
impl RelativeUnitId {pub fn unitid(self) -> UnitId {self.0} pub fn new_proven(u: UnitId) -> Self {Self(u)}}
impl LogRatioUnitId {pub fn unitid(self) -> UnitId {self.0} pub fn new_proven(u: UnitId) -> Self {Self(u)}}
impl LogPointUnitId {pub fn unitid(self) -> UnitId {self.0} pub fn new_proven(u: UnitId) -> Self {Self(u)}}

impl ExpressionUnitId {
    pub fn unitid(self) -> UnitId {self.0}
    pub fn new_absolute(abs: AbsoluteUnitId) -> Self {Self(abs.0)}
    pub fn new_relative(rel: RelativeUnitId) -> Self {Self(rel.0)}
}

/*impl GeometricUnitId {
    pub fn unitid(self) -> UnitId {self.0}
    pub fn new_absolute(abs: AbsoluteUnitId) -> Self {Self(abs.0)}
    pub fn new_log(log: LogRatioUnitId) -> Self {Self(log.0)}
}*/
