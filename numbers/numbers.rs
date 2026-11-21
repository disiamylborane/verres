//! [Stub] A stub to represent numbers used in scientific calculations, contains errors

#[cfg(not(target_family = "wasm"))]
#[path = "details_dashu.rs"]
mod details;

#[cfg(target_family = "wasm")]
#[path = "details_primitives.rs"]
mod details;

use core::hint::unreachable_unchecked;

use details::{BigQ, BigF};

const _: () = {
    assert!(size_of::<usize>() <= size_of::<u64>());
    assert!(align_of::<BigQ>() >= 8);
    assert!(align_of::<BigF>() >= 8);
};

#[repr(transparent)]
pub struct Rational {payload: u64}

#[repr(transparent)]
pub struct Number {payload: u64}

// Small float representation:
// [sign][9-bit exponent][mantissa][0][n-bit 1][1]
// Small rational representation:
// [i31][u31][1][0]
// Big rational representation:
// [pointer-i61][0][0][0]
// Big float representation:
// [pointer-i61][1][0][0]

const _CHECK_SMALL_REPR: () = {
    assert!(Rational::NUM_SZ + Rational::DEN_SZ + Rational::SMALL_RATIONAL_SHIFT == 64);
};

const SMALL_RATIONAL_TAG: u64 = 0b10;
const BIG_FLOAT_TAG: u64 = 0b100;
const BIG_RATIONAL_TAG: u64 = 0b000;
const TAG_MASK: u64 = 0b111;

impl Rational {
    pub const ZERO : Self = Self::new_small(0, 1);
    pub const ONE : Self = Self::new_small(1, 1);

    const SMALL_RATIONAL_SHIFT: u8 = SMALL_RATIONAL_TAG.trailing_zeros() as u8 + 1;
    const NUM_SZ: u8 = 31;
    const DEN_SZ: u8 = 31;
    const NUMERATOR_SHIFT: u8 = Self::SMALL_RATIONAL_SHIFT;
    const DENOMINATOR_SHIFT: u8 = Self::NUMERATOR_SHIFT + Self::NUM_SZ;

    const unsafe fn pack_inline(numer: i32, denom: u32) -> Self { // in fact i31, u31
        let payload = (((numer as u32 & ((1<<Self::NUM_SZ)-1)) as u64) << Self::NUMERATOR_SHIFT) | ((denom as u64) << Self::DENOMINATOR_SHIFT) | SMALL_RATIONAL_TAG;
        Self { payload }
    }

    unsafe fn pack_boxed(r: Box<BigQ>) -> Self {
        let payload = Box::into_raw(r) as usize as u64 | BIG_RATIONAL_TAG;
        Self { payload }
    }

    unsafe fn from_reduced(numer: i64, denom: u64) -> Self {
        if ((-1i64 << (Self::NUM_SZ-1))..=((1i64 << (Self::NUM_SZ-1)) - 1)).contains(&numer) && denom < (1 << Self::DEN_SZ) {
            unsafe {Self::pack_inline(numer as i32, denom as u32)}
        } else {
            unsafe {Self::pack_boxed(Box::new(BigQ::from((numer, denom))))}
        }
    }

    fn from_big(r: BigQ) -> Self {
        if let Some((numer, denom)) = r.to_parts() {
            unsafe{Self::from_reduced(numer, denom)}
        }
        else {
            unsafe{Self::pack_boxed(Box::new(r))}
        }
    }

