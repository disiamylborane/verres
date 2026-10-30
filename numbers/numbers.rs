//! A rational number and a real number used in calculations

use std::{borrow::Cow, cmp::Ordering, hint::unreachable_unchecked, str::FromStr};

use rug::{Rational as RugQ, ops::Pow, Complete};


/// Arbitrary precision rational number, 8 bytes, stack-allocated if small
#[repr(transparent)]
#[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
pub struct Rational{payload: u64}


/// Arbitrary precision real number, 8 bytes, stack-allocated if small
#[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
#[repr(transparent)]
pub struct Number{payload: u64}


const _: () = {
    assert!(size_of::<usize>() <= size_of::<Number>());
    assert!(align_of::<RugQ>() >= 4);
};


enum UnpackedRational<'a> {
    SmallQ{num: i64, den: i64},
    BigQ(&'a rug::Rational),
}


enum UnpackedNumber<'a> {
    Rational(&'a Rational),
    Real{val: f64},
}


enum UnpackedNumberVal {
    Rational(Rational),
    Real{val: f64},
}


impl Rational {
    const MAX_NUMER: i64 = (1 << 30) - 1;
    const MIN_NUMER: i64 = -(1 << 30);
    const MAX_DENOM: i64 = (1 << 31) - 1;

    /// Zero constant
    pub const ZERO: Self = Self::new_small(0, 1);

    /// One constant
    pub const ONE: Self = Self::new_small(1, 1);

    const fn unpack(&self) -> UnpackedRational<'_> {
        match self.payload % 4 {
            0b00 => {
                let rational_ptr = self.payload as *const RugQ;
                unsafe{UnpackedRational::BigQ(&*rational_ptr)}
            },
            0b01 => {
                let rational_bits = self.payload / 4;
                let denominator = rational_bits % (1 << 31);// 62-bit number
                let numerator_unsigned = rational_bits / (1 << 31);
                let numerator = ((numerator_unsigned as i64) << 33) >> 33;
                UnpackedRational::SmallQ { num: numerator, den: denominator as i64 }
            }
            _ => unsafe{unreachable_unchecked()}
        }
    }

    /// Create a stack-allocated Rational
    pub const fn new_small(numer: i16, denom: u16) -> Self {
        let payload = (((numer as i64 * (1 << 31)) + denom as i64) * 4 + 0b01) as u64;
        Self { payload }
    }

    /// Get as i16/u16 pair, if possible
    pub const fn as_small(&self) -> Option<(i16, u16)> {
        match self.unpack() {
            UnpackedRational::SmallQ { num, den } => {
                let num = if num >= i16::MIN as i64 && num <= i16::MAX as i64 {num as i16} else {return None};
                let den = if den >= u16::MIN as i64 && den <= u16::MAX as i64 {den as u16} else {return None};
                Some((num, den))
            }
            UnpackedRational::BigQ(_) => None,
        }
    }

    /// Indicates whether the number is not heap-allocated
    pub const fn is_small(&self) -> bool {
        (self.payload % 2) == 1
    }

    /// Get as i16/u16 pair, if possible
    pub const fn into_number(self) -> Number {
        let payload = self.payload;
        core::mem::forget(self);
        Number { payload }
    }

    /// Create Rational
    pub fn new(mut numer: i64, mut denom: i64) -> Self {
        assert!(denom != 0);

        if denom < 0 {
            numer = -numer;
            denom = -denom;
        }

        let gcd = gcd(numer.abs(), denom);
        if gcd > 1 {
            numer /= gcd;
            denom /= gcd;
        }

        if (Self::MIN_NUMER..=Self::MAX_NUMER).contains(&numer) && denom <= Self::MAX_DENOM {
            let payload = (((numer * (1 << 31)) + denom) * 4 + 0b01) as u64;
            Self { payload }
        } else {
            let r = RugQ::from((numer, denom));
            let payload = Box::into_raw(Box::new(r)) as usize as u64;
            Self { payload }
        }
    }

    pub fn new_i128(numer: i128, denom: u128) -> Self {
        assert!(denom != 0);

        let r = RugQ::from((numer, denom));
        let payload = Box::into_raw(Box::new(r)) as usize as u64;
        Self { payload }
    }

    /// Create from [`rug::Rational`]
    pub fn from_rational(q: RugQ) -> Self {
        let (n, d) = q.into_numer_denom();
        if let (Some(n), Some(d)) = (n.to_i64(), d.to_i64()) {
            Self::new(n, d)
        }
        else {
            let payload = Box::into_raw(Box::new(RugQ::from((n, d)))) as usize as u64;
            Self { payload }
        }
    }

    /// Convert into [`rug::Rational`]
    pub fn to_rational(&self) -> RugQ {
        match self.unpack() {
            UnpackedRational::SmallQ { num, den } => RugQ::from((num, den)),
            UnpackedRational::BigQ(rational) => rational.clone(),
        }
    }

    /// Create from [`rug::Rational`]
    fn to_rational_ref(&self) -> Cow<'_, RugQ> {
        match self.unpack() {
            UnpackedRational::SmallQ { num, den } => Cow::Owned(RugQ::from((num, den))),
            UnpackedRational::BigQ(rational) => Cow::Borrowed(rational),
        }
    }

    /// Convert into f64, truncating precision
    pub fn get_as_f64(&self) -> f64 {
        match self.unpack() {
            UnpackedRational::SmallQ { num, den } => num as f64 / den as f64,
            UnpackedRational::BigQ(rational) => rational.to_f64(),
        }
    }

    pub fn cmp0(&self) -> Ordering {
        match self.unpack() {
            UnpackedRational::SmallQ { num, den:_ } => num.cmp(&0),
            UnpackedRational::BigQ(rational) => rational.cmp0(),
        }
    }
}

