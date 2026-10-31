
use numbers::Rational;
use super::UnitId;

const MAX_BASE_UNITS: usize = super::unitid::MAX_BASE_UNITS as usize;

/// The nature of physical quantity expressed as a product of base units powers
/// 
/// The Dimensionality is an ordered set of powers of the base units. If all powers
/// are zero, it is [`Dimensionality::DIMENSIONLESS`].
pub struct Dimensionality(u64);


impl core::fmt::Debug for Dimensionality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let numbers = self.clone().into_numbers();
        write!(f, "Dimensionality::new({:?})", numbers)
    }
}

struct SevenRatiosBits(u64);


enum ExpandedDimensionality<'a> {
    Short(SevenRatiosBits),
    Long(&'a [Rational; MAX_BASE_UNITS]),
}


const _ASSERT_SIZE: () = {
    assert!(MAX_BASE_UNITS < 8);
};


impl SevenRatiosBits {
    pub fn nth_power(&self, n: u8) -> Rational {
        assert!((n as usize) < MAX_BASE_UNITS);
        let [signs, bytes@..] = self.0.to_le_bytes();
        Self::ratio_of_byte(bytes[n as usize], signs & (1 << (n + 1)) != 0)
    }

    pub fn unpack(self) -> [Rational; MAX_BASE_UNITS] {
        let [signs, bytes@..] = self.0.to_le_bytes();

        [
            Self::ratio_of_byte(bytes[0], signs & (1 << 1) != 0),
            Self::ratio_of_byte(bytes[1], signs & (1 << 2) != 0),
            Self::ratio_of_byte(bytes[2], signs & (1 << 3) != 0),
            Self::ratio_of_byte(bytes[3], signs & (1 << 4) != 0),
            Self::ratio_of_byte(bytes[4], signs & (1 << 5) != 0),
            Self::ratio_of_byte(bytes[5], signs & (1 << 6) != 0),
            Self::ratio_of_byte(bytes[6], signs & (1 << 7) != 0),
        ]
    }

    const fn ratio_of_byte(byte: u8, sign: bool) -> Rational {
        let num = (byte & 0x0F) as _;
        let den = ((byte & 0xF0) >> 4) as _;
        if sign {
            Rational::new_small(-num, den)
        } else {
            Rational::new_small(num, den)
        }
    }

    pub const fn pack(val: &[Rational; MAX_BASE_UNITS]) -> Option<Self> {
        const fn byte_of_ratio(r: &Rational, byte: &mut u8, sign_byte: &mut u8, bit: u8) -> Option<()> {
            let Some((n, d)): Option<(i16, u16)> = r.as_small() else {return None};
            let sign = n < 0;

            let n: u8 = if 0 <= n && n <= 15 {n as u8} else {return None};
            let d: u8 = if d <= 15 {d as u8} else {return None};

            *byte = d << 4 | n;
            *sign_byte |= if sign {1 << bit} else {0};
            Some(())
        }

        let mut le = [0u8; 8];

        let [f, s@..] = &mut le;

        let mut i = 0;
        while i < 8 {
            match byte_of_ratio(&val[i], &mut s[i], f, i as u8 + 1) {
                None => return None,
                Some(empty) => empty,
            }
            i += 1;
        }

        Some(Self(u64::from_le_bytes(le)))
    }
}


impl Dimensionality {
    /// Dimensionless unit
    pub const DIMENSIONLESS : Self = Self(0x1010101010101001);

    /// Dimensionality that is a base unit in power one
    pub fn unit(base_unit: u8) -> Dimensionality {
        let mut powers: [Rational; MAX_BASE_UNITS] = [Rational::ZERO; MAX_BASE_UNITS];
        powers[base_unit as usize] = Rational::ONE;
        Dimensionality::new(powers)
    }

    /// Cloning conversion into raw representation
    pub fn new(content: [Rational; MAX_BASE_UNITS]) -> Dimensionality {
        if let Some(short) = SevenRatiosBits::pack(&content) {
            Self(short.0 | 1)
        }
        else {
            Self(Box::into_raw(Box::new(content)) as usize as u64)
        }
    }

    /// Cloning conversion into raw representation
    pub fn into_numbers(self) -> [Rational; MAX_BASE_UNITS] {
        match self.unpack() {
            ExpandedDimensionality::Short(seven_ratios_bits) => seven_ratios_bits.unpack(),
            ExpandedDimensionality::Long(ns) => ns.clone(),
        }
    }