    const fn unpack(&self) -> UnpackedRational<'_> {
        match self.payload & 0b10 {
            BIG_RATIONAL_TAG => {
                let rational_ptr = (self.payload & !TAG_MASK) as *const BigQ;
                unsafe{UnpackedRational::BigQ(&*rational_ptr)}
            },
            SMALL_RATIONAL_TAG => {
                let rational_bits = self.payload >> Self::SMALL_RATIONAL_SHIFT;
                let shift = 32 - Self::NUM_SZ;
                let numerator = ((rational_bits & ((1 << Self::NUM_SZ)-1))  as u32 as i32) << shift >> shift;
                let denominator = rational_bits >> Self::NUM_SZ;
                UnpackedRational::SmallQ { num: numerator, den: denominator as u32}
            }
            _ => unsafe{unreachable_unchecked()}
        }
    }

    fn unpack_val(self) -> UnpackedRationalVal {
        let ret = match self.unpack() {
            UnpackedRational::SmallQ { num, den } => UnpackedRationalVal::SmallQ { num, den },
            UnpackedRational::BigQ(big_q) => {
                let rational_ptr = big_q as *const BigQ as *mut BigQ;
                unsafe{UnpackedRationalVal::BigQ(*Box::from_raw(rational_ptr))}
            }
        };
        core::mem::forget(self);
        ret
    }

    pub const fn new_small(numer: i16, denom: u16) -> Self {
        let gcd = gcd(numer.unsigned_abs() as u64, denom as u64);
        unsafe{Self::pack_inline(numer as i32 / gcd as i32, denom as u32 / gcd as u32)}
    }

    pub const fn as_small(&self) -> Option<(i16, u16)> {
        match self.unpack() {
            UnpackedRational::SmallQ { num, den } => {
                let num = if num >= i16::MIN as i32 && num <= i16::MAX as i32 {num as i16} else {return None};
                let den = if den >= u16::MIN as u32 && den <= u16::MAX as u32 {den as u16} else {return None};
                Some((num, den))
            }
            UnpackedRational::BigQ(_) => None,
        }
    }

    pub fn new_med(mut numer: i64, mut denom: u64) -> Self {
        assert!(denom != 0);

        let gcd = gcd(numer.unsigned_abs(), denom);
        numer /= gcd as i64;  // May be converted, gcd is less than numer.abs()
        denom /= gcd;

        unsafe{Self::from_reduced(numer, denom)}
    }

    pub fn new_big(numer: i128, denom: u128) -> Self {
        assert!(denom != 0);
        Self::from_big(BigQ::from((numer, denom)))
    }

    fn to_big(&self) -> std::borrow::Cow<'_, BigQ> {
        match self.unpack() {
            UnpackedRational::SmallQ { num, den } => std::borrow::Cow::Owned(BigQ::from((num as i64, den as u64))),
            UnpackedRational::BigQ(big_q) => std::borrow::Cow::Borrowed(big_q),
        }
    }

    pub const fn into_number(self) -> Number {
        let payload = self.payload;
        core::mem::forget(self);
        Number { payload }
    }

    pub fn cmp0(&self) -> std::cmp::Ordering {
        match self.unpack() {
            UnpackedRational::SmallQ { num, den:_ } => num.cmp(&0),
            UnpackedRational::BigQ(rational) => rational.cmp0(),
        }
    }
}


impl Clone for Rational {
    fn clone(&self) -> Self {
        unsafe{
            match self.unpack() {
                UnpackedRational::SmallQ { num, den } => Self::pack_inline(num, den),
                UnpackedRational::BigQ(big_q) => Self::pack_boxed(Box::new(big_q.clone())),
            }
        }
    }
}


impl Clone for Number {
    fn clone(&self) -> Self {
        unsafe{
            match self.unpack() {
                UnpackedNumber::Rational(rational) => Self::new_rational(rational.clone()),
                UnpackedNumber::SmallF { val, precision } => Self::pack_f64(val, precision),
                UnpackedNumber::BigF(f) => Self::pack_boxed_float(Box::new(f.clone())),
            }
        }
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
            (UnpackedRational::BigQ(rational1), UnpackedRational::BigQ(rational2)) => rational1.eq(rational2),
        }
    }
}

impl Drop for Rational {
    fn drop(&mut self) {
        match self.unpack() {
            UnpackedRational::SmallQ { num:_, den:_ } => (),
            UnpackedRational::BigQ(big_q) => {
                let rational_ptr = big_q as *const BigQ as *mut BigQ;
                let _box_to_drop = unsafe{Box::from_raw(rational_ptr)};
            }
        };
    }
}

