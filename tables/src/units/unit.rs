
use core::cmp::Ordering;

use numbers::{Number, Rational};

use crate::units::{Dimensionality, Prefix, Quantity, definition::{LogPointUnitDefinition, LogRatioUnitDefinition, RelativeUnitDefinition, as_prefixed_options}, unit_system::UnitType, unitid::{ExpressionUnitId, LogPointUnitId, LogRatioUnitId, RelativeUnitId}};

use super::{UnitId, UnitSystem};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParsedUnitAtom {
    Expression{prefix: Prefix, root: ExpressionUnitId},
    LinPoint{root: RelativeUnitId},
    LogRatio{prefix: Prefix, root: LogRatioUnitId},
    LogPoint{prefix: Prefix, root: LogPointUnitId},
}


impl ParsedUnitAtom {
    pub fn parse(symbol: &str, units: &UnitSystem) -> Result<Self, UnitParsingError> {
        if let Some(prefixed) = symbol.strip_prefix(['°', '\'']) {
            for (prefix, sym) in as_prefixed_options(prefixed, true) {
                if let Some((root, def)) = units.get_by_symbol(sym, true) {
                    if let UnitType::Relative(uroot) = units.get_type(root) {
                        return if def.allows_prefixes() || prefix == Prefix::absent() {
                            Ok(Self::LinPoint { root: uroot })
                        }
                        else {Err(UnitParsingError::MustNotHavePrefix(root))}
                    }
                    else {unreachable!()}
                }
            }
            Err(UnitParsingError::SymbolNotFound(symbol.to_owned()))
        }
        else if let Some(suffixed) = symbol.strip_suffix(['°', '\'']) {
            for (prefix, sym) in as_prefixed_options(suffixed, true) {
                if let Some((root, def)) = units.get_by_symbol(sym, true) {
                    if let UnitType::Relative(uroot) = units.get_type(root) {
                        return if def.allows_prefixes() || prefix == Prefix::absent() {
                            Ok(Self::Expression { prefix, root: ExpressionUnitId::new_relative(uroot) })
                        }
                        else {Err(UnitParsingError::MustNotHavePrefix(root))}
                    }
                    else {unreachable!()}
                }
            }
            Err(UnitParsingError::SymbolNotFound(symbol.to_owned()))
        }
        else {
            for (prefix, sym) in as_prefixed_options(symbol, true) {
                if let Some((root, def)) = units.get_by_symbol(sym, false) {
                    if !def.allows_prefixes() && prefix != Prefix::absent() {
                        return Err(UnitParsingError::MustNotHavePrefix(root))
                    }

                    return match units.get_type(root) {
                        UnitType::Absolute(root) => 
                            Ok(Self::Expression { prefix, root: ExpressionUnitId::new_absolute(root) }),
                        UnitType::Relative(_) => unreachable!(),
                        UnitType::LogRatio(root) => Ok(Self::LogRatio { prefix, root }),
                        UnitType::LogPoint(root) => Ok(Self::LogPoint { prefix, root }),
                    }
                }
            }
            Err(UnitParsingError::SymbolNotFound(symbol.to_owned()))
        }
    }
}


/// Any unit of measurement, including unit expressions
#[derive(Clone, Debug)]
pub struct Unit(pub(super) UnitVariant);


#[derive(Debug)]
/// Numeric value expressed with some specified unit
pub struct Value {
    /// Numeric value
    pub num: Number,

    /// Unit of measurement
    pub unit: Unit
}


#[derive(Clone, Debug)]
pub(super) enum UnitVariant {
    Product{items: Vec<(Prefix, ExpressionUnitId, Rational)>},
    LinearPoint(RelativeUnitId),
    Geometric{dimensionless: Option<(Prefix, ExpressionUnitId)>, over: Vec<(Prefix, ExpressionUnitId, Rational)>},
    GeometricLog{dimensionless: (Prefix, LogRatioUnitId), over: Vec<(Prefix, ExpressionUnitId, Rational)>},
    LogPoint(Prefix, LogPointUnitId),
}