impl Number {
    const fn payload_into_f64(p: u64) -> f64 {f64::from_bits(p / 4 * 4)}

    const fn unpack(&self) -> UnpackedNumber<'_> {
        match self.payload % 4 {
            0b00 | 0b01 => {
                unsafe{UnpackedNumber::Rational(&*(self as *const Number as *const Rational))}
            },
            _ => {
                UnpackedNumber::Real { val: Self::payload_into_f64(self.payload)  }
            }
        }
    }

    const fn unpack_val(self) -> UnpackedNumberVal {
        let ret = match self.payload % 4 {
            0b00 | 0b01 => {
                UnpackedNumberVal::Rational(Rational { payload: self.payload })
            },
            _ => {
                UnpackedNumberVal::Real { val: Self::payload_into_f64(self.payload)  }
            }
        };
        core::mem::forget(self);
        ret
    }

    /// Create a stack-allocated Number from f32
    pub const fn from_small_real(r: f32) -> Self {
        Self::shrink(r as f64)
    }

    /// Create a f62 number from f64 primitive
    pub const fn shrink(r: f64) -> Self {
        let r = r.to_bits();
        let bit = if r & 0b10 == 0b10 {1} else {0};
        let payload = ((r / 4 + bit) * 4) | 0b11;
        Self { payload }
    }

    /// Convert into f64, truncating precision
    pub fn get_as_f64(&self) -> f64 {
        match self.unpack() {
            UnpackedNumber::Rational(rational) => rational.get_as_f64(),
            UnpackedNumber::Real { val } => val,
        }
    }
}

impl From<Rational> for Number {
    fn from(r: Rational) -> Self {r.into_number()}
}

impl Clone for Rational {
    fn clone(&self) -> Self {
        match self.unpack() {
            UnpackedRational::SmallQ { num:_, den:_ } => Self { payload: self.payload },
            UnpackedRational::BigQ(rational) => {
                let cloned_rational = (*rational).clone();
                Self::from_rational(cloned_rational)
            }
        }
    }
}

impl Drop for Rational {
    fn drop(&mut self) {
        match self.unpack() {
            UnpackedRational::SmallQ { num:_, den:_ } => {}
            UnpackedRational::BigQ(rational) => {
                let _: &RugQ = rational;
                unsafe { let _boxed = Box::from_raw(rational as *const RugQ as *mut RugQ); }
            }
        }
    }
}

impl Clone for Number {
    fn clone(&self) -> Self {
        match self.unpack() {
            UnpackedNumber::Rational(r) => {r.clone().into_number()}
            UnpackedNumber::Real { val:_ } => Self { payload: self.payload },
        }
    }
}