impl Drop for Number {
    fn drop(&mut self) {
        match self.unpack() {
            UnpackedNumber::Rational(rational) => match rational.unpack() {
                UnpackedRational::SmallQ { num:_, den:_ } => (),
                UnpackedRational::BigQ(big_q) => {
                    let rational_ptr = big_q as *const BigQ as *mut BigQ;
                    let _box_to_drop = unsafe{Box::from_raw(rational_ptr)};
                }
            },
            UnpackedNumber::SmallF { val:_, precision:_ } => (),
            UnpackedNumber::BigF(big_f) => {
                    let fptr = big_f as *const BigF as *mut BigF;
                    let _box_to_drop = unsafe{Box::from_raw(fptr)};
                }
        };
    }
}

const fn gcd(mut a: u64, mut b: u64) -> u64 {
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


enum UnpackedRational<'a> {
    SmallQ{num: i32, den: u32},
    BigQ(&'a BigQ),
}


enum UnpackedRationalVal {
    SmallQ{num: i32, den: u32},
    BigQ(BigQ),
}


enum UnpackedNumberVal {
    Rational(Rational),
    SmallF{val: f64, precision: u8},
    BigF(BigF),
}


enum UnpackedNumber<'a> {
    Rational(&'a Rational),
    SmallF{val: f64, precision: u8},
    BigF(&'a BigF),
}


impl core::ops::Add for Rational {
    type Output = Self;
    fn add(self, rhs: Rational) -> Self {
        let (mut b1, b2) = match (self.unpack_val(), rhs.unpack_val()) {
            (UnpackedRationalVal::SmallQ { num:n1, den:d1 }, UnpackedRationalVal::SmallQ { num:n2, den:d2 }) =>
                return Rational::new_med((n1 as i64 * d2 as i64) + (n2 as i64 * d1 as i64), d1 as u64 * d2 as u64),
            (UnpackedRationalVal::SmallQ { num, den }, UnpackedRationalVal::BigQ(big_q))
            | (UnpackedRationalVal::BigQ(big_q), UnpackedRationalVal::SmallQ { num, den }) => {
                (big_q, BigQ::from((num as i64, den as u64)))
            }
            (UnpackedRationalVal::BigQ(b1), UnpackedRationalVal::BigQ(b2)) => (b1, b2),
        };
        b1.add(&b2);
        Self::from_big(b1)
    }
}


impl core::ops::Sub for Rational {
    type Output = Self;
    fn sub(self, rhs: Rational) -> Self {
        let (mut b1, b2) = match (self.unpack_val(), rhs.unpack_val()) {
            (UnpackedRationalVal::SmallQ { num:n1, den:d1 }, UnpackedRationalVal::SmallQ { num:n2, den:d2 }) =>
                return Rational::new_med((n1 as i64 * d2 as i64) - (n2 as i64 * d1 as i64), d1 as u64 * d2 as u64),
            (UnpackedRationalVal::SmallQ { num, den }, UnpackedRationalVal::BigQ(big_q)) => {
                (BigQ::from(((num as i64), den as u64)), big_q)
            }
            (UnpackedRationalVal::BigQ(big_q), UnpackedRationalVal::SmallQ { num, den }) => {
                (big_q, BigQ::from((num as i64, den as u64)))
            }
            (UnpackedRationalVal::BigQ(b1), UnpackedRationalVal::BigQ(b2)) => (b1, b2),
        };
        b1.sub(&b2);
        Self::from_big(b1)
    }
}


impl core::ops::Mul for Rational {
    type Output = Self;
    fn mul(self, rhs: Rational) -> Self {
        let (mut b1, b2) = match (self.unpack_val(), rhs.unpack_val()) {
            (UnpackedRationalVal::SmallQ { num:n1, den:d1 }, UnpackedRationalVal::SmallQ { num:n2, den:d2 }) =>
                return Rational::new_med(n1 as i64 * n2 as i64, d1 as u64 * d2 as u64),
            (UnpackedRationalVal::SmallQ { num, den }, UnpackedRationalVal::BigQ(big_q))
            | (UnpackedRationalVal::BigQ(big_q), UnpackedRationalVal::SmallQ { num, den }) => {
                (big_q, BigQ::from((num as i64, den as u64)))
            }
            (UnpackedRationalVal::BigQ(b1), UnpackedRationalVal::BigQ(b2)) => (b1, b2),
        };
        b1.mul(&b2);
        Self::from_big(b1)
    }
}


impl core::ops::Div for Rational {
    type Output = Self;
    fn div(self, rhs: Rational) -> Self {
        let (mut b1, b2) = match (self.unpack_val(), rhs.unpack_val()) {
            (UnpackedRationalVal::SmallQ { num:n1, den:d1 }, UnpackedRationalVal::SmallQ { num:n2, den:d2 }) =>
                return Rational::new_med(n1 as i64 * d2 as i64, d1 as u64 * n2 as u64),
            (UnpackedRationalVal::SmallQ { num, den }, UnpackedRationalVal::BigQ(big_q)) => {
                (BigQ::from(((num as i64), den as u64)), big_q)
            }
            (UnpackedRationalVal::BigQ(big_q), UnpackedRationalVal::SmallQ { num, den }) => {
                (big_q, BigQ::from((num as i64, den as u64)))
            }
            (UnpackedRationalVal::BigQ(b1), UnpackedRationalVal::BigQ(b2)) => (b1, b2),
        };
        b1.div(&b2);
        Self::from_big(b1)
    }
}


impl core::ops::Neg for Rational {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self.unpack_val() {
            UnpackedRationalVal::SmallQ { num, den } => Self::new_med(-(num as i64), den as u64),
            UnpackedRationalVal::BigQ(big_q) => Self::from_big(big_q.neg()),
        }
    }
}


impl Number {
    pub fn binop_ctx<Ctx>(self, rhs: Number, r: &dyn Fn(Rational, Rational, Ctx) -> Number, f: fn(&mut BigF, BigF), c: Ctx) -> Self {
        let (mut f1, f2) = match (self.unpack_val(), rhs.unpack_val()) {
            (UnpackedNumberVal::Rational(r1), UnpackedNumberVal::Rational(r2)) =>
                return r(r1, r2, c),

            (UnpackedNumberVal::Rational(r), UnpackedNumberVal::SmallF { val, precision }) =>
                (BigF::from_rational_precision(r.to_big().into_owned(), precision as u16), BigF::from_float_precision(val, precision)),
            (UnpackedNumberVal::Rational(r), UnpackedNumberVal::BigF(f)) =>
                (BigF::from_rational_precision(r.to_big().into_owned(), f.precision()), f),
            (UnpackedNumberVal::SmallF { val, precision }, UnpackedNumberVal::Rational(r)) =>
                (BigF::from_float_precision(val, precision), BigF::from_rational_precision(r.to_big().into_owned(), precision as u16)),
            (UnpackedNumberVal::SmallF { val: v1, precision: p1 }, UnpackedNumberVal::SmallF { val: v2, precision: p2 }) =>
                (BigF::from_float_precision(v1, p1), BigF::from_float_precision(v2, p2)),
            (UnpackedNumberVal::SmallF { val, precision }, UnpackedNumberVal::BigF(big_f)) =>
                (BigF::from_float_precision(val, precision), big_f),
            (UnpackedNumberVal::BigF(f), UnpackedNumberVal::Rational(r)) => {
                let prec = f.precision();
                (f, BigF::from_rational_precision(r.to_big().into_owned(), prec))
            }
            (UnpackedNumberVal::BigF(f), UnpackedNumberVal::SmallF { val, precision }) =>
                (f, BigF::from_float_precision(val, precision)),
            (UnpackedNumberVal::BigF(f), UnpackedNumberVal::BigF(f2)) =>
                (f, f2),
        };
        f(&mut f1, f2);
        Self::from_big(f1)
    }

    pub fn binop(self, rhs: Number, r: fn(Rational, Rational) -> Number, f: fn(&mut BigF, BigF)) -> Self {
        self.binop_ctx(rhs, &|r1,r2,_ctx| r(r1, r2), f, ())
    }
}


impl core::ops::Add for Number {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.binop(rhs, |r1, r2| Self::new_rational(r1+r2), |f1, f2| f1.add(f2))
    }
}


