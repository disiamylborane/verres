use std::str::FromStr;


#[derive(Clone)]
#[repr(align(8))]
pub struct BigQ{n: i128, d: u128}

#[derive(Clone)]
#[repr(align(8))]
pub struct BigF(f64);


impl From<(i64, u64)> for BigQ {
    fn from(value: (i64, u64)) -> Self {
        Self{n: value.0.into(), d: value.1.into()}
    }
}


impl From<(i128, u128)> for BigQ {
    fn from(value: (i128, u128)) -> Self {
        Self{n: value.0, d: value.1}
    }
}


const fn gcd(mut a: u128, mut b: u128) -> u128 {
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


impl BigQ {
    pub fn new(mut n: i128, mut d: u128) -> Self {
        let gcd = gcd(n.unsigned_abs(), d);
        n /= gcd as i128;  // May be converted, gcd is less than n.abs()
        d /= gcd;

        Self{n, d}
    }

    pub fn to_parts(&self) -> Option<(i64, u64)> {
        let n: i64 = self.n.try_into().ok()?;
        let d: u64 = self.d.try_into().ok()?;
        Some((n, d))
    }

    pub fn to_f64(&self) -> f64 {
        self.n as f64 / self.d as f64
    }

    pub fn cmp0(&self) -> std::cmp::Ordering {
        self.n.cmp(&0)
    }

    pub fn neg(self) -> Self {
        Self{n: -self.n, d: self.d}
    }

    pub fn inv(self) -> Self {
        let n = if self.n.signum() < 0 {-(self.d as i128)} else {self.d as i128};
        Self{n, d: self.n.unsigned_abs()}
    }

    pub fn add(&mut self, rhs: &BigQ) {
        let n = self.n * (rhs.d as i128) + rhs.n * (self.d as i128);
        let d = self.d * rhs.d;
        *self = Self{n, d};
    }

    pub fn sub(&mut self, rhs: &BigQ) {
        let n = self.n * (rhs.d as i128) - rhs.n * (self.d as i128);
        let d = self.d * rhs.d;
        *self = Self{n, d};
    }

    pub fn mul(&mut self, rhs: &BigQ) {
        let n = self.n * rhs.n;
        let d = self.d * rhs.d;
        *self = Self{n, d};
    }

    pub fn div(&mut self, rhs: &BigQ) {
        assert!(rhs.n != 0, "division by zero");
        let sign = rhs.n.signum();
        let n = self.n * (rhs.d as i128) * sign;
        let d = self.d * rhs.n.unsigned_abs();
        *self = Self{n, d};
    }

    pub fn rational_pow(&self, exp: &BigQ, _precision: u16) -> ExactOrReal {
        let (exp_num, exp_den) = (exp.n, exp.d);

        if let Ok(p) = i32::try_from(exp_num) {
            if exp_den == 1 {
                if p > 0 {
                    return ExactOrReal::Exact(BigQ::new(self.n.pow(p.unsigned_abs()), self.d.pow(p.unsigned_abs())));
                }
                else if p == 0 {
                    return ExactOrReal::Exact(BigQ::new(1, 1));
                }
                else {
                    let sign = self.n.signum();
                    return ExactOrReal::Exact(BigQ::new((self.d.pow(p.unsigned_abs()) as i128) * sign, self.n.unsigned_abs().pow(p.unsigned_abs())));
                }
            }

            if let Ok(q) = u32::try_from(exp_den) {
                let (base_num, base_den) = (self.n, self.d);

                if !(base_num < 0 && q % 2 == 0) {
                    let abs_num = base_num.unsigned_abs();
                    let root_num = nth_root(abs_num, q).0;
                    let root_den = nth_root(base_den, q).0;

                    if root_num.pow(q) == abs_num && root_den.pow(q) == base_den {
                        if p > 0 {
                            return ExactOrReal::Exact(BigQ {
                                n: root_num.pow(p.unsigned_abs()) as i128 * (if base_num > 0 {1} else {-1}),
                                d: root_den.pow(p.unsigned_abs())
                            });
                        }
                        else if p == 0 {
                            return ExactOrReal::Exact(BigQ::new(1, 1));
                        }
                        else {
                            return ExactOrReal::Exact(BigQ {
                                n: root_den.pow(p.unsigned_abs()) as i128 * (if base_num > 0 {1} else {-1}),
                                d: root_num.pow(p.unsigned_abs())
                            });
                        }
                    }
                }
            }
        }

        let base_f = self.to_f64();
        let exp_f = exp.to_f64();
        ExactOrReal::Approx(BigF(base_f.powf(exp_f)))
    }


    pub fn rational_log(&self, x: &BigQ, precision: u16) -> ExactOrReal {
        log_rational(self, x, precision as usize)
    }

    pub fn eq(&self, rhs: &BigQ) -> bool {
        self.n == rhs.n && self.d == rhs.d
    }
}


pub fn nth_root(x: u128, n: u32) -> (u128, u128) {
    assert!(n > 0, "root degree n must be greater than 0");
    if x <= 1 || n == 1 {
        return (x, 0);
    }
    if n >= 128 {
        // Since 2^128 overflows u128, for x >= 2 the nth root is always 1
        return (1, x - 1);
    }

    // Tight upper bound: 2^ceil(bits / n)
    let bits = 128 - x.leading_zeros();
    let mut low = 1u128 << ((bits - 1) / n);
    let mut high = 1u128 << bits.div_ceil(n);

    while low < high {
        let mid = low + (high - low).div_ceil(2);
        match mid.checked_pow(n) {
            Some(p) if p <= x => low = mid,
            _ => high = mid - 1,
        }
    }

    (low, x - low.pow(n))
}


pub enum ExactOrReal {
    Exact(BigQ),
    Approx(BigF),
}

impl core::fmt::Display for BigQ {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.d == 1 {
            return write!(f, "{}", self.n);
        }

        // Factor out all 2s and 5s from d
        let twos = self.d.trailing_zeros();
        let mut d = self.d >> twos;
        let mut fives = 0u32;
        while d % 5 == 0 {
            d /= 5;
            fives += 1;
        }

        // Terminating decimal iff no other prime factors remain
        if d == 1 {
            let places = twos.max(fives) as usize;
            let sign = if self.n < 0 { "-" } else { "" };
            let abs_num = self.n.unsigned_abs();
            let int_part = abs_num / self.d;
            let mut rem = abs_num % self.d;

            write!(f, "{sign}{int_part}.")?;
            for _ in 0..places {
                rem *= 10;
                write!(f, "{}", rem / self.d)?;
                rem %= self.d;
            }
            Ok(())
        } else {
            write!(f, "{}/{}", self.n, self.d)
        }
    }
}