impl Drop for Number {
    fn drop(&mut self) {
        match self.unpack() {
            UnpackedNumber::Rational(r) => match r.unpack() {
                UnpackedRational::SmallQ { num:_, den:_ } => {}
                UnpackedRational::BigQ(rational) => {
                    let _: &RugQ = rational;
                    unsafe { let _boxed = Box::from_raw(rational as *const RugQ as *mut RugQ); }
                }
            }
            UnpackedNumber::Real { val:_ } => {}
        }
    }
}


fn gcd(mut a: i64, mut b: i64) -> i64 {
    if a == 0 { return b; }
    if b == 0 { return a; }
    let shift = (a | b).trailing_zeros();
    a >>= a.trailing_zeros();
    while b != 0 {
        b >>= b.trailing_zeros();
        if a > b { std::mem::swap(&mut a, &mut b); }
        b -= a;
    }
    a << shift
}


impl core::fmt::Display for Rational {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.unpack() {
            UnpackedRational::SmallQ { num, den } =>
                if den == 1 {write!(f, "{}", num)}
                else {write!(f, "{}/{}", num, den)},
            UnpackedRational::BigQ(rational) => write!(f, "{}", rational),
        }
    }
}
impl core::fmt::Debug for Rational {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}


impl core::fmt::Display for Number {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.unpack() {
            UnpackedNumber::Rational(r) => r.fmt(f),
            UnpackedNumber::Real { val } => val.fmt(f),
        }
    }
}
impl core::fmt::Debug for Number {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}



impl PartialEq<Rational> for Rational {
    fn eq(&self, other: &Rational) -> bool {
        match (self.unpack(), other.unpack()) {
            (UnpackedRational::SmallQ { num:num1, den:den1 }, UnpackedRational::SmallQ { num:num2, den:den2 }) => {
                num1 == num2 && den1 == den2
            }
            (UnpackedRational::SmallQ { num:_, den:_ }, UnpackedRational::BigQ(_))
            | (UnpackedRational::BigQ(_), UnpackedRational::SmallQ { num:_, den:_ }) => false,
            (UnpackedRational::BigQ(rational1), UnpackedRational::BigQ(rational2)) => rational1 == rational2,
        }
    }
}


impl core::ops::Add for Rational {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        if let (UnpackedRational::SmallQ { num:n1, den:d1 }, UnpackedRational::SmallQ { num:n2, den:d2 }) = (self.unpack(), rhs.unpack()) {
            return Rational::new((n1 * d2) + (n2 * d1), d1 * d2);
        }
        let mut r1 = self.to_rational();
        let r2 = rhs.to_rational_ref();
        r1 += r2.as_ref();
        Self::from_rational(r1)
    }
}

impl core::ops::Sub for Rational {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        if let (UnpackedRational::SmallQ { num:n1, den:d1 }, UnpackedRational::SmallQ { num:n2, den:d2 }) = (self.unpack(), rhs.unpack()) {
            return Rational::new((n1 * d2) - (n2 * d1), d1 * d2);
        }
        let mut r1 = self.to_rational();
        let r2 = rhs.to_rational_ref();
        r1 -= r2.as_ref();
        Self::from_rational(r1)
    }
}

impl core::ops::Neg for Rational {
    type Output = Self;
    fn neg(self) -> Self::Output {
        if let UnpackedRational::SmallQ { num, den } = self.unpack() {
            return Rational::new(-num, den);
        }
        Self::from_rational(-self.to_rational())
    }
}

impl core::ops::Mul for Rational {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        if let (UnpackedRational::SmallQ { num:n1, den:d1 }, UnpackedRational::SmallQ { num:n2, den:d2 }) = (self.unpack(), rhs.unpack()) {
            return Rational::new(n1 * n2, d1 * d2);
        }
        let mut r1 = self.to_rational();
        let r2 = rhs.to_rational_ref();
        r1 *= r2.as_ref();
        Self::from_rational(r1)
    }
}

impl core::ops::Div for Rational {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        if let (UnpackedRational::SmallQ { num:n1, den:d1 }, UnpackedRational::SmallQ { num:n2, den:d2 }) = (self.unpack(), rhs.unpack()) {
            return Rational::new(n1 * d2, d1 * n2);
        }
        let mut r1 = self.to_rational();
        let r2 = rhs.to_rational_ref();
        r1 /= r2.as_ref();
        Self::from_rational(r1)
    }
}