impl core::ops::Sub for Number {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.binop(rhs, |r1, r2| Self::new_rational(r1-r2), |f1, f2| f1.sub(f2))
    }
}


impl core::ops::Mul for Number {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.binop(rhs, |r1, r2| Self::new_rational(r1*r2), |f1, f2| f1.mul(f2))
    }
}


impl core::ops::Div for Number {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        self.binop(rhs, |r1, r2| Self::new_rational(r1/r2), |f1, f2| f1.div(f2))
    }
}


impl Number {
    pub fn pow(self, rhs: Self, new_precision: u16) -> Self {
        self.binop_ctx(rhs,
            &|r1, r2, nprec| {
                let r1 = r1.to_big().into_owned();
                let r2 = r2.to_big();

                match r1.rational_pow(r2.as_ref(), nprec) {
                    details::ExactOrReal::Exact(r) => Rational::from_big(r).into_number(),
                    details::ExactOrReal::Approx(f) => Number::from_big(f),
                }
            },
            |f1, f2| f1.powf(&f2),
            new_precision
        )
    }
    pub fn pown(self, rhs: Self) -> Self {
        self.pow(rhs, 53)
    }
    pub fn powr(self, rhs: Rational) -> Self {
        self.pow(rhs.into_number(), 53)
    }
    pub fn log(self, rhs: Self, new_precision: u16) -> Self {
        self.binop_ctx(rhs,
            &|r1, r2, nprec| {
                let r1 = r1.to_big().into_owned();
                let r2 = r2.to_big();

                match r1.rational_log(r2.as_ref(), nprec) {
                    details::ExactOrReal::Exact(r) => Rational::from_big(r).into_number(),
                    details::ExactOrReal::Approx(f) => Number::from_big(f),
                }
            },
            |f1, f2| f1.log(&f2),
            new_precision
        )
    }
    pub fn logn(self, rhs: Self) -> Self {
        self.log(rhs, 53)
    }
}


