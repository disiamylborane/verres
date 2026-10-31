//! Implementation of unit systems
//! 
//! The unit system is defined by the [`UnitSystem`] structure. It contains base and
//! derived units. The base units are the primary ones, they are absolute, proportional
//! and linearly independent. Every numeric value may be expressed as a number and a
//! product of base units powers. The amount of base units is limited to
//! [`unitid::MAX_BASE_UNITS`], and typically equals 5: s, m, g, C and K.
//! 
//! The units allow prefixes, if explicitly specified. So, in addition to m, we have
//! km, Mm, ... and vice versa - cm, mm, μm, and so on. The prefixes are defined by
//! [`Prefix`].
//! 
//! The derived units may be one of the following kinds:
//! - [`UnitKind::Absolute`] - the regular absolute unit of measurement which allows
//!   multiplication and division. All the base units are absolute.
//! - [`UnitKind::Relative`] - the offset unit, like Celsius degrees or European shoe sizes.
//!   They are denoted by the degree symbol either before or after the unit: °C or C°. The
//!   degree-prefixed unit is a Point (100 °C = 373.15 K), the degree-suffixed one is a
//!   distance between points (100 C° = 100 K). Points may only exist alone, distances may
//!   be parts of unit power expressions (C° / s).
//! - [`UnitKind::LinearRatio`] - percents, ppms, halves and other fractions of another value.
//!   Actually dimensionless. Typically used with special math operators ⨣ and -&#x0302;-,
//!   where a ⨣ b = a * (1 + b), a -&#x0302;- b = a * (1 - b)
//! - [`UnitKind::LogDimensionless`] - decibels, octaves and other units of logarithmic ratios.
//!   Either exists alone and in that case it is dimensioness, or as a combination of log ratio
//!   value per unit power expression denoted with // operator (dB // s, dB // m2 s-1), which
//!   means the geometric progression increase over some unit. The units are used with the operators
//!   ** (power) and // (root).
//! - [`UnitKind::LogPoint`] - a point unit expressed in log scale (like dB SPL). Always exist alone.
//!   Actually just a representation for some absolute value.

mod unitid;
pub use unitid::UnitId;

mod dimensionality;
pub use dimensionality::Dimensionality;

mod quantity;
pub use quantity::Quantity;

mod prefix;
pub use prefix::Prefix;

mod definition;
pub use definition::{UnitDefinition, BaseUnitDefinition, UnitKind};

mod unit_system;
pub use unit_system::{UnitSystem};

mod atom;
pub use atom::{SymbolicUnitElement, AtomicUnit, AtomicComposeableUnit};