#[derive(Debug)]
pub enum UnitParsingError {
    IllegalAtomic(ParsedUnitAtom),
    MustBeDimensionless(ParsedUnitAtom),
    MustNotHavePower(ParsedUnitAtom),
    IllegalLogRatio(ParsedUnitAtom),
    MustBeDegreed,
    MustNotBeDegreed,
    MustNotHavePrefix(UnitId),
    SymbolNotFound(String),
}

#[derive(Debug)]
pub enum ValueError {
    DegreesMayNotPrefix,
    DegreeMustBeBefore,
    WrongPointUnit,
    WrongComposeableUnit(UnitId),
    DisallowsPrefixes(UnitId),
    MustBeDimensionless,
    MayNotDegree,
    WrongLogRatioUnit,
    DimensionalityMismatch
}


#[derive(Debug)]
pub enum ArithmeticError {
    DimensionalityMismatch,
    CannotApplyToUnit,
}




impl Unit {
    /// Dimensionless unit
    pub fn one() -> Self {Self(UnitVariant::Product { items: vec![] })}

    fn parse_ordinary(elements: Vec<(ParsedUnitAtom, Rational)>) -> Result<Vec<(Prefix, ExpressionUnitId, Rational)>, UnitParsingError> {
        elements.into_iter()
            .map(|v| {
                if let ParsedUnitAtom::Expression { prefix, root } = v.0 {
                    Ok((prefix, root, v.1))
                }
                else {Err(UnitParsingError::IllegalAtomic(v.0))}
            })
            .collect::<Result<Vec<_>,_>>()
    }

    /// Parse the unit as regular expression
    pub fn parse_expr(elements: Vec<(ParsedUnitAtom, Rational)>) -> Result<Self, UnitParsingError> {
        match TryInto::<[(ParsedUnitAtom, Rational); 1]>::try_into(elements) {
            Ok([(sym, power)]) => {
                if power == Rational::ONE {
                    match sym {
                        ParsedUnitAtom::Expression { prefix, root } => {
                            Ok(Self(UnitVariant::Product { items: vec![(prefix, root, Rational::ONE)] }))
                        }
                        ParsedUnitAtom::LinPoint { root } => Ok(Unit(UnitVariant::LinearPoint(root))),
                        ParsedUnitAtom::LogRatio { prefix, root } => Ok(Unit(UnitVariant::GeometricLog {dimensionless: (prefix, root), over: vec![]})),
                        ParsedUnitAtom::LogPoint { prefix, root } => Ok(Unit(UnitVariant::LogPoint(prefix, root))),
                    }
                }
                else {
                    match sym {
                        ParsedUnitAtom::Expression { prefix, root } => {
                            Ok(Unit(UnitVariant::Product{items: vec![(prefix, root, power)]}))
                        }
                        _ => Err(UnitParsingError::MustNotHavePower(sym)),
                    }
                }
            },
            Err(v) => {
                Ok(Unit(UnitVariant::Product{items: Self::parse_ordinary(v)?}))
            }
        }
    }

    /// Parse the unit as LogScale // expression syntax
    pub fn parse_over_logslash(dimensionless: Option<ParsedUnitAtom>, elements: Vec<(ParsedUnitAtom, Rational)>) -> Result<Self, UnitParsingError> {
        let over = Self::parse_ordinary(elements)?;
        match dimensionless {
            Some(ParsedUnitAtom::Expression { prefix, root }) => {
                Ok(Self(UnitVariant::Geometric{dimensionless: Some((prefix, root)), over}))
            }
            Some(ParsedUnitAtom::LogRatio { prefix, root }) => {
                Ok(Self(UnitVariant::GeometricLog{dimensionless: (prefix, root), over}))
            }
            None => Ok(Self(UnitVariant::Geometric { dimensionless: None, over })),
            Some(dimensionless) => Err(UnitParsingError::IllegalAtomic(dimensionless)),
        }
    }