impl core::str::FromStr for Rational {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(Rational::from_big(BigQ::from_str(s)?))
    }
}


impl core::str::FromStr for Number {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(if s.contains('~') {
            Self::from_big(BigF::from_str(s)?)
        } else {
            Rational::from_big(BigQ::from_str(s)?).into_number()
        })
    }
}



const fn decode_f64(val: f64) -> (i8, i16, u64) {
    let bits = val.to_bits();
    let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
    let biased_exp = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = bits & 0x000f_ffff_ffff_ffff;

    (sign, biased_exp - 1024, mantissa)
}

const fn encode_f64(sign: i8, exponent: i16, mantissa: u64) -> f64 {
    let sign_bit: u64 = if sign < 0 { 1 << 63 } else { 0 };
    let biased_exp = ((exponent + 1024) as u64) & 0x7ff;
    let raw_mant = mantissa & 0x000f_ffff_ffff_ffff;

    f64::from_bits(sign_bit | (biased_exp << 52) | raw_mant)
}
impl Number {
    pub const EXPONENT_BITS: u8 = 9;
    pub const MANTISSA_BITS: u8 = 52;

    pub const F64_EXPONENT_BITS: u8 = 11;

    pub fn new_rational(r: Rational) -> Self {
        let ret = Self{payload: r.payload};
        core::mem::forget(r);
        ret
    }

    unsafe fn pack_f64(val: f64, precision: u8) -> Self { // exponent fits, prec in range 0..=MANTISSA_BITS
        let mantissa_bits = precision - 1;

        let (sign, exponent, mantissa) = decode_f64(val);

        let sign = if sign < 0 {1 << 63} else {0};
        let exponent = ((exponent + 256) as u16 as u64) << (64 - 1 - Self::EXPONENT_BITS);

        // FIXME: halfeven round
        let mantissa = mantissa << (Self::F64_EXPONENT_BITS - Self::EXPONENT_BITS);
        let indicator_ones = (1 << (52 + 1 - mantissa_bits)) - 1;
        let indicator_zero = !(1 << (52 + 1 - mantissa_bits));

        let payload = (sign | exponent | mantissa | indicator_ones) & indicator_zero;

        Self {payload}
    }