impl core::ops::Add for Number {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match (self.unpack_val(), rhs.unpack_val()) {
            (UnpackedNumberVal::Rational(rational1), UnpackedNumberVal::Rational(rational2)) => {
                (rational1 + rational2).into_number()
            }
            (UnpackedNumberVal::Rational(rational), UnpackedNumberVal::Real { val }) => Self::shrink(rational.get_as_f64() + val),
            (UnpackedNumberVal::Real { val }, UnpackedNumberVal::Rational(rational)) => Self::shrink(val + rational.get_as_f64()),
            (UnpackedNumberVal::Real { val: val1 }, UnpackedNumberVal::Real { val: val2 }) => Self::shrink(val1 + val2),
        }
    }
}

impl core::ops::Sub for Number {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self.unpack_val(), rhs.unpack_val()) {
            (UnpackedNumberVal::Rational(rational1), UnpackedNumberVal::Rational(rational2)) => {
                (rational1 - rational2).into_number()
            }
            (UnpackedNumberVal::Rational(rational), UnpackedNumberVal::Real { val }) => Self::shrink(rational.get_as_f64() - val),
            (UnpackedNumberVal::Real { val }, UnpackedNumberVal::Rational(rational)) => Self::shrink(val - rational.get_as_f64()),
            (UnpackedNumberVal::Real { val: val1 }, UnpackedNumberVal::Real { val: val2 }) => Self::shrink(val1 - val2),
        }
    }
}

impl core::ops::Mul for Number {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self.unpack_val(), rhs.unpack_val()) {
            (UnpackedNumberVal::Rational(rational1), UnpackedNumberVal::Rational(rational2)) => {
                (rational1 * rational2).into_number()
            }
            (UnpackedNumberVal::Rational(rational), UnpackedNumberVal::Real { val }) => Self::shrink(rational.get_as_f64() * val),
            (UnpackedNumberVal::Real { val }, UnpackedNumberVal::Rational(rational)) => Self::shrink(val * rational.get_as_f64()),
            (UnpackedNumberVal::Real { val: val1 }, UnpackedNumberVal::Real { val: val2 }) => Self::shrink(val1 * val2),
        }
    }
}

impl core::ops::Div for Number {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        match (self.unpack_val(), rhs.unpack_val()) {
            (UnpackedNumberVal::Rational(rational1), UnpackedNumberVal::Rational(rational2)) => {
                (rational1 / rational2).into_number()
            }
            (UnpackedNumberVal::Rational(rational), UnpackedNumberVal::Real { val }) => Self::shrink(rational.get_as_f64() / val),
            (UnpackedNumberVal::Real { val }, UnpackedNumberVal::Rational(rational)) => Self::shrink(val / rational.get_as_f64()),
            (UnpackedNumberVal::Real { val: val1 }, UnpackedNumberVal::Real { val: val2 }) => Self::shrink(val1 / val2),
        }
    }
}

impl core::ops::Neg for Number {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self.unpack_val() {
            UnpackedNumberVal::Rational(r) => {
                (-r).into_number()
            }
            UnpackedNumberVal::Real { val } => {
                Self::shrink(-val)
            }
        }
    }
}


impl Rational {
    /// Take a rational power
    pub fn pow(self, exponent: Rational) -> Number {
        let exp_rug = exponent.to_rational();
        if exp_rug.is_zero() {
            return Rational::ONE.into_number();
        }

        let base_rug = self.to_rational();
        let exp_num = exp_rug.numer();
        let exp_denom = exp_rug.denom();

        let p = match exp_num.to_i32() {
            Some(val) => val,
            None => return Number::shrink(base_rug.to_f64().powf(exp_rug.to_f64())),
        };

        let d = match exp_denom.to_u32() {
            Some(val) => val,
            None => return Number::shrink(base_rug.to_f64().powf(exp_rug.to_f64())),
        };

        let intermediate: RugQ = base_rug.clone().pow(&p);

        let num_intel = intermediate.numer().clone();
        let denom_intel = intermediate.denom().clone();

        let (num_root, num_rem) = num_intel.root_rem(rug::Integer::new(), d);
        let (denom_root, denom_rem) = denom_intel.root_rem(rug::Integer::new(), d);

        if num_rem.is_zero() && denom_rem.is_zero() {
            let exact_rug = rug::Rational::from((num_root, denom_root));
            if let (Some(n), Some(d)) = (exact_rug.numer().to_i64(), exact_rug.denom().to_i64()) {
                Rational::new(n, d).into_number()
            } else {
                Rational::from_rational(exact_rug).into_number()
            }
        } else {
            let base_f = base_rug.to_f64();
            let exp_f = exp_rug.to_f64();
            Number::shrink(base_f.powf(exp_f))
        }
    }
}
impl Number {
    /// Take a rational power
    pub fn powr(self, r: Rational) -> Number {
        match self.unpack_val() {
            UnpackedNumberVal::Rational(rat) => rat.pow(r),
            UnpackedNumberVal::Real { val } => Number::shrink(val.powf(r.get_as_f64())),
        }
    }

