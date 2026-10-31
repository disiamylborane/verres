use std::collections::HashMap;


use numbers::Rational;

use crate::{strings::{Language, Name, Symbol}, units::Quantity};

use super::{UnitDefinition, UnitId, UnitKind, Dimensionality, unitid::MAX_BASE_UNITS, BaseUnitDefinition};


/// The description for all units in scope
pub struct UnitSystem {
    base: Vec<UnitDefinition>,
    derived: HashMap<UnitId, UnitDefinition>,
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
    pub fn add_base(base_units: Vec<BaseUnitDefinition>) -> Result<Self, UnitDefinitionError> {
        if base_units.len() > MAX_BASE_UNITS as usize {
            Err(UnitDefinitionError::NoSpace)
        }
        else {
            for i in 0..base_units.len() {
                for j in (i+1)..base_units.len() {
                    if base_units[i].symbol == base_units[j].symbol {
                        return Err(UnitDefinitionError::SymbolDuplication)
                    }
                }
            }

            let mut base = vec![];
            for (i, baseunit) in base_units.into_iter().enumerate() {
                let mut dim = [Rational::ZERO; MAX_BASE_UNITS as usize];
                dim[i] = Rational::ONE;
                let dimensionality = Dimensionality::new(dim);
                let kind = UnitKind::Absolute {
                    scale: Quantity {
                        value: Rational::ONE.into_number(),
                        dimensionality
                    },
                    allow_prefixes: baseunit.allows_prefixes,
                };
                base.push(UnitDefinition {
                    symbol: Symbol::new(&baseunit.symbol).ok_or(UnitDefinitionError::IncorrectSymbol)?,
                    name: baseunit.name,
                    description: baseunit.description,
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
        unit_def.name.insert(lang, Name::new(name).ok_or(UnitDefinitionError::IncorrectName)?);
        Ok(())
    }

    /// Add the description to a unit on another language
    pub fn add_description(&mut self, unit: UnitId, description: String, lang: Language) {
        let unit_def = &mut self.index_mut(unit);
        unit_def.description.insert(lang, description.into());
    }

    /// Get unit by [`UnitId`]
    pub fn get(&self, id: UnitId) -> Option<&UnitDefinition> {
        if id.0 < MAX_BASE_UNITS as u16 {
            self.base.get(id.0 as usize)
        }
        else {
            self.derived.get(&id)
        }
    }

    /// Get unit by the symbol (only root, no prefixes)
    pub fn get_by_symbol(&self, symbol: &str, degreed: bool) -> Option<(UnitId, &UnitDefinition)> {
        self.iter().find(|(_, def)| def.is_degreed() == degreed && &*def.symbol == symbol)
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