    printer!{
        /// Print as expression
        repr (this = &Unit, units: &UnitSystem, f) {
        match &this.0 {
            &UnitVariant::LinearPoint(id) => {
                write!(f, "°{}", units[id.unitid()].names.symbol)
            }
            UnitVariant::LogPoint(prefix, id) => {
                write!(f, "{}{}", prefix.name(), units[id.unitid()].names.symbol)
            }
            UnitVariant::Product{items} => {
                let mut space = false;

                for (prefix, id, pow) in items.iter().filter(|p| p.2.cmp0() == Ordering::Greater) {
                    if space {write!(f, " ")?;}
                    let def = &units[id.unitid()];
                    write!(f, "{}{}{}", prefix.name(), def.names.symbol, if def.is_degreed() {"°"} else {""})?;
                    if pow != &Rational::ONE {
                        write!(f, "{pow}")?;
                    }
                    space = true;
                }

                for (prefix, id, pow) in items.iter().filter(|p| p.2.cmp0() == Ordering::Less) {
                    if space {write!(f, " ")?;}
                    let def = &units[id.unitid()];
                    write!(f, "{}{}{}", prefix.name(), def.names.symbol, if def.is_degreed() {"°"} else {""})?;
                    write!(f, "{pow}")?;
                    space = true;
                }

                Ok(())
            }
            UnitVariant::Geometric { dimensionless, over } => {
                let mut space = false;
                if let Some((prefix, id)) = dimensionless {
                    write!(f, "{}{}", prefix.name(), units[id.unitid()].names.symbol)?;
                    space = true;
                }
                if !over.is_empty() {
                    if space {write!(f, " ")?;}
                    write!(f, "//")?;

                    for (prefix, id, pow) in over.iter().filter(|p| p.2.cmp0() == Ordering::Greater) {
                        write!(f, " ")?;
                        let def = &units[id.unitid()];
                        write!(f, "{}{}{}", prefix.name(), def.names.symbol, if def.is_degreed() {"°"} else {""})?;
                        if pow != &Rational::ONE {
                            write!(f, "{pow}")?;
                        }
                    }

                    for (prefix, id, pow) in over.iter().filter(|p| p.2.cmp0() == Ordering::Less) {
                        write!(f, " ")?;
                        let def = &units[id.unitid()];
                        write!(f, "{}{}{}", prefix.name(), def.names.symbol, if def.is_degreed() {"°"} else {""})?;
                        write!(f, "{pow}")?;
                    }
                }

                Ok(())
            }
            UnitVariant::GeometricLog { dimensionless, over } => {
                let (prefix, id) = dimensionless;
                write!(f, "{}{}", prefix.name(), units[id.unitid()].names.symbol)?;
                if !over.is_empty() {
                    write!(f, " ")?;
                    write!(f, "//")?;

                    for (prefix, id, pow) in over.iter().filter(|p| p.2.cmp0() == Ordering::Greater) {
                        write!(f, " ")?;
                    write!(f, "{}{}", prefix.name(), units[id.unitid()].names.symbol)?;
                        if pow != &Rational::ONE {
                            write!(f, "{pow}")?;
                        }
                    }

                    for (prefix, id, pow) in over.iter().filter(|p| p.2.cmp0() == Ordering::Less) {
                    write!(f, "{}{}", prefix.name(), units[id.unitid()].names.symbol)?;
                        if pow != &Rational::ONE {
                            write!(f, "{pow}")?;
                        }
                    }
                }

                Ok(())
            }
        }
    }}
}



/*fn atomic_combinable(at: ParsedUnitAtom, units: &UnitSystem) -> Option<(Prefix, ExpressionUnitId)> {
    let ParsedUnitAtom{prefix, root, degree} = at;
    match (degree, units.get_type(root)) {
        (ParsedDegree::None, UnitType::Absolute(root)) => {
            Some((prefix, ExpressionUnitId::new_absolute(root)))
        }
        (ParsedDegree::Postfix, UnitType::Relative(root)) => {
            Some((prefix, ExpressionUnitId::new_relative(root)))
        }
        _ => None
    }
}*/