    /// Take an arbitrary power
    pub fn pown(self, n: Number) -> Number {
        match n.unpack_val() {
            UnpackedNumberVal::Rational(rational) => self.powr(rational),
            UnpackedNumberVal::Real { val:exp } => match self.unpack_val() {
                UnpackedNumberVal::Rational(rat) => Number::shrink(rat.get_as_f64().powf(exp)),
                UnpackedNumberVal::Real { val:base } => Number::shrink(base.powf(exp)),
            }
        }
    }
}

#[derive(Debug)]
pub struct NumberParseError;

impl core::fmt::Display for NumberParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid number")
    }
}

impl FromStr for Number {
    type Err = NumberParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_suffix("f") {
            Ok(Number::shrink(s.parse().map_err(|_| NumberParseError)?))
        } else {
            Ok(Rational::from_str(s)?.into_number())
        }
    }
}

impl FromStr for Rational {
    type Err = NumberParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("/") {
            return Ok(Self::from_rational(RugQ::from_str(s).map_err(|_|NumberParseError)?));
        }

        use rug::ops::Pow;
        use rug::{Integer, Rational};
        let clean_s = s.replace('_', "").to_lowercase();

        let (mantissa_str, exp_str) = match clean_s.split_once('e') {
            Some((m, e)) => (m, Some(e)),
            None => (clean_s.as_str(), None),
        };

        let mut rational = if let Some((init, frac)) = mantissa_str.split_once('.') {
            let combined_str = format!("{}{}", init, frac);
            let numerator = Integer::parse(&combined_str).map_err(|_| NumberParseError)?.complete();
            let denominator = Integer::from(10).pow(frac.len() as u32);
            Rational::from((numerator, denominator))
        } else {
            let numer = Integer::parse(mantissa_str).map_err(|_| NumberParseError)?.complete();
            Rational::from(numer)
        };

        if let Some(exp_s) = exp_str {
            let exp_clean = exp_s.strip_prefix('+').unwrap_or(exp_s);
            let exp_val = exp_clean.parse::<i32>().map_err(|_| NumberParseError)?;

            let base = Integer::from(10).pow(exp_val.unsigned_abs());
            let multiplier = Rational::from(base);

            if exp_val >= 0 {
                rational *= multiplier;
            } else {
                rational /= multiplier;
            }
        }

        Ok(Self::from_rational(rational))
    }
}


