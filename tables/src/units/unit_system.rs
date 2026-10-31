use std::collections::HashMap;

use numbers::Rational;

use crate::{strings::{Language, Name}, units::{Quantity, definition::{AbsoluteUnitDefinition, LogPointUnitDefinition, LogRatioUnitDefinition, RelativeUnitDefinition}, unitid::{AbsoluteUnitId, ExpressionUnitId, LogPointUnitId, LogRatioUnitId, RelativeUnitId}}};

use super::{UnitDefinition, UnitId, UnitKind, Dimensionality, unitid::MAX_BASE_UNITS, BaseUnitDefinition};


/// The description for all units in scope
pub struct UnitSystem {
    base: Vec<UnitDefinition>,
    derived: HashMap<UnitId, UnitDefinition>,
}


pub(crate) enum UnitType {
    Absolute(AbsoluteUnitId),
    Relative(RelativeUnitId),
    LogRatio(LogRatioUnitId),
    LogPoint(LogPointUnitId),
}



#[derive(Debug)]
pub enum UnitDefinitionError {
    NoSpace,
    SymbolDuplication,
    IllegalBaseDefinition(u8),
    IncorrectSymbol,
    IncorrectName,
    IncorrectPrefix,
    IncorrectLanguage,
    IncorrectId,
}


impl UnitSystem {
    /// Add a new base unit to the engine
    pub fn new(base_units: Vec<BaseUnitDefinition>) -> Result<Self, UnitDefinitionError> {
        if base_units.len() > MAX_BASE_UNITS as usize {
            Err(UnitDefinitionError::NoSpace)
        }
        else {
            for i in 0..base_units.len() {
                for j in (i+1)..base_units.len() {
                    if base_units[i].names.symbol == base_units[j].names.symbol {
                        return Err(UnitDefinitionError::SymbolDuplication)
                    }
                }
            }

            let mut base = vec![];
            for (i, baseunit) in base_units.into_iter().enumerate() {
                let mut dim = [Rational::ZERO; MAX_BASE_UNITS as usize];
                dim[i] = Rational::ONE;
                let dimensionality = Dimensionality::new(dim);
                let kind = UnitKind::Absolute(AbsoluteUnitDefinition {
                    scale: Quantity {
                        value: Rational::ONE.into_number(),
                        dimensionality
                    },
                    allows_prefixes: baseunit.allows_prefixes,
                });
                base.push(UnitDefinition {
                    names: baseunit.names,
                    kind
                });
            }
            Ok(Self {base, derived: HashMap::new()})
        }
    }

    /// Add a new derived unit to the engine, return id of a unit
    pub fn define_derived(&mut self, desc: UnitDefinition) -> Result<UnitId, UnitDefinitionError> {
        let next_id = match self.derived.keys().max().cloned().map(|id| id.next_derived()) {
            Some(v) => v.ok_or(UnitDefinitionError::NoSpace)?,
            None => UnitId::first_derived(),
        };

        self.add_derived(next_id, desc)?;

        Ok(next_id)
    }

    fn add_derived(&mut self, id: UnitId, desc: UnitDefinition) -> Result<(), UnitDefinitionError> {
        if self.derived.contains_key(&id) {
            return Err(UnitDefinitionError::NoSpace);
        };

        for (_, udesc) in self.iter() {
            for (_, this_sym) in desc.as_prefixed_options() {
                for (_, existing_sym) in udesc.as_prefixed_options() {
                    if desc.is_degreed() == udesc.is_degreed() && this_sym == existing_sym {
                        return Err(UnitDefinitionError::SymbolDuplication);
                    }
                }
            }
        }

        self.derived.insert(id, desc);

        Ok(())
    }

    /// Add a name to a unit on another language
    pub fn add_name(&mut self, unit: UnitId, name: &str, lang: Language) -> Result<(), UnitDefinitionError> {
        let unit_def = &mut self.index_mut(unit);
        unit_def.names.name.insert(lang, Name::new(name).ok_or(UnitDefinitionError::IncorrectName)?);
        Ok(())
    }