impl Value {
    /// Create corresponding physical quantity
    pub fn quantity(&self, units: &UnitSystem) -> Result<Quantity, ValueError> {
        match &self.unit.0 {
            UnitVariant::Product { items: linear } => {
                Self::quantity_of_absolute(self.num.clone(), units, linear)
            }
            &UnitVariant::LinearPoint(root) => {
                let RelativeUnitDefinition{scale, offset} = &units[root];

                let absolute_coef = self.num.clone() * scale.value.clone() + offset.clone();
                Ok(Quantity {
                    value: absolute_coef,
                    dimensionality: scale.dimensionality.clone(),
                })
            }
            &UnitVariant::LogPoint(prefix, root) => {
                let LogPointUnitDefinition { zero_unit, log_base, allows_prefixes:_ } = &units[root];
                Ok(zero_unit.clone() * Quantity{value: log_base.clone().pown(prefix.meaning().into_number() * self.num.clone()), dimensionality: Dimensionality::DIMENSIONLESS})
            }
            UnitVariant::Geometric { dimensionless, over } => {
                let log_quantity = if let Some((prefix, root)) = *dimensionless {
                    let scale = &units[root];
                    scale.value.clone() * self.num.clone() * prefix.meaning().into_number()
                } else {self.num.clone()};

                let over_unit = Self::quantity_of_absolute(Rational::ONE.into_number(), units, over)?;
                let Quantity { value, dimensionality } = Quantity{
                    value: Rational::ONE.into_number(),
                    dimensionality: Dimensionality::DIMENSIONLESS
                } / over_unit;
                Ok(Quantity{value: log_quantity.pown(value), dimensionality})
            }
            UnitVariant::GeometricLog { dimensionless, over } => {
                let (prefix, root) = *dimensionless;
                let LogRatioUnitDefinition{power_per_unit, allows_prefixes:_} = &units[root];
                let log_quantity = power_per_unit.clone().pown(self.num.clone() * prefix.meaning().into_number());

                let over_unit = Self::quantity_of_absolute(Rational::ONE.into_number(), units, over)?;
                let Quantity { value, dimensionality } = Quantity{
                    value: Rational::ONE.into_number(),
                    dimensionality: Dimensionality::DIMENSIONLESS
                } / over_unit;
                Ok(Quantity{value: log_quantity.pown(value), dimensionality})
            }
        }
    }

    fn quantity_of_absolute(coef: Number, units: &UnitSystem, linear: &Vec<(Prefix, ExpressionUnitId, Rational)>) -> Result<Quantity, ValueError> {
        let mut accumulated_coef = coef;
        let mut accumulated_dims = Dimensionality::DIMENSIONLESS;

        for &(prefix, root, ref power) in linear {
            let scale = &units[root];

            accumulated_coef = accumulated_coef * (scale.value.clone() * prefix.meaning().into_number()).powr(power.clone());
            accumulated_dims = accumulated_dims * scale.dimensionality.pow(power);
        }

        Ok(Quantity { value: accumulated_coef, dimensionality: accumulated_dims })
    }


    /// Express physical quantity as number and powers of base units
    pub fn from_quantity_canonical(qty: Quantity, engine: &UnitSystem) -> Value {
        let mut linear_elements: Vec<(Prefix, ExpressionUnitId, Rational)> = Vec::new();

        let mut value = qty.value;
        for (root, exponent) in qty.dimensionality.iter_base_units(false) {
            let prefix = engine[root].allows_prefixes.unwrap_or(Prefix::absent());
            if prefix != Prefix::absent() {
                value = value / prefix.meaning().pow(exponent.clone());
            }
            linear_elements.push((prefix, ExpressionUnitId::new_absolute(root), exponent));
        }

        let unit = Unit(UnitVariant::Product { items: linear_elements });

        Value { num: value, unit }
    }