    unsafe fn pack_boxed_float(f: Box<BigF>) -> Self {
        let payload = Box::into_raw(f) as usize as u64 | BIG_FLOAT_TAG;
        Self { payload }
    }


    const fn unpack(&self) -> UnpackedNumber<'_> {
        match self.payload.trailing_zeros() {
            0 => {
                let p = self.payload;
                let precision = Self::MANTISSA_BITS + 2 - p.trailing_ones() as u8;
                let sign = if self.payload & (1<<63) != 0 {-1} else {1};
                let exponent = (self.payload >> (64 - 1 - Self::EXPONENT_BITS)) as u16 as i16 - 256;
                let mantissa = self.payload >> (Self::F64_EXPONENT_BITS - Self::EXPONENT_BITS);

                let indicator_ones = ((1 << (52 + 1 - (precision - 1))) - 1) >> (Self::F64_EXPONENT_BITS - Self::EXPONENT_BITS);

                UnpackedNumber::SmallF { val: encode_f64(sign, exponent, mantissa & !indicator_ones), precision }
            }
            1 => UnpackedNumber::Rational(unsafe{core::mem::transmute::<&Number, &Rational>(self)}),
            2 => { // big float
                let fptr = (self.payload & !TAG_MASK) as *const BigF;
                unsafe{UnpackedNumber::BigF(&*fptr)}
            }
            _ => UnpackedNumber::Rational(unsafe{core::mem::transmute::<&Number, &Rational>(self)}),
        }
    }

    fn unpack_val(self) -> UnpackedNumberVal {
        let ret = match self.unpack() {
            UnpackedNumber::Rational(_r) => UnpackedNumberVal::Rational(unsafe{core::mem::transmute::<u64, Rational>(self.payload)}),
            UnpackedNumber::SmallF { val, precision } => UnpackedNumberVal::SmallF { val, precision },
            UnpackedNumber::BigF(big_f) => {
                let f_ptr = big_f as *const BigF as *mut BigF;
                unsafe{UnpackedNumberVal::BigF(*Box::from_raw(f_ptr))}
            }
        };
        core::mem::forget(self);
        ret
    }

    fn to_big(&self) -> BigNumber {
        match self.unpack() {
            UnpackedNumber::Rational(rational) => BigNumber::Q(rational.to_big().into_owned()),
            UnpackedNumber::SmallF { val, precision } => BigNumber::F(BigF::from_float_precision(val, precision)),
            UnpackedNumber::BigF(big_f) => BigNumber::F(big_f.clone()),
        }
    }

    pub fn from_big(f: BigF) -> Self {
        unsafe{Self::pack_boxed_float(Box::new(f))}
    }

    pub fn shrink(f: f64) -> Self {
        if f.is_normal() {
            // FIXME: this should be calculated from constants, not hardcoded here
            let exponent = decode_f64(f).1;
            if (-256..=255).contains(&exponent) {
                return unsafe{Self::pack_f64(f, 53)};
            }
        }
        unsafe{Self::pack_boxed_float(Box::new(BigF::from_float_precision(f, 53)))}
    }


    pub fn get_as_f64(&self) -> f64 {
        match self.unpack() {
            UnpackedNumber::Rational(rational) => match rational.unpack() {
                UnpackedRational::SmallQ { num, den } => num as f64 / den as f64,
                UnpackedRational::BigQ(rational) => rational.to_f64(),
            }
            UnpackedNumber::BigF(bf) => bf.to_f64(),
            UnpackedNumber::SmallF{val, precision:_} => val,
        }
    }
}


enum BigNumber {
    Q(BigQ),
    F(BigF),
}


#[derive(Clone, Copy)]
pub enum NumberDisplayMode {
    Standard
}


impl core::fmt::Display for Rational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_big())
    }
}

impl core::fmt::Debug for Rational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_big())
    }
}

impl core::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.to_big() {
            BigNumber::Q(big_q) => write!(f, "{}", big_q),
            BigNumber::F(big_f) => write!(f, "{}", big_f),
        }
    }
}

impl core::fmt::Debug for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
