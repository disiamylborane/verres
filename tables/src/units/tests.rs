use std::assert_matches;

use numbers::{Number, Rational};

use crate::{
    strings::{Language, Localized, Name, Symbol}, units::{BaseUnitDefinition, Dimensionality, Prefix, Quantity, Unit, UnitDefinition, UnitId, UnitSystem, Value, definition::{AbsoluteUnitDefinition, LogPointUnitDefinition, LogRatioUnitDefinition, RelativeUnitDefinition, UnitNames}, unit::{ParsedUnitAtom, UnitVariant}
    }
};


#[test]
pub fn test_quantities() {
    let units = testing_system();

    let m = ParsedUnitAtom::parse("m", &units).unwrap();
    let g = ParsedUnitAtom::parse("g", &units).unwrap();
    let prc = ParsedUnitAtom::parse("%", &units).unwrap();
    let sz_eu_pt = ParsedUnitAtom::parse("'szEU", &units).unwrap();
    let sz_eu_rg = ParsedUnitAtom::parse("szEU'", &units).unwrap();
    let dbe = ParsedUnitAtom::parse("dBE", &units).unwrap();
    let be = ParsedUnitAtom::parse("BE", &units).unwrap();
    let dbm = ParsedUnitAtom::parse("dBm", &units).unwrap();

    let unit = Unit::parse_expr(vec![(sz_eu_pt, Rational::ONE)]).unwrap();
    let value = Value{num: Rational::ZERO.into_number(), unit: unit.clone()};
    let canonical = Value::from_quantity_canonical(value.quantity(&units).unwrap(), &units);
    assert_eq!(canonical.repr(&units).to_string(), "-1/75 m");
    let reeval = Value::from_quantity(value.quantity(&units).unwrap(), unit, &units).unwrap();
    assert_eq!(reeval.repr(&units).to_string(), "0 °szEU");

    let unit = Unit::parse_expr(vec![(sz_eu_pt, Rational::ONE)]).unwrap();
    let value = Value{num: Rational::new_small(38, 1).into_number(), unit: unit.clone()};
    let canonical = Value::from_quantity_canonical(value.quantity(&units).unwrap(), &units);
    assert_eq!(canonical.repr(&units).to_string(), "6/25 m");
    let reeval = Value::from_quantity(value.quantity(&units).unwrap(), unit, &units).unwrap();
    assert_eq!(reeval.repr(&units).to_string(), "38 °szEU");

    let unit = Unit::parse_expr(vec![(sz_eu_rg, Rational::ONE)]).unwrap();
    let value = Value{num: Rational::new_small(6, 1).into_number(), unit: unit.clone()};
    let canonical = Value::from_quantity_canonical(value.quantity(&units).unwrap(), &units);
    assert_eq!(canonical.repr(&units).to_string(), "1/25 m");
    let reeval = Value::from_quantity(value.quantity(&units).unwrap(), unit, &units).unwrap();
    assert_eq!(reeval.repr(&units).to_string(), "6 szEU°");

    let unit = Unit::parse_expr(vec![(g, Rational::ONE)]).unwrap();
    let value = Value{num: Rational::new_small(6000, 1).into_number(), unit: unit.clone()};
    let canonical = Value::from_quantity_canonical(value.quantity(&units).unwrap(), &units);
    assert_eq!(canonical.repr(&units).to_string(), "6 kg");
    let reeval = Value::from_quantity(value.quantity(&units).unwrap(), unit, &units).unwrap();
    assert_eq!(reeval.repr(&units).to_string(), "6000 g");

    let unit = Unit::parse_expr(vec![(prc, Rational::ONE)]).unwrap();
    let value = Value{num: Rational::new_small(6, 1).into_number(), unit: unit.clone()};
    let canonical = Value::from_quantity_canonical(value.quantity(&units).unwrap(), &units);
    assert_eq!(canonical.repr(&units).to_string(), "3/50");
    let reeval = Value::from_quantity(value.quantity(&units).unwrap(), unit, &units).unwrap();
    assert_eq!(reeval.repr(&units).to_string(), "6 %");

    let unit = Unit::parse_expr(vec![(dbm, Rational::ONE)]).unwrap();
    let value = Value{num: Rational::new_small(40, 1).into_number(), unit: unit.clone()};
    let canonical = Value::from_quantity_canonical(value.quantity(&units).unwrap(), &units);
    assert_eq!(canonical.num.get_as_f64().round(), 100.0);
    let reeval = Value::from_quantity(value.quantity(&units).unwrap(), unit, &units).unwrap();
    assert_eq!(reeval.repr(&units).to_string(), "40 dBm");

    let unit = Unit::parse_expr(vec![(dbe, Rational::ONE)]).unwrap();
    let value = Value{num: Rational::new_small(40, 1).into_number(), unit: unit.clone()};
    let canonical = Value::from_quantity_canonical(value.quantity(&units).unwrap(), &units);
    assert_eq!(canonical.num.get_as_f64().round(), 10000.0);
    let reeval = Value::from_quantity(value.quantity(&units).unwrap(), unit, &units).unwrap();
    assert_eq!(reeval.repr(&units).to_string(), "40 dBE");

    let unit = Unit::parse_over_logslash(None, vec![(m, Rational::ONE)]).unwrap();
    let value = Value{num: Rational::new_small(40, 1).into_number(), unit: unit.clone()};
    let canonical = Value::from_quantity_canonical(value.quantity(&units).unwrap(), &units);
    assert_eq!(canonical.num.get_as_f64().round(), 40.0);
    let reeval = Value::from_quantity(value.quantity(&units).unwrap(), unit, &units).unwrap();
    assert_eq!(reeval.repr(&units).to_string(), "40 // m");

    let unit = Unit::parse_over_logslash(Some(dbe), vec![(m, Rational::ONE)]).unwrap();
    let value = Value{num: Rational::new_small(40, 1).into_number(), unit: unit.clone()};
    let canonical = Value::from_quantity_canonical(value.quantity(&units).unwrap(), &units);
    assert_eq!(canonical.num.get_as_f64().round(), 10000.0);
    let reeval = Value::from_quantity(value.quantity(&units).unwrap(), unit, &units).unwrap();
    assert_eq!(reeval.repr(&units).to_string(), "40 dBE // m");
    let reeval = Value::from_quantity(value.quantity(&units).unwrap(), Unit::parse_over_logslash(Some(be), vec![(m, Rational::ONE)]).unwrap(), &units).unwrap();
    assert_eq!(reeval.repr(&units).to_string(), "4 BE // m");
}