    /// Express physical quantity as number and powers of base units
    pub fn from_quantity(arg: Quantity, unit: Unit, units: &UnitSystem) -> Result<Value, ValueError> {
        match &unit.0 {
            UnitVariant::Product { items } => {
                let base_quantity = Self::quantity_of_absolute(Rational::ONE.into_number(), units, items)?;
                if base_quantity.dimensionality != arg.dimensionality {
                    return Err(ValueError::DimensionalityMismatch);
                }
                let num = arg.value / base_quantity.value;
                Ok(Value { num, unit })
            }
            &UnitVariant::LinearPoint(root) => {
                let RelativeUnitDefinition { scale, offset } = &units[root];
                let num = (arg.value - offset.clone()) / scale.value.clone();
                Ok(Value { num, unit })
            }
            &UnitVariant::LogPoint(prefix, root) => {
                let LogPointUnitDefinition { allows_prefixes, log_base, zero_unit } = &units[root];
                if allows_prefixes.is_none() && prefix != Prefix::absent() {
                    return Err(ValueError::DisallowsPrefixes(root.unitid()))
                }

                let Quantity { value, dimensionality } = arg / zero_unit.clone();
                if dimensionality != Dimensionality::DIMENSIONLESS {
                    return Err(ValueError::MustBeDimensionless);
                }
                if allows_prefixes.is_none() && prefix != Prefix::absent() {
                    return Err(ValueError::DegreesMayNotPrefix)
                }
                Ok( Value{num: Number::shrink(value.get_as_f64().log(log_base.get_as_f64()) / prefix.meaning().get_as_f64()), unit})
            }
            UnitVariant::Geometric { dimensionless, over } => {
                let over_quantity = Self::quantity_of_absolute(Rational::ONE.into_number(), units, over)?;
                let Quantity { value, dimensionality } = arg * over_quantity;
                if dimensionality != Dimensionality::DIMENSIONLESS {
                    return Err(ValueError::DimensionalityMismatch);
                }

                let num = if let &Some((prefix, root)) = dimensionless {
                    let scale = &units[root];
                    if scale.dimensionality != Dimensionality::DIMENSIONLESS {
                        return Err(ValueError::MustBeDimensionless)
                    }
                    value / scale.value.clone() / prefix.meaning().into_number()
                } else {value};

                Ok(Value { num, unit })
            }
            UnitVariant::GeometricLog { dimensionless, over } => {
                let over_quantity = Self::quantity_of_absolute(Rational::ONE.into_number(), units, over)?;
                let Quantity { value, dimensionality } = arg * over_quantity;
                if dimensionality != Dimensionality::DIMENSIONLESS {
                    return Err(ValueError::DimensionalityMismatch);
                }

                let (prefix, root) = *dimensionless;
                let LogRatioUnitDefinition { power_per_unit, allows_prefixes:_ } = &units[root];
                let num = Number::shrink(value.get_as_f64().log(power_per_unit.get_as_f64()) / prefix.meaning().get_as_f64());
                Ok(Value { num, unit })
            }
        }
    }

    /// Addition of two values
    pub fn add(self, rhs: Value, units: &UnitSystem) -> Result<Value, ArithmeticError> {
        match (&self.unit.0, &rhs.unit.0) {
            (UnitVariant::Product { items:_ }, UnitVariant::Product { items:_ }) => {
                let diml = Value::quantity(&self, units).unwrap();
                let dimr = Value::quantity(&rhs, units).unwrap();
                let result = (diml + dimr).ok_or(ArithmeticError::DimensionalityMismatch)?;
                Ok(Value::from_quantity(result, self.unit, units).unwrap())
            }
            (UnitVariant::Product { items:_ }, pt@(UnitVariant::LinearPoint(_) | UnitVariant::LogPoint(_, _)))
            | (pt@(UnitVariant::LinearPoint(_) | UnitVariant::LogPoint(_, _)), UnitVariant::Product { items:_ }) => {
                let upt = Unit(pt.clone());
                let diml = Value::quantity(&self, units).unwrap();
                let dimr = Value::quantity(&rhs, units).unwrap();
                let result = (diml + dimr).ok_or(ArithmeticError::DimensionalityMismatch)?;
                Ok(Value::from_quantity(result, upt, units).unwrap())
            }
            _ => Err(ArithmeticError::CannotApplyToUnit),
        }
    }

    /// Subtraction of value from value
    pub fn sub(self, rhs: Value, units: &UnitSystem) -> Result<Value, ArithmeticError> {
        let diml = Value::quantity(&self, units).unwrap();
        let dimr = Value::quantity(&rhs, units).unwrap();
        let result = (diml - dimr).ok_or(ArithmeticError::DimensionalityMismatch)?;
        match (&self.unit.0, &rhs.unit.0) {
            (UnitVariant::Product { items:_ }, UnitVariant::Product { items:_ }) => {
                Ok(Value::from_quantity(result, self.unit, units).unwrap())
            }
            (&UnitVariant::LinearPoint(_), UnitVariant::Product { items:_ }) => {
                Ok(Value::from_quantity(result, self.unit, units).unwrap())
            }
            (&UnitVariant::LinearPoint(relative), UnitVariant::LinearPoint(_)) => {
                Ok(Value::from_quantity(result, Unit(UnitVariant::Product { items: vec![(Prefix::absent(), ExpressionUnitId::new_relative(relative), Rational::ONE)] }), units).unwrap())
            }
            _ => Err(ArithmeticError::CannotApplyToUnit),
        }
    }