    /// Express
    pub fn nth_power(&self, n: u8) -> Rational {
        assert!(n as usize <= MAX_BASE_UNITS);

        match self.unpack() {
            ExpandedDimensionality::Short(seven_ratios_bits) => seven_ratios_bits.nth_power(n),
            ExpandedDimensionality::Long(ns) => ns[n as usize].clone(),
        }
    }

    /// Returns an iterator over UnitId and power expressed in base units
    pub fn iter_base_units<'a>(&'a self, include_zeros: bool) -> impl Iterator<Item=(UnitId, Rational)> + 'a {
        struct Gen<'a>{dim: &'a Dimensionality, curr_idx: u8, return_zeros: bool}

        impl Iterator for Gen<'_> {
            type Item = (UnitId, Rational);

            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    if self.curr_idx as usize >= MAX_BASE_UNITS {
                        break None;
                    }

                    let power = self.dim.nth_power(self.curr_idx);
                    if self.return_zeros || power != Rational::new_small(0, 1) {
                        let ret = UnitId::base(self.curr_idx);
                        self.curr_idx += 1;
                        break Some((ret, power));
                    }

                    self.curr_idx += 1;
                }
            }
        }

        Gen{dim: self, curr_idx: 0, return_zeros: include_zeros}
    }

    fn unpack(&self) -> ExpandedDimensionality<'_> {
        match self.0 & 1 {
            0 => ExpandedDimensionality::Long(unsafe{&*(self.0 as *const [Rational; MAX_BASE_UNITS])}),
            _ => ExpandedDimensionality::Short(SevenRatiosBits(self.0)),
        }
    }
}

impl Drop for Dimensionality {
    fn drop(&mut self) {
        if self.0 & 1 == 0 {
            core::mem::drop(unsafe{Box::from_raw(self.0 as *mut [Rational; MAX_BASE_UNITS])});
        }
    }
}


impl Clone for Dimensionality {
    fn clone(&self) -> Self {
        match self.unpack() {
            ExpandedDimensionality::Short(bits) => Self(bits.0 | 1),
            ExpandedDimensionality::Long(rational_array) => {
                let cloned_rational = rational_array.clone();
                Self::new(cloned_rational)
            }
        }
    }
}

impl PartialEq for Dimensionality {
    fn eq(&self, other: &Self) -> bool {
        self.clone().into_numbers() == other.clone().into_numbers()
    }
}



impl core::ops::Mul<Dimensionality> for Dimensionality {
    type Output = Dimensionality;
    fn mul(self, rhs: Dimensionality) -> Dimensionality {
        let [l0, l1, l2, l3, l4, l5, l6] = self.into_numbers();
        let [r0, r1, r2, r3, r4, r5, r6] = rhs.into_numbers();
        Self::new([l0+r0, l1+r1, l2+r2, l3+r3, l4+r4, l5+r5, l6+r6])
    }
}
impl core::ops::Div<Dimensionality> for Dimensionality {
    type Output = Dimensionality;
    fn div(self, rhs: Dimensionality) -> Dimensionality {
        let [l0, l1, l2, l3, l4, l5, l6] = self.into_numbers();
        let [r0, r1, r2, r3, r4, r5, r6] = rhs.into_numbers();
        Self::new([l0-r0, l1-r1, l2-r2, l3-r3, l4-r4, l5-r5, l6-r6])
    }
}

impl Dimensionality {
    /// Get a rational power of a dimensionality
    pub fn pow(&self, pwr: &Rational) -> Self {
        Dimensionality::new(self.clone().into_numbers().map(|a| a * pwr.clone()))
    }
}


#[test]
fn test_dimensionality() {
    const O: Rational = Rational::ZERO;
    const I: Rational = Rational::ONE;
    const TTH: Rational = Rational::new_small(2, 3);

    let dim = Dimensionality::DIMENSIONLESS;
    assert_eq!(dim.iter_base_units(false).next(), None);

    for i in 0..MAX_BASE_UNITS as u8 {
        let dim = Dimensionality::unit(i);
        assert_eq!(dim.iter_base_units(false).next().unwrap().0, UnitId::base(i));
    }

    let d0 = Dimensionality::unit(0);
    let d3 = Dimensionality::unit(3);

    assert_eq!(d0.clone() * d3.clone(), Dimensionality::new([I, O, O, I, O, O, O]));
    assert_eq!(d0.clone() / d3.clone(), Dimensionality::new([I, O, O, -I, O, O, O]));
    assert_eq!((d0.clone() / d3.clone()).pow(&TTH), Dimensionality::new([TTH, O, O, -TTH, O, O, O]));
}