#[test]
pub fn test_unitsystem() {
    let units = testing_system();

    let m = ParsedUnitAtom::parse("m", &units).unwrap();
    let kg = ParsedUnitAtom::parse("kg", &units).unwrap();
    let prc = ParsedUnitAtom::parse("%", &units).unwrap();

    ParsedUnitAtom::parse("szEU", &units).unwrap_err();
    let sz_eu_pt = ParsedUnitAtom::parse("'szEU", &units).unwrap();
    let sz_eu_range = ParsedUnitAtom::parse("szEU'", &units).unwrap();

    let dbe = ParsedUnitAtom::parse("dBE", &units).unwrap();
    let dbm = ParsedUnitAtom::parse("dBm", &units).unwrap();

    let m_unit = Unit::parse_expr(vec![(m, Rational::ONE)]).unwrap();
    let m2_unit = Unit::parse_expr(vec![(m, Rational::ONE + Rational::ONE)]).unwrap();
    let mkg_unit = Unit::parse_expr(vec![(m, Rational::ONE), (kg, Rational::ONE)]).unwrap();
    let mm1p_unit = Unit::parse_expr(vec![(m, -Rational::ONE), (prc, Rational::ONE)]).unwrap();

    Unit::parse_expr(vec![(sz_eu_pt, Rational::ONE), (prc, Rational::ONE)]).unwrap_err();
    Unit::parse_expr(vec![(sz_eu_pt, -Rational::ONE)]).unwrap_err();
    let szeu_unit = Unit::parse_expr(vec![(sz_eu_pt, Rational::ONE)]).unwrap();

    let one_over_m = Unit::parse_over_logslash(None, vec![(m, Rational::ONE)]).unwrap();
    let dbe_over_m = Unit::parse_over_logslash(Some(dbe), vec![(m, Rational::ONE)]).unwrap();

    Unit::parse_over_logslash(Some(dbm), vec![(m, Rational::ONE)]).unwrap_err();

    assert_matches!(&m_unit, Unit(UnitVariant::Product { items:_ }));
    assert_matches!(&m2_unit, Unit(UnitVariant::Product { items:_ }));
    assert_matches!(&mkg_unit, Unit(UnitVariant::Product { items:_ }));
    assert_matches!(&mm1p_unit, Unit(UnitVariant::Product { items:_ }));
    assert_matches!(&szeu_unit, Unit(UnitVariant::LinearPoint(_)));
    assert_matches!(&Unit::parse_expr(vec![(sz_eu_range, Rational::ONE)]).unwrap(), Unit(UnitVariant::Product { items:_ }));

    assert_matches!(&Unit::parse_expr(vec![(dbe, Rational::ONE)]).unwrap(), Unit(UnitVariant::GeometricLog { dimensionless: (pfx, _), over }) if over.is_empty() && pfx.name() == "d");
    assert_matches!(&Unit::parse_expr(vec![(dbm, Rational::ONE)]).unwrap(), Unit(UnitVariant::LogPoint(pfx, _)) if pfx.name() == "d");

    assert_matches!(&one_over_m, Unit(UnitVariant::Geometric { dimensionless: None, over }) if over.len() == 1);
    assert_matches!(&dbe_over_m, Unit(UnitVariant::GeometricLog { dimensionless: (pfx, _), over }) if over.len() == 1 && pfx.name() == "d");

    assert_eq!(m_unit.repr(&units).to_string(), "m");
    assert_eq!(m2_unit.repr(&units).to_string(), "m2");
    assert_eq!(mkg_unit.repr(&units).to_string(), "m kg");
    assert_eq!(mm1p_unit.repr(&units).to_string(), "% m-1");
    assert_eq!(one_over_m.repr(&units).to_string(), "// m");
    assert_eq!(dbe_over_m.repr(&units).to_string(), "dBE // m");
}


