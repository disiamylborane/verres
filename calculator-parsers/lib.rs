
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

mod unitsystem;

use std::{collections::HashMap, ops::Deref, str::FromStr};
use numbers::{Number, Rational};
use tables::{strings::{Language, LanguageResolution, Name, Symbol}, units::{AbsoluteUnitDefinition, ArithmeticError, LogPointUnitDefinition, LogRatioUnitDefinition, ParsedUnitAtom, RelativeUnitDefinition, Unit, UnitParsingError, UnitSystem, Value, ValueError}};


#[derive(Debug)]
pub enum CalculatorError {
    UnitDoesNotExist(String, UnitParsingError),
    UnitParsingError(UnitParsingError),
    ValueError(ValueError),
    ArithmeticError(ArithmeticError),
    NoSuchVariable(Symbol),
}


pub enum Statement {
    Expression(Value),
    Assign(Symbol, Value),
    Info(ParsedUnitAtom),
    InfoAll,
}



peg::parser!{
    pub grammar calc_parser(c: &CalculatorObject, units: &UnitSystem, lr: &LanguageResolution) for str {

        rule _() = [' ' | '\t' | '\r' | '\n']*
        rule __() = [' ' | '\t' | '\r' | '\n']+


        pub rule natural() -> Rational
            = n:$(['0'..='9']+) {Rational::from_str(n).unwrap()}

        pub rule rational() -> Rational
            = n:natural() "/" d:natural() {n / d}
            / "-" n:natural() "/" d:natural() {-n / d}
            / n:natural() {n}
            / "-" n:natural() {-n}

        pub rule number() -> Number
            = rt:$("-"? ['0'..='9']+ !"/" ("." ['0'..='9']*)? ("e" ['+' | '-']? ['0'..='9']+)? ("~" (['1'..='9']['0'..='9']*)?)?)  {
                Number::from_str(rt).unwrap()
            }
            / r:rational() {r.into_number()}


        pub rule symbol() -> Symbol
            = s:$(['A'..='Z' | 'a'..='z' | 'Α'..='Ω' | 'α'..='ω' | '%']+) {Symbol::new(s).unwrap()}

        pub rule name() -> Name
            = s:$(['A'..='Z' | 'a'..='z' | 'Α'..='Ω' | 'α'..='ω']['A'..='Z' | 'a'..='z' | 'Α'..='Ω' | 'α'..='ω' | '0'..='9' | ' ' | '-']*) {
                Name::new(s.trim()).unwrap()
            }

        pub rule unit_atom() -> Result<ParsedUnitAtom, CalculatorError>
            = s:$(['°' | '\'']? s:symbol() ['°' | '\'']?) {
                ParsedUnitAtom::parse(s, units).map_err(|ue| CalculatorError::UnitDoesNotExist(s.to_string(), ue))
            }

        pub rule unit_and_power() -> Result<(ParsedUnitAtom, Rational), CalculatorError>
            = a:unit_atom() r:rational() {Ok((a?, r))}
            / a:unit_atom() {Ok((a?, Rational::ONE))}

        pub rule unit_chain() -> Result<Vec<(ParsedUnitAtom, Rational)>, CalculatorError>
            = u:unit_and_power() nxt:(__ u:unit_and_power(){u})+ {[u].into_iter().chain(nxt.into_iter()).collect::<Result<_, _>>()}
            / u:unit_and_power() {Ok(vec![u?])}


        pub rule unit() -> Result<Unit, CalculatorError>
            = dl:unit_atom() _ "//" _ u:unit_chain() {
                Unit::parse_over_logslash(Some(dl?), u?)
                    .map_err(CalculatorError::UnitParsingError)
            }
            / u:unit_chain() {Unit::parse_expr(u?).map_err(CalculatorError::UnitParsingError)}
            / "//" _ u:unit_chain() {
                Unit::parse_over_logslash(None, u?)
                    .map_err(CalculatorError::UnitParsingError)
            }

        pub rule value() -> Result<Value, CalculatorError>
            = num:number() __ unit:unit() {Ok(Value { num, unit: unit? })}
            / num:number() {Ok(Value { num, unit: Unit::one() })}


        pub rule value_expr() -> Result<Value, CalculatorError> = precedence! {
            x:(@) _ "||" {
                Ok(Value::from_quantity_canonical(x?.quantity(units), units))
            }
            x:(@) _ "|" _ u:unit() {
                let q = x?.quantity(units);
                Value::from_quantity(q, u?, units).map_err(CalculatorError::ValueError)
            }
            x:(@) _ "|" {
                Value::from_quantity(x?.quantity(units), Unit::one(), units).map_err(CalculatorError::ValueError)
            }
            --
            x:(@) _ "+^" _ y:@ {
                x?.add_frac(y?, false, units).map_err(CalculatorError::ArithmeticError)
            }
            x:(@) _ "-^" _ y:@ {
                x?.add_frac(y?, true, units).map_err(CalculatorError::ArithmeticError)
            }
            x:(@) _ "+" _ y:@ {
                x?.add(y?, units).map_err(CalculatorError::ArithmeticError)
            }
            x:(@) _ "-" _ y:@ {
                x?.sub(y?, units).map_err(CalculatorError::ArithmeticError)
            }
            --
            x:(@) _ "**" _ y:@ {
                x?.powmul(y?, units).map_err(CalculatorError::ArithmeticError)
            }
            x:(@) _ "//" _ y:@ {
                x?.powover(y?, units).map_err(CalculatorError::ArithmeticError)
            }
            --
            x:(@) _ "*" _ y:@ {
                x?.mul_or_div(y?, false, units).map_err(CalculatorError::ArithmeticError)
            }
            x:(@) _ "/" _ y:@ {
                x?.mul_or_div(y?, true, units).map_err(CalculatorError::ArithmeticError)
            }
            --
            "_" var:symbol() {
                c.memory.get(&var).cloned().ok_or_else(|| CalculatorError::NoSuchVariable(var))
            }
            "_" {
                Ok(c.ans.clone())
            }
            at:value() { at }
            "(" _ x:value_expr() _ ")" {x}
        }

        pub rule statement() -> Result<Statement, CalculatorError> 
            = "_" var:symbol() _ "=" _ e:value_expr() {Ok(Statement::Assign(var, e?))}
            / "?" _ s:unit_atom() {Ok(Statement::Info(s?))}
            / "?" {Ok(Statement::InfoAll)}
            / e:value_expr() {Ok(Statement::Expression(e?))}
    }
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct CalculatorObject {
    units: UnitSystem,
    memory: HashMap<Symbol, Value>,
    ans: Value,
    lr: LanguageResolution,
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl CalculatorObject {
    #[allow(clippy::new_without_default)]
    #[cfg_attr(target_family = "wasm", wasm_bindgen(constructor))]
    pub fn new() -> Self {
        Self {
            units: unitsystem::unitsystem(),
            memory: Default::default(),
            lr: LanguageResolution { primary: Language::new("en").unwrap(), secondary: None },
            ans: Value { num: Rational::ZERO.into_number(), unit: Unit::one() },
        }
    }

    #[cfg_attr(target_family = "wasm", wasm_bindgen)]
    pub fn execute(&mut self, line: &str) -> String {
        let input = line.trim();
        if input.is_empty() {
            return "".to_string();
        }

        let v = &*self;
        let val = match calc_parser::statement(input, v, &self.units, &self.lr) {
            Ok(Ok(val)) => val,
            Ok(Err(ee)) => {return format!("Evaluation error: {ee:?}")}
            Err(se) => {return format!("Syntax error: {se:?}")}
        };

        match val {
            Statement::Expression(value) => {
                self.ans = value;
                format!("{}", self.ans.repr(&self.units))
            }
            Statement::Assign(symbol, value) => {
                self.memory.insert(symbol, value);
                "".to_string()
            }
            Statement::Info(unit) => {
                use core::fmt::Write;

                let root = match unit {
                    ParsedUnitAtom::Expression { prefix:_, root } => root.unitid(),
                    ParsedUnitAtom::LinPoint { root } => root.unitid(),
                    ParsedUnitAtom::LogRatio { prefix:_, root } => root.unitid(),
                    ParsedUnitAtom::LogPoint { prefix:_, root } => root.unitid(),
                };

                let desc = &self.units[root];
                let mut ret = String::with_capacity(256);
                writeln!(&mut ret, "{}", desc.names.name.get(&self.lr).str).unwrap();
                writeln!(&mut ret, "{}", desc.names.description.get(&self.lr).str).unwrap();
                match &desc.kind {
                    tables::units::UnitKind::Absolute(AbsoluteUnitDefinition{scale, allows_prefixes:_}) => 
                        write!(&mut ret, "1 {} = {}", desc.names.symbol, Value::from_quantity_canonical(scale.clone(), &self.units).repr(&self.units)),
                    tables::units::UnitKind::Relative(RelativeUnitDefinition{scale, offset}) => {
                        let unit = Value::from_quantity_canonical(scale.clone(), &self.units);
                        write!(&mut ret, "0 °{} = {} {}\n1 {}° = {}", desc.names.symbol, offset, unit.unit.repr(&self.units), desc.names.symbol, unit.repr(&self.units))
                    }
                    tables::units::UnitKind::LogDimensionless(LogRatioUnitDefinition{allows_prefixes:_, power_per_unit}) => {
                        write!(&mut ret, "*1 {} = *{}", desc.names.symbol, power_per_unit)
                    }
                    tables::units::UnitKind::LogPoint(LogPointUnitDefinition{allows_prefixes:_, log_base, zero_unit}) => {
                        write!(&mut ret, "0 {} = {}\n+1 {} = *{}", desc.names.symbol, Value::from_quantity_canonical(zero_unit.clone(), &self.units).repr(&self.units), desc.names.symbol, log_base)
                    }
                }.unwrap();
                ret
            }
            Statement::InfoAll => {
                self.units.iter().map(|u| u.1.names.symbol.deref()).collect::<Vec<_>>().join(" ")
            }
        }
    }
}