    /// Add the description to a unit on another language
    pub fn add_description(&mut self, unit: UnitId, description: String, lang: Language) {
        let unit_def = &mut self.index_mut(unit);
        unit_def.names.description.insert(lang, description.into());
    }

    pub(super) fn get_type(&self, id: UnitId) -> UnitType {
        match self[id].kind {
            UnitKind::Absolute(_) => UnitType::Absolute(AbsoluteUnitId::new_proven(id)),
            UnitKind::Relative(_) => UnitType::Relative(RelativeUnitId::new_proven(id)),
            UnitKind::LogDimensionless(_) => UnitType::LogRatio(LogRatioUnitId::new_proven(id)),
            UnitKind::LogPoint(_) => UnitType::LogPoint(LogPointUnitId::new_proven(id)),
        }
    }

    /// Get unit by the symbol (only root, no prefixes)
    pub fn get_by_symbol(&self, symbol: &str, degreed: bool) -> Option<(UnitId, &UnitDefinition)> {
        self.iter().find(|(_, def)| def.is_degreed() == degreed && &*def.names.symbol == symbol)
    }

    /// Enumerate all units, base and derived
    pub fn iter(&self) -> impl Iterator<Item=(UnitId, &UnitDefinition)> {
        self.base.iter().enumerate().map(|(i, desc)| (UnitId::base(i as _), desc)).chain(self.derived.iter().map(|(&uid, desc)| (uid, desc)))
    }

    fn index_mut(&mut self, id: UnitId) -> &mut UnitDefinition {
        if id.0 < MAX_BASE_UNITS as u16 {  &mut self.base[id.0 as usize] }
        else { self.derived.get_mut(&id).unwrap() }
    }
}

impl std::ops::Index<UnitId> for UnitSystem {
    type Output = UnitDefinition;

    fn index(&self, id: UnitId) -> &Self::Output {
        if id.0 < MAX_BASE_UNITS as u16 {  &self.base[id.0 as usize] }
        else { &self.derived[&id] }
    }
}

impl std::ops::Index<AbsoluteUnitId> for UnitSystem {
    type Output = AbsoluteUnitDefinition;

    fn index(&self, id: AbsoluteUnitId) -> &Self::Output {
        let UnitKind::Absolute(ret) = &self[id.unitid()].kind else {panic!("AbsoluteUnitId corresponds to non-absolute unit")};
        ret
    }
}

impl std::ops::Index<RelativeUnitId> for UnitSystem {
    type Output = RelativeUnitDefinition;

    fn index(&self, id: RelativeUnitId) -> &Self::Output {
        let UnitKind::Relative(ret) = &self[id.unitid()].kind else {panic!("RelativeUnitId corresponds to non-relative unit")};
        ret
    }
}

impl std::ops::Index<LogRatioUnitId> for UnitSystem {
    type Output = LogRatioUnitDefinition;

    fn index(&self, id: LogRatioUnitId) -> &Self::Output {
        let UnitKind::LogDimensionless(ret) = &self[id.unitid()].kind else {panic!("LogRatioUnitId corresponds to non-logratio unit")};
        ret
    }
}

impl std::ops::Index<LogPointUnitId> for UnitSystem {
    type Output = LogPointUnitDefinition;

    fn index(&self, id: LogPointUnitId) -> &Self::Output {
        let UnitKind::LogPoint(ret) = &self[id.unitid()].kind else {panic!("LogPointUnitId corresponds to non-logpoint unit")};
        ret
    }
}


impl std::ops::Index<ExpressionUnitId> for UnitSystem {
    type Output = Quantity;

    fn index(&self, id: ExpressionUnitId) -> &Self::Output {
        match &self[id.unitid()].kind {
            UnitKind::Absolute(AbsoluteUnitDefinition { scale, allows_prefixes:_ }) => scale,
            UnitKind::Relative(RelativeUnitDefinition { scale, offset:_ }) => scale,
            _ => panic!("ExpressionUnitId corresponds to non-expression unit")
        }
    }
}