fn testing_system() -> UnitSystem {
    let en = Language::new("en").unwrap();

    let mut units = UnitSystem::new(vec![
        BaseUnitDefinition{
            allows_prefixes: Some(Prefix::absent()),
            names: UnitNames {
                description: Localized::new(en, "meter".into()),
                symbol: Symbol::new("m").unwrap(),
                name: Localized::new(en, Name::new("meter").unwrap()),
            }
        },
        BaseUnitDefinition{
            allows_prefixes: Some(Prefix::parse("k").unwrap()),
            names: UnitNames {
                description: Localized::new(en, "gram".into()),
                symbol: Symbol::new("g").unwrap(),
                name: Localized::new(en, Name::new("gram").unwrap()),
            }
        },
    ]).unwrap();

    let perc = units.define_derived(UnitDefinition{
        kind: super::UnitKind::Absolute(AbsoluteUnitDefinition{
            allows_prefixes: None,
            scale: super::Quantity { value: Rational::new_small(1, 100).into_number(), dimensionality: Dimensionality::DIMENSIONLESS }
        }),
        names: super::definition::UnitNames {
                description: Localized::new(en, "%".into()),
                symbol: Symbol::new("%").unwrap(),
                name: Localized::new(en, Name::new("Percent").unwrap()),
        }
    }).unwrap();

    units.define_derived(UnitDefinition{
        kind: super::UnitKind::Relative(RelativeUnitDefinition{
            offset: Rational::new_small(-1, 75).into_number(),
            scale: super::Quantity { value: Rational::new_small(1, 150).into_number(), dimensionality: Dimensionality::unit(0) }
        }),
        names: super::definition::UnitNames {
            description: Localized::new(en, "European shoe size".into()),
            symbol: Symbol::new("szEU").unwrap(),
            name: Localized::new(en, Name::new("European shoe size").unwrap()),
        }
    }).unwrap();

    units.define_derived(UnitDefinition{
        kind: super::UnitKind::LogDimensionless(LogRatioUnitDefinition{
            power_per_unit: Rational::new_small(10, 1).into_number(),
            allows_prefixes: Some(Prefix::parse("d").unwrap()),
        }),
        names: super::definition::UnitNames {
            description: Localized::new(en, "Bell energy".into()),
            symbol: Symbol::new("BE").unwrap(),
            name: Localized::new(en, Name::new("Bell energy").unwrap()),
        }
    }).unwrap();

    units.define_derived(UnitDefinition{
        kind: super::UnitKind::LogPoint(LogPointUnitDefinition{
            zero_unit: Quantity{value: Rational::new_small(1, 1).into_number(), dimensionality: Dimensionality::unit(0)},
            log_base: Number::shrink(10.0f64.sqrt()),
            allows_prefixes: Some(Prefix::parse("d").unwrap()),
        }),
        names: super::definition::UnitNames {
            description: Localized::new(en, "Bell amplitude over one meter".into()),
            symbol: Symbol::new("Bm").unwrap(),
            name: Localized::new(en, Name::new("Bell energy").unwrap()),
        }
    }).unwrap();

    units.add_name(UnitId::base(0), "Metr", Language::new("pl").unwrap()).unwrap();
    units.add_description(UnitId::base(0), "Metr".into(), Language::new("pl").unwrap());

    units.add_name(perc, "Procent", Language::new("pl").unwrap()).unwrap();
    units.add_description(perc, "Procent".into(), Language::new("pl").unwrap());

    units
}