#[test]
fn test_rationals() {
    assert_eq!(std::mem::size_of::<Rational>(), 8);

    const CONST_A: Rational = Rational::new_small(5, 7);
    const CONST_B: Rational = Rational::new_small(-1, 3);

    assert!(CONST_A.is_small());
    assert!(CONST_B.is_small());
    assert_eq!(*CONST_A.to_rational().numer(), 5);
    assert_eq!(*CONST_A.to_rational().denom(), 7);
    assert_eq!(*CONST_B.to_rational().numer(), -1);
    assert_eq!(*CONST_B.to_rational().denom(), 3);

    let reduce_it = Rational::new(3_000_000, 9_000_000);
    assert_eq!(format!("{}", reduce_it), "1/3");
    assert!(reduce_it.is_small());

    let r1 = Rational::new(1, 4);
    let r2 = Rational::new(2, 3);

    let res_add = r1.clone() + r2.clone();
    assert!(res_add.is_small());
    assert_eq!(res_add, Rational::new_small(11, 12));

    let res_sub = r1.clone() - r2.clone();
    assert!(res_sub.is_small());
    assert_eq!(res_sub, Rational::new_small(-5, 12));

    let res_mul = r1.clone() * r2.clone();
    assert!(res_mul.is_small());
    assert_eq!(res_mul, Rational::new_small(1, 6));

    let res_div = r1.clone() / r2.clone();
    assert!(res_div.is_small());
    assert_eq!(res_div, Rational::new_small(3, 8));


    let big_stack1 = Rational::new(400_000_000, 3);
    let big_stack2 = Rational::new(400_000_000, 7);

    assert!(big_stack1.is_small());
    assert!(big_stack2.is_small());

    let heap_result = big_stack1 + big_stack2;
    assert!(!heap_result.is_small());
    assert_eq!(heap_result, Rational::new(4_000_000_000, 21));

    let neg1 = Rational::new(-15, 4);
    let neg2 = Rational::new(5, -2);
    assert_eq!(*neg1.to_rational().numer(), -15);
    assert_eq!(*neg1.to_rational().denom(), 4);
    assert_eq!(*neg2.to_rational().numer(), -5);
    assert_eq!(*neg2.to_rational().denom(), 2);

    let res_neg_mul = neg1 * neg2;
    assert_eq!(format!("{}", res_neg_mul), "75/8");
    assert!(res_neg_mul.is_small());

    let options_small = [
        Rational::new(0, 1),
        Rational::new(0, Rational::MAX_DENOM),
        Rational::new(0, Rational::MAX_DENOM+1),

        Rational::new(1, 1),
        Rational::new(1, Rational::MAX_DENOM),
        Rational::new(Rational::MAX_NUMER, 1),
        Rational::new(Rational::MAX_NUMER, Rational::MAX_DENOM),
        Rational::new(Rational::MIN_NUMER, 1),
        Rational::new(Rational::MIN_NUMER, Rational::MAX_DENOM),

        Rational::new(Rational::MAX_NUMER+1, Rational::MAX_DENOM+1), // == 1/2
    ];

    let options_big = [
        Rational::new(1, Rational::MAX_DENOM+1),
        Rational::new(Rational::MAX_NUMER+1, 1),
        Rational::new(Rational::MAX_NUMER+1, Rational::MAX_DENOM),
        Rational::new(Rational::MIN_NUMER-1, 1),
        Rational::new(Rational::MIN_NUMER-2, Rational::MAX_DENOM),
        Rational::new(Rational::MIN_NUMER-1, Rational::MAX_DENOM+1),
        Rational::new(Rational::MIN_NUMER-1, Rational::MAX_DENOM),
    ];

    let options_all = options_small.iter().cloned().chain(options_big.iter().cloned()).collect::<Vec<_>>();

    for opt1 in &options_small {
        assert!(opt1.is_small());
    }
    for opt1 in &options_big {
        assert!(!opt1.is_small());
    }

    for opt1 in &options_all {
        for opt2 in &options_all {
            let _ = opt1.clone() + opt2.clone();
            let _ = opt1.clone() - opt2.clone();
            let _ = opt1.clone() * opt2.clone();
            if opt2.clone() != Rational::new(0, 1) {
                let _ = opt1.clone() / opt2.clone();
            }
            else {
                assert_eq!(*opt2, Rational::new_small(0, 1));
            }
        }
    }
}


#[test]
fn test_numbers() {
    assert_eq!(Rational::new(3, 5).to_string(), "3/5");
    assert_eq!(Rational::new(3, -5).to_string(), "-3/5");
    assert_eq!(Rational::new(3, 5).into_number().to_string(), "3/5");
    assert_eq!(Rational::new(3, -5).into_number().to_string(), "-3/5");

    assert_eq!((Rational::new(3, 5).into_number() + Rational::new(1, 5).into_number()).to_string(), "4/5");
    assert_eq!((Rational::new(3, 5).into_number() + Rational::new(2, 5).into_number()).to_string(), "1");

    assert_eq!((Rational::new(3, 5).into_number() - Rational::new(1, 5).into_number()).to_string(), "2/5");
    assert_eq!((Rational::new(3, 5).into_number() - Rational::new(2, 5).into_number()).to_string(), "1/5");
    assert_eq!((Rational::new(3, 5).into_number() - Rational::new(8, 5).into_number()).to_string(), "-1");

    assert_eq!((Rational::new(2, 10).into_number() + Number::shrink(0.3)).to_string(), "0.5");
    assert_eq!((Number::shrink(0.3) + Rational::new(2, 10).into_number()).to_string(), "0.5");
    assert_eq!((Rational::new(2, 10).into_number() + Rational::new(2, 10).into_number()).to_string(), "2/5");
}
