use std::ops::Neg;
use std::str::FromStr;

use dashu::{Rational, base::Inverse};


#[derive(Clone)]
#[repr(align(8))]
pub struct BigQ(dashu::Rational);

#[derive(Clone)]
#[repr(align(8))]
pub struct BigF(dashu::float::FBig<dashu::float::round::mode::HalfEven, 2>);


impl From<(i64, u64)> for BigQ {
    fn from(value: (i64, u64)) -> Self {
        Self(dashu::Rational::from_parts(value.0.into(), value.1.into()))
    }
}


impl From<(i128, u128)> for BigQ {
    fn from(value: (i128, u128)) -> Self {
        Self(dashu::Rational::from_parts(value.0.into(), value.1.into()))
    }
}

impl BigQ {
    pub fn to_parts(&self) -> Option<(i64, u64)> {
        let n: i64 = self.0.numerator().try_into().ok()?;
        let d: u64 = self.0.denominator().try_into().ok()?;
        Some((n, d))
    }

    pub fn to_f64(&self) -> f64 {
        self.0.to_f64().value()
    }

    pub fn cmp0(&self) -> std::cmp::Ordering {
        self.0.cmp(&dashu::Rational::ZERO)
    }

    pub fn neg(self) -> Self {
        Self(self.0.neg())
    }

    pub fn inv(self) -> Self {
        Self(self.0.inv())
    }

    pub fn add(&mut self, rhs: &BigQ) {
        self.0 += rhs.0.clone();
    }

    pub fn sub(&mut self, rhs: &BigQ) {
        self.0 -= rhs.0.clone();
    }

    pub fn mul(&mut self, rhs: &BigQ) {
        self.0 *= rhs.0.clone();
    }

    pub fn div(&mut self, rhs: &BigQ) {
        self.0 /= rhs.0.clone();
    }

    pub fn rational_pow(&self, exp: &BigQ, precision: u16) -> ExactOrReal {
        let (exp_num, exp_den) = exp.0.clone().into_parts();

        if let Ok(p) = isize::try_from(&exp_num) {
            if exp_den == UBig::ONE {
                return ExactOrReal::Exact(BigQ(self.0.pow(p)));
            }

            if let Ok(q) = usize::try_from(&exp_den) {
                let (base_num, base_den) = self.0.clone().into_parts();
                let (sign, abs_num) = base_num.into_parts();

                if !(sign == Sign::Negative && q % 2 == 0) {
                    let root_num = abs_num.nth_root(q);
                    let root_den = base_den.nth_root(q);

                    if root_num.pow(q) == abs_num && root_den.pow(q) == base_den {
                        let signed_root_num = IBig::from_parts(sign, root_num);
                        let exact_root = Rational::from_parts(signed_root_num, root_den);
                        return ExactOrReal::Exact(BigQ(exact_root.pow(p)));
                    }
                }
            }
        }

        let base_f = self.0.to_float::<dashu::complex::round::mode::HalfEven, 2>(precision as usize).value();
        let exp_f = exp.0.to_float::<dashu::complex::round::mode::HalfEven, 2>(precision as usize).value();
        ExactOrReal::Approx(BigF(base_f.powf(&exp_f)))
    }


    pub fn rational_log(&self, x: &BigQ, precision: u16) -> ExactOrReal {
        log_rational(&self.0, &x.0, precision as usize)
    }

    pub fn eq(&self, rhs: &BigQ) -> bool {
        self.0 == rhs.0
    }
}

pub enum ExactOrReal {
    Exact(BigQ),
    Approx(BigF),
}

impl core::fmt::Display for BigQ {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match dashu::Decimal::try_from(self.0.clone()) {
            Ok(dec) => write!(f, "{dec}"),
            Err(_) => write!(f, "{}", self.0),
        }
    }
}



impl core::str::FromStr for BigQ {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let x = if s.contains('/') {
            s.parse()
        } else {
            Rational::from_str_decimal(s)
        }.map_err(|_|());
        Ok(BigQ(x?))
    }
}


use dashu::integer::{IBig, Sign, UBig};

impl BigF {
    pub fn from_float_precision(f: f64, p: u8) -> Self {
        Self(dashu::float::FBig::<dashu::complex::round::mode::HalfEven, 2>::try_from(f).unwrap().with_precision(p as usize).value())
    }
    pub fn from_rational_precision(r: BigQ, p: u16) -> Self {
        Self(r.0.to_float(p as usize).value())
    }
    pub fn precision(&self) -> u16 {
        self.0.precision() as u16
    }

    pub fn to_f64(&self) -> f64 {
        self.0.to_f64().value()
    }

    pub fn add(&mut self, rhs: BigF) {
        self.0 += rhs.0;
    }

    pub fn sub(&mut self, rhs: BigF) {
        self.0 -= rhs.0;
    }

    pub fn mul(&mut self, rhs: BigF) {
        self.0 *= rhs.0;
    }

    pub fn div(&mut self, rhs: BigF) {
        self.0 /= rhs.0;
    }

    pub fn eq(&self, rhs: &BigF) -> bool {
        self.0 == rhs.0
    }

    pub fn powf(&mut self, rhs: &BigF) {
        self.0 = self.0.powf(&rhs.0).with_precision(self.0.precision().min(rhs.0.precision())).value();
    }