    /// Multiplication or division of two values
    pub fn mul_or_div(self, rhs: Value, invert: bool, units: &UnitSystem) -> Result<Value, ArithmeticError> {
        let vl = Value::quantity(&self, units).unwrap();
        let vr = Value::quantity(&rhs, units).unwrap();

        let unit = {
            match (self.unit.0, rhs.unit.0) {
                (UnitVariant::Product { items:litems }, UnitVariant::Product { items:ritems }) => {
                    let mut items: Vec<(Prefix, ExpressionUnitId, Rational)> = vec![];

                    for (prefix, root, power) in litems {
                        if let Some(entry) = items.iter_mut().find(|el| el.1 == root) {
                            entry.2 = entry.2.clone() + power;
                        }
                        else {
                            items.push((prefix, root, power));
                        }
                    }
                    for (prefix, root, power) in ritems {
                        let power = if invert {-power} else {power};
                        if let Some(entry) = items.iter_mut().find(|el| el.1 == root) {
                            entry.2 = entry.2.clone() + power;
                        }
                        else {
                            items.push((prefix, root, power));
                        }
                    }

                    Some(Unit(UnitVariant::Product { items }))
                }
                (UnitVariant::Product { items:_ }, i @ (UnitVariant::LinearPoint(_) | UnitVariant::LogPoint(_, _))) => {
                    if vl.dimensionality == Dimensionality::DIMENSIONLESS {Some(Unit(i))}
                    else {None}
                }
                (i @ (UnitVariant::LinearPoint(_) | UnitVariant::LogPoint(_, _)), UnitVariant::Product { items:_ }) => {
                    if vr.dimensionality == Dimensionality::DIMENSIONLESS {Some(Unit(i))}
                    else {None}
                }
                (UnitVariant::GeometricLog { dimensionless:_, over }, i @ (UnitVariant::LinearPoint(_) | UnitVariant::LogPoint(_, _) | UnitVariant::Product { items:_ }))
                | (i @ (UnitVariant::LinearPoint(_) | UnitVariant::LogPoint(_, _) | UnitVariant::Product { items:_ }), UnitVariant::GeometricLog { dimensionless:_, over }) => {
                    if over.is_empty() {Some(Unit(i))}
                    else {None}
                }
                (UnitVariant::GeometricLog { dimensionless, over:lover }, UnitVariant::GeometricLog { dimensionless:_, over:rover } | UnitVariant::Geometric { dimensionless:_, over:rover }) => {
                    if Self::quantity_of_absolute(Rational::ONE.into_number(), units, &lover).unwrap().dimensionality == Self::quantity_of_absolute(Rational::ONE.into_number(), units, &rover).unwrap().dimensionality {
                        Some(Unit(UnitVariant::GeometricLog { dimensionless, over:lover }))
                    }
                    else {None}
                }
                (UnitVariant::Geometric { dimensionless, over:lover }, UnitVariant::GeometricLog { dimensionless:_, over:rover } | UnitVariant::Geometric { dimensionless:_, over:rover }) => {
                    if Self::quantity_of_absolute(Rational::ONE.into_number(), units, &lover).unwrap().dimensionality == Self::quantity_of_absolute(Rational::ONE.into_number(), units, &rover).unwrap().dimensionality {
                        Some(Unit(UnitVariant::Geometric { dimensionless, over:lover }))
                    }
                    else {None}
                }
                _ => None,
            }
        };

        let result = if invert {vl / vr} else {vl * vr};
        Ok(Value::from_quantity(result, unit.ok_or(ArithmeticError::CannotApplyToUnit)?, units).unwrap())
    }

    printer!{
        /// Express as parseable
        repr(Value { num, unit } = &Value, units: &UnitSystem, f) {
            if matches!(&unit.0, UnitVariant::Product{items} if items.is_empty()) {
                write!(f, "{}", num)
            }
            else {
                write!(f, "{} {}", num, unit.repr(units))
            }
        }
    }
}

