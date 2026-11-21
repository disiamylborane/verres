
use tables::{strings::Localized, units::{AbsoluteUnitDefinition, BaseUnitDefinition, Dimensionality, LogPointUnitDefinition, LogRatioUnitDefinition, Prefix, Quantity, RelativeUnitDefinition, UnitDefinition, UnitKind, UnitNames}};

use super::*;



pub fn unitsystem() -> UnitSystem {
    let en = Language::new("en").unwrap();

    let mut units = UnitSystem::new(vec![
        BaseUnitDefinition{
            allows_prefixes: Some(Prefix::absent()),
            names: UnitNames {
                description: Localized::new(en, "The duration of exactly $9192631770$ periods of the radiation corresponding to the transition between the two hyperfine levels of the ground state of the unperturbed caesium-133 atom at rest at a thermodynamic temperature of absolute zero".into()),
                symbol: Symbol::new("s").unwrap(),
                name: Localized::new(en, Name::new("second").unwrap()),
            }
        },
        BaseUnitDefinition{
            allows_prefixes: Some(Prefix::absent()),
            names: UnitNames {
                description: Localized::new(en, "The length of the path travelled by light in vacuum during a time interval of exactly $frac(1, 299792458)$ of a second".into()),
                symbol: Symbol::new("m").unwrap(),
                name: Localized::new(en, Name::new("meter").unwrap()),
            }
        },
        BaseUnitDefinition{
            allows_prefixes: Some(Prefix::parse("k").unwrap()),
            names: UnitNames {
                description: Localized::new(en, "The mass of a particle whose de Broglie wavelength is exactly $6.62607015 dot.c 10^(-31)$ metres when it is moving at a speed of exactly $1$ metre per second".into()),
                symbol: Symbol::new("g").unwrap(),
                name: Localized::new(en, Name::new("gram").unwrap()),
            }
        },
        BaseUnitDefinition{
            allows_prefixes: Some(Prefix::absent()),
            names: UnitNames {
                description: Localized::new(en, "The electric charge of exactly $frac(1, 1.602176634 times 10^(-19))$ elementary charges".into()),
                symbol: Symbol::new("C").unwrap(),
                name: Localized::new(en, Name::new("coulomb").unwrap()),
            }
        },
        BaseUnitDefinition{
            allows_prefixes: Some(Prefix::absent()),
            names: UnitNames {
                description: Localized::new(en, "The thermodynamic temperature interval for which the mean kinetic energy per translational degree of freedom of an ideal-gas particle is exactly $6.903245 dot.c 10^(-24)$ kilogram metres squared per second squared".into()),
                symbol: Symbol::new("K").unwrap(),
                name: Localized::new(en, Name::new("kelvin").unwrap()),
            }
        },
    ]).unwrap();
    let s = || Dimensionality::unit(0);
    let m = || Dimensionality::unit(1);
    let g = || Dimensionality::unit(2);
    let c = || Dimensionality::unit(3);
    let k = || Dimensionality::unit(4);

    let sym = |str| Symbol::new(str).unwrap();
    let ls = |str| Localized::new(en, Name::new(str).unwrap());
    let ts = |str: &str| Localized::new(en, str.into());


    units.define_derived(UnitDefinition{
        names: UnitNames {
            symbol: Symbol::new("C").unwrap(),
            name: Localized::new(en, Name::new("Celsius degree").unwrap()),
            description: Localized::new(en, "the logarithmic unit of ratio, 2 times per unit".into()),
        },
        kind: UnitKind::Relative(RelativeUnitDefinition {
            scale: Quantity{value: Rational::ONE.into_number(), dimensionality: k()},
            offset: Rational::new_med(27315, 100).into_number(),
        })
    }).unwrap();

    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("F"),
        name: ls("Fahrenheit degree"),
        description: ts("A temperature scale where 32 represents the freezing point of water and 212 represents the boiling point."),
        },
        kind: UnitKind::Relative(RelativeUnitDefinition { scale: Quantity { value: Rational::new_small(5, 9).into_number(), dimensionality: k() }, offset: Rational::new_med(45967, 100).into_number() })
    }).unwrap();

    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("R"),
        name: ls("Rankine degree"),
        description: ts("An absolute thermodynamic temperature scale named after William John Macquorn Rankine."),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: Rational::new_small(5, 9).into_number(), dimensionality: k() }, allows_prefixes: None }),
    }).unwrap();

    units.define_derived(UnitDefinition{
        names: UnitNames {
        symbol: Symbol::new("octave").unwrap(),
        name: Localized::new(en, Name::new("Octave").unwrap()),
        description: Localized::new(en, "the logarithmic unit of ratio, 2 times per unit".into()),
        },
        kind: UnitKind::LogDimensionless(LogRatioUnitDefinition {
            power_per_unit: Rational::new_small(2, 1).into_number(),
            allows_prefixes: None
        }),
    }).unwrap();

    units.define_derived(UnitDefinition{
        names: UnitNames {
        symbol: Symbol::new("BE").unwrap(),
        name: Localized::new(en, Name::new("bel energy").unwrap()),
        description: Localized::new(en, "the logarithmic unit of ratio, 10 times per unit".into()),
        },
        kind: UnitKind::LogDimensionless(LogRatioUnitDefinition {
            power_per_unit: Rational::new_small(10, 1).into_number(),
            allows_prefixes: Some(Prefix::parse("d").unwrap())
        }),
    }).unwrap();

    units.define_derived(UnitDefinition{
        names: UnitNames {
        symbol: Symbol::new("BA").unwrap(),
        name: Localized::new(en, Name::new("bel amplitude").unwrap()),
        description: Localized::new(en, "the logarithmic unit of ratio, 20 times per unit".into()),
        },
        kind: UnitKind::LogDimensionless(LogRatioUnitDefinition {
            power_per_unit: Number::shrink(10.0f64.sqrt()),
            allows_prefixes: Some(Prefix::parse("d").unwrap())
        }),
    }).unwrap();

    units.define_derived(UnitDefinition{
        names: UnitNames {
            symbol: Symbol::new("BSPL").unwrap(),
            name: Localized::new(en, Name::new("bel Sound Pressure Level").unwrap()),
            description: Localized::new(en, "".into()),
        },
        kind: UnitKind::LogPoint(LogPointUnitDefinition {
            zero_unit: Quantity { value: Rational::new_small(2, 100).into_number(), dimensionality: g() / m() / s() / s() },
            log_base: Number::shrink(10.0f64.sqrt()),
            allows_prefixes: Some(Prefix::parse("d").unwrap()),
        }),
    }).unwrap();

    units.define_derived(UnitDefinition{
        names: UnitNames {
            symbol: Symbol::new("Bm").unwrap(),
            name: Localized::new(en, Name::new("Bel over milliwatt").unwrap()),
            description: Localized::new(en, "".into()),
        },
        kind: UnitKind::LogPoint(LogPointUnitDefinition {
            zero_unit: Quantity { value: Rational::new_med(1, 1_000_000).into_number(), dimensionality: g() * m() * m() / s() / s() / s() },
            log_base: Rational::new_small(1, 10).into_number(),
            allows_prefixes: Some(Prefix::parse("d").unwrap()),
        }),
    }).unwrap();

    units.define_derived(UnitDefinition{
        names: UnitNames {
        symbol: Symbol::new("%").unwrap(),
        name: Localized::new(en, Name::new("Percent").unwrap()),
        description: Localized::new(en, "One hundredth part (1/100)".into()),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: Rational::new_small(1, 100).into_number(), dimensionality: Dimensionality::DIMENSIONLESS }, allows_prefixes: None })
    }).unwrap();

    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: Symbol::new("permille").unwrap(), name: ls("Permille"), description: ts("One thousandth part (1/1,000)"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: Rational::new_small(1, 1000).into_number(), dimensionality: Dimensionality::DIMENSIONLESS }, allows_prefixes: None })
    }).unwrap();

    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: Symbol::new("ppm").unwrap(), name: ls("Parts per million"), description: ts("One millionth part (1/1,000,000)"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: Rational::new_med(1, 1_000_000).into_number(), dimensionality: Dimensionality::DIMENSIONLESS }, allows_prefixes: None })
    }).unwrap();

    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: Symbol::new("ppb").unwrap(), name: ls("Parts per billion"), description: ts("One billionth part (1/1,000,000,000) using the short scale"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: Rational::new_med(1, 1_000_000_000).into_number(), dimensionality: Dimensionality::DIMENSIONLESS }, allows_prefixes: None })
    }).unwrap();

    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: Symbol::new("ppt").unwrap(), name: ls("Parts per trillion"), description: ts("One trillionth part (1/1,000,000,000,000) using the short scale"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: Rational::new_med(1, 1_000_000_000_000).into_number(), dimensionality: Dimensionality::DIMENSIONLESS }, allows_prefixes: None })
    }).unwrap();

    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: Symbol::new("ppq").unwrap(), name: ls("Parts per quadrillion"), description: ts("One quadrillionth part (1/1,000,000,000,000,000) using the short scale"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: Rational::new_med(1, 1_000_000_000_000_000).into_number(), dimensionality: Dimensionality::DIMENSIONLESS }, allows_prefixes: None })
    }).unwrap();


    let one = Rational::ONE.into_number();
    let thsd = Rational::new_small(1000, 1).into_number();
    let thsdm1 = Rational::new_small(1, 1000).into_number();

    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("A"),
        name: ls("Ampere"),
        description: ts("A current of 1 Coloumb per second"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: one.clone(), dimensionality: c()/s()}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();

    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("J"),
        name: ls("Joule"),
        description: ts("An energy equal to $1 \"kg\" \"m\"^2/\"s\"^2$"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: thsd.clone(), dimensionality: g()*m()*m()/s()/s() }, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();
    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("N"),
        name: ls("Newton"),
        description: ts("A force equal to $1 \"kg\" \"m\"/\"s\"^2$"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: thsd.clone(), dimensionality: g()*m()/s()/s()}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();

    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("Hz"),
        name: ls("Hertz"),
        description: ts("A frequency of one cycle per second"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: one.clone(), dimensionality: s().pow(&-Rational::ONE)}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();
    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("Pa"),
        name: ls("Pascal"),
        description: ts("A pressure of 1 Newton per square metre"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: thsd.clone(), dimensionality: g()/m()/s()/s()}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();
    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("W"),
        name: ls("Watt"),
        description: ts("A power of 1 Joule per second"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: thsd.clone(), dimensionality: g()*m()*m()/s()/s()/s()}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();
    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("V"),
        name: ls("Volt"),
        description: ts("An electric potential difference of 1 Watt per Ampere"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: thsd.clone(), dimensionality: g()*m()*m()/s()/s()/c()}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();

    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("F"),
        name: ls("Farad"),
        description: ts("An electric capacitance of 1 Coulomb per Volt"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: thsdm1.clone(), dimensionality: c() / (g()*m()*m()/s()/s()/c())}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();
    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("Ω"),
        name: ls("Ohm"),
        description: ts("An electric resistance of 1 Volt per Ampere"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: thsd.clone(), dimensionality: g()*m()*m()/s()/c()/c()}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();
    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("S"),
        name: ls("Siemens"),
        description: ts("An electric conductance of 1 Ampere per Volt"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: thsdm1.clone(), dimensionality: (g()*m()*m()/s()/c()/c()).pow(&-Rational::ONE)}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();
    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("Wb"),
        name: ls("Weber"),
        description: ts("A magnetic flux that produces an electromotive force of 1 Volt in one second"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: thsd.clone(), dimensionality: g()*m()*m()/s()/s()/c()/s()}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();
    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("T"),
        name: ls("Tesla"),
        description: ts("A magnetic flux density of 1 Weber per square metre"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: thsd.clone(), dimensionality: g()/s()/s()/c()/s()}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();
    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("H"),
        name: ls("Henry"),
        description: ts("An inductance that produces an electromotive force of 1 Volt from a current change of 1 Ampere per second"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: thsd.clone(), dimensionality: g()*m()*m()/s()/s()/c()/c()}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();
    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("Bq"),
        name: ls("Becquerel"),
        description: ts("An activity of one nuclear decay per second"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: one.clone(), dimensionality: s().pow(&-Rational::ONE)}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();
    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("Gy"),
        name: ls("Gray"),
        description: ts("An absorbed dose of 1 Joule of energy per kilogram of matter"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: one.clone(), dimensionality: m()*m()/s()/s()}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();
    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("Sv"),
        name: ls("Sievert"),
        description: ts("A dose equivalent of 1 Joule of energy per kilogram of matter scaled by biological impact"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: one.clone(), dimensionality: m()*m()/s()/s()}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();

    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("L"),
        name: ls("Liter"),
        description: ts("A volume equal to $1 med \"dm\"^3$"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: thsdm1.clone(), dimensionality: m()*m()*m()}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();

    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("min"),
        name: ls("Minute"),
        description: ts("A time equal to 60 seconds"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: Rational::new_small(60, 1).into_number(), dimensionality: s()}, allows_prefixes: None }),
    }).unwrap();
    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("h"),
        name: ls("Hour"),
        description: ts("A time equal to 60 min"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: Rational::new_small(3600, 1).into_number(), dimensionality: s()}, allows_prefixes: None }),
    }).unwrap();
    units.define_derived(UnitDefinition {
        names: UnitNames {
        symbol: sym("t"),
        name: ls("Tonne"),
        description: ts("A metric ton, unit of mass equal to 1000 kg"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: Rational::new_med(1_000_000, 1).into_number(), dimensionality: g()}, allows_prefixes: None }),
    }).unwrap();

    units.define_derived(UnitDefinition {
        names: UnitNames {
            symbol: sym("e"),
            name: ls("Electron"),
            description: ts("An elementary charge"),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition { scale: Quantity { value: Rational::new_big(801088317, 5000000000000000000000000000).into_number(), dimensionality: c()}, allows_prefixes: Some(Prefix::absent()) }),
    }).unwrap();

    units.define_derived(UnitDefinition {
        names: UnitNames {
            symbol: sym("mol"),
            name: ls("Mole"),
            description: ts("The SI base unit for amount of substance, containing exactly 6.02214076e23 elementary entities."),
        },
        kind: UnitKind::Absolute(AbsoluteUnitDefinition {
            scale: Quantity {
                value: Rational::new_big(602214076000000000000000, 1).into_number(),
                dimensionality: Dimensionality::DIMENSIONLESS
            },
            allows_prefixes: None
        }),
    }).unwrap();


    units.define_derived(UnitDefinition{
        kind: UnitKind::Relative(RelativeUnitDefinition{
            offset: Rational::new_small(-1, 75).into_number(),
            scale: Quantity { value: Rational::new_small(1, 150).into_number(), dimensionality: m() }
        }),
        names: UnitNames {
            description: Localized::new(en, "European shoe size".into()),
            symbol: Symbol::new("szEU").unwrap(),
            name: Localized::new(en, Name::new("European shoe size").unwrap()),
        }
    }).unwrap();

    units
}