impl core::str::FromStr for BigQ {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // 1. Fraction format: "a/b"
        if let Some((num_str, den_str)) = s.split_once('/') {
            let n: i128 = num_str.trim().parse().map_err(|_| ())?;
            let d: u128 = den_str.trim().parse().map_err(|_| ())?;
            if d == 0 {
                return Err(());
            }
            return Ok(Self::new(n, d));
        }

        // 2. Decimal format: "abcd.efghi"
        if let Some((int_str, frac_str)) = s.split_once('.') {
            if int_str.is_empty() && frac_str.is_empty() {
                return Err(());
            }
            if !frac_str.bytes().all(|b| b.is_ascii_digit()) {
                return Err(());
            }

            let is_neg = int_str.starts_with('-');
            let digits = format!("{int_str}{frac_str}");
            let mut n: i128 = match digits.as_str() {
                "" | "-" | "+" => 0,
                d => d.parse().map_err(|_| ())?,
            };
            if is_neg && n > 0 {
                n = -n; // handles "-0.5" where int_str is "-0"
            }

            let d = 10u128
                .checked_pow(frac_str.len() as u32)
                .ok_or(())?;
            return Ok(Self::new(n, d));
        }

        // 3. Plain integer: "123"
        let n: i128 = s.parse().map_err(|_| ())?;
        Ok(Self::new(n, 1))
    }
}


impl BigF {
    pub fn from_float_precision(f: f64, _p: u8) -> Self {
        Self(f)
    }
    pub fn from_rational_precision(r: BigQ, _p: u16) -> Self {
        Self(r.to_f64())
    }
    pub fn precision(&self) -> u16 {
        53
    }

    pub fn to_f64(&self) -> f64 {
        self.0
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
        self.0 = self.0.powf(rhs.0);
    }

    pub fn log(&mut self, rhs: &BigF) {
        self.0 = self.0.ln() / rhs.0.ln();
    }
}

impl core::fmt::Display for BigF {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}~", self.0)
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

        let sized = f64::from_str(val_str).map_err(|_|())?;

        Ok(BigF(sized))
    }
}

/// Refines a list of positive integers into a pairwise coprime basis such that
/// every input is a product of powers of the basis elements. Uses only gcds.
fn coprime_base(mut items: Vec<u128>) -> Vec<u128> {
    items.retain(|x| *x > 1);
    'restart: loop {
        for i in 0..items.len() {
            for j in (i + 1)..items.len() {
                let g = gcd(items[i], items[j]);
                if g > 1 {
                    let a = items[i] / g;
                    let b = items[j] / g;
                    // j > i, so removing j first keeps index i valid.
                    items.swap_remove(j);
                    items.swap_remove(i);
                    for v in [a, b, g] {
                        if v > 1 && !items.contains(&v) {
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
fn valuation(m: &u128, c: &u128) -> u128 {
    let mut m = *m;
    let mut k = 0;
    while m.is_multiple_of(*c) {
        m /= c;
        k += 1;
    }
    k
}

/// log_base(x) for positive rationals x and base (base != 1).
pub fn log_rational(x: &BigQ, base: &BigQ, _precision: usize) -> ExactOrReal {
    let xn = x.n.unsigned_abs();
    let xd = x.d;
    let bn = base.n.unsigned_abs();
    let bd = base.d;

    let basis = coprime_base(vec![xn, xd, bn, bd]);

    // Signed valuations of x and of the base at each basis element.
    let vals: Vec<(i128, u128)> = basis
        .iter()
        .map(|c| {
            let vx = (valuation(&xn, c) - valuation(&xd, c)) as i128;
            let vb = valuation(&bn, c) - valuation(&bd, c);
            (vx, vb)
        })
        .collect();

    // x = base^r exactly iff vx_c = r * vb_c for every basis element c.
    if let Some(k) = vals.iter().position(|&(_, vb)| vb != 0) {
        let (vx0, vb0) = vals[k];
        let proportional = vals.iter().all(|&(vx, vb)| vx * vb0 as i128 == vx0 * vb as i128);
        if proportional {
            return ExactOrReal::Exact(BigQ{n: vx0, d: vb0});
        }
    }

    // Fallback: log_b(x) = ln(x) / ln(b), evaluated at the requested precision.
    let xr = x.to_f64(); //Real::from_rational(x, precision);
    let br = base.to_f64();
    ExactOrReal::Approx(BigF(xr.ln() / br.ln()))
}