    pub fn log(&mut self, rhs: &BigF) {
        self.0 = (self.0.ln() / rhs.0.ln()).with_precision(self.0.precision().min(rhs.0.precision())).value();
    }
}

impl core::fmt::Display for BigF {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let prec = self.0.precision();
        let v = self.0.clone().with_rounding::<dashu::float::round::mode::HalfEven>().with_base::<10>().value();
        if prec == 53 {
            write!(f, "{}~", v)
        }
        else {
            write!(f, "{}~{}", v, prec)
        }
    }
}

impl FromStr for BigF {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        let (val_str, prec_str) = s.rsplit_once(['~']).ok_or(())?;

        if val_str.is_empty() || prec_str.starts_with(['+', '-']) {
            return Err(());
        }

        let precision: usize = if prec_str.is_empty() {53} else {prec_str.parse().map_err(|_| ())?};
        if precision == 0 {
            return Err(());
        }

        let parsed = dashu::float::FBig::<dashu::float::round::mode::HalfEven, 10>::from_str(val_str).map_err(|_|())?;

        let sized = parsed.with_base_and_precision::<2>(precision).value();

        Ok(BigF(sized))
    }
}

#[test]
fn test_printing() {
    let zero = BigQ(Rational::ZERO);
    let one = BigQ(Rational::ONE);
    let half = BigQ(Rational::ONE / Rational::from(2));
    let quarter = BigQ(Rational::ONE / Rational::from(4));
    let third = BigQ(Rational::ONE / Rational::from(3));
    let sotp = BigQ(Rational::from(-35) / Rational::from(990) - Rational::from(71) / Rational::from(10));

    assert_eq!("0", zero.to_string());
    assert_eq!("1", one.to_string());
    assert_eq!("0.5", half.to_string());
    assert_eq!("0.25", quarter.to_string());
    assert_eq!("1/3", third.to_string());
    assert_eq!("-3532/495", sotp.to_string());

    assert_eq!(zero.0, BigQ::from_str("0").unwrap().0);
    assert_eq!(one.0, BigQ::from_str("1").unwrap().0);
    assert_eq!(half.0, BigQ::from_str("0.5").unwrap().0);
    assert_eq!(quarter.0, BigQ::from_str("0.25").unwrap().0);
    assert_eq!(third.0, BigQ::from_str("1/3").unwrap().0);
    assert_eq!(sotp.0, BigQ::from_str("-3532/495").unwrap().0);
}

/// Refines a list of positive integers into a pairwise coprime basis such that
/// every input is a product of powers of the basis elements. Uses only gcds.
fn coprime_base(mut items: Vec<UBig>) -> Vec<UBig> {
    use dashu::base::Gcd;
    items.retain(|x| *x > UBig::ONE);
    'restart: loop {
        for i in 0..items.len() {
            for j in (i + 1)..items.len() {
                let g = (&items[i]).gcd(&items[j]);
                if g > UBig::ONE {
                    let a = &items[i] / &g;
                    let b = &items[j] / &g;
                    // j > i, so removing j first keeps index i valid.
                    items.swap_remove(j);
                    items.swap_remove(i);
                    for v in [a, b, g] {
                        if v > UBig::ONE && !items.contains(&v) {
                            items.push(v);
                        }
                    }
                    continue 'restart;
                }
            }
        }
        break;
    }
    items
}

/// Number of times `c` divides `m`, removing those factors from `m`.
fn valuation(m: &UBig, c: &UBig) -> i128 {
    let mut m = m.clone();
    let mut k = 0i128;
    while &m % c == UBig::ZERO {
        m = &m / c;
        k += 1;
    }
    k
}

/// log_base(x) for positive rationals x and base (base != 1).
pub fn log_rational(x: &dashu::Rational, base: &dashu::Rational, precision: usize) -> ExactOrReal {
    use dashu::base::UnsignedAbs;

    let xn = x.numerator().clone().unsigned_abs();
    let xd = x.denominator().clone();
    let bn = base.numerator().clone().unsigned_abs();
    let bd = base.denominator().clone();

    let basis = coprime_base(vec![xn.clone(), xd.clone(), bn.clone(), bd.clone()]);

    // Signed valuations of x and of the base at each basis element.
    let vals: Vec<(i128, i128)> = basis
        .iter()
        .map(|c| {
            let vx = valuation(&xn, c) - valuation(&xd, c);
            let vb = valuation(&bn, c) - valuation(&bd, c);
            (vx, vb)
        })
        .collect();

    // x = base^r exactly iff vx_c = r * vb_c for every basis element c.
    if let Some(k) = vals.iter().position(|&(_, vb)| vb != 0) {
        let (vx0, vb0) = vals[k];
        let proportional = vals.iter().all(|&(vx, vb)| vx * vb0 == vx0 * vb);
        if proportional {
            let r = dashu::Rational::from(dashu::integer::IBig::from(vx0))
                / dashu::Rational::from(dashu::integer::IBig::from(vb0));
            return ExactOrReal::Exact(BigQ(r));
        }
    }

    // Fallback: log_b(x) = ln(x) / ln(b), evaluated at the requested precision.
    let xr = x.to_float(precision).value(); //dashu::Real::from_rational(x, precision);
    let br = base.to_float(precision).value();
    ExactOrReal::Approx(BigF(xr.ln() / br.ln()))
}