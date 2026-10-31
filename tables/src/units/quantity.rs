
use numbers::{Number, Rational};

use super::{dimensionality::Dimensionality};

/// A specific value with dimensionality not expressed in any specific units
#[derive(Clone, Debug)]
pub struct Quantity {
    /// Value expressed in natural powers of base units
    pub value: Number,

    /// Dimensionality of the value
    pub dimensionality: Dimensionality,
}

impl core::ops::Add<Quantity> for Quantity {
    type Output = Option<Quantity>;
    fn add(self, rhs: Quantity) -> Self::Output {
        if self.dimensionality == rhs.dimensionality {
            Some(Self{value: self.value + rhs.value, dimensionality: self.dimensionality})
        }
        else {None}
    }
}

impl core::ops::Sub<Quantity> for Quantity {
    type Output = Option<Quantity>;
    fn sub(self, rhs: Quantity) -> Self::Output {
        if self.dimensionality == rhs.dimensionality {
            Some(Self{value: self.value - rhs.value, dimensionality: self.dimensionality})
        }
        else {None}
    }
}

impl core::ops::Mul<Quantity> for Quantity {
    type Output = Quantity;
    fn mul(self, rhs: Quantity) -> Self::Output {
        Self{value: self.value * rhs.value, dimensionality: self.dimensionality * rhs.dimensionality}
    }
}

impl core::ops::Div<Quantity> for Quantity {
    // TODO: prevent division by zero?
    type Output = Quantity;
    fn div(self, rhs: Quantity) -> Self::Output {
        Self{value: self.value / rhs.value, dimensionality: self.dimensionality / rhs.dimensionality}
    }
}

impl Quantity {
    /// Take a power of a quantity
    pub fn pow(self, r: Rational) -> Quantity {
        let dimensionality = self.dimensionality.pow(&r);
        Self{value: self.value.powr(r), dimensionality}
    }
}
