
use numbers::Rational;
use ribbit::{Pack, Unpack};

#[derive(ribbit::Pack, Clone, Copy, PartialEq, Eq)]
#[ribbit(size=4, derive(Eq))]
enum MilleniumPrefix {
    P3, P6, P9, P12, P15, P18, P21, P24, P27, P30
}


#[derive(ribbit::Pack, Clone, Copy, PartialEq, Eq)]
#[ribbit(size=4, derive(Eq))]
enum LowPrefix { P1, P2 }


#[derive(ribbit::Pack, Clone, Copy, PartialEq, Eq)]
#[ribbit(size=5, derive(Eq))]
enum ExtendedPrefix {
    #[ribbit(size = 4)]
    Low(crate::units::prefix::LowPrefix),
    #[ribbit(size = 4)]
    Base(crate::units::prefix::MilleniumPrefix)
}



//#[packable(u8)]
#[derive(ribbit::Pack, Clone, Copy, PartialEq, Eq)]
#[ribbit(size=7, derive(Eq))]
enum PrefixMeaning {
    Absent,
    #[ribbit(size = 4)]
    Binary(crate::units::prefix::MilleniumPrefix),
    #[ribbit(size = 5)]
    DecimalBig(crate::units::prefix::ExtendedPrefix),
    #[ribbit(size = 5)]
    DecimalSmall(crate::units::prefix::ExtendedPrefix),
}


/// A decimal or binary prefix of the unit, including the 'absent' one
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Prefix(PrefixMeaningPacked);


impl PrefixMeaning {
    /// Parse a prefix from 1-2 bytes of UTF-8
    pub const fn parse(s: &str) -> Option<Self> {
        Some(match s.as_bytes() {
            b"" => PrefixMeaning::Absent,

            b"da" => PrefixMeaning::DecimalBig(ExtendedPrefix::Low(LowPrefix::P1)),
            b"h" => PrefixMeaning::DecimalBig(ExtendedPrefix::Low(LowPrefix::P2)),
            b"k" => PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P3)),
            b"M" => PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P6)),
            b"G" => PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P9)),
            b"T" => PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P12)),
            b"P" => PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P15)),
            b"E" => PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P18)),
            b"Z" => PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P21)),
            b"Y" => PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P24)),
            b"R" => PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P27)),
            b"Q" => PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P30)),

            b"d" => PrefixMeaning::DecimalSmall(ExtendedPrefix::Low(LowPrefix::P1)),
            b"c" => PrefixMeaning::DecimalSmall(ExtendedPrefix::Low(LowPrefix::P2)),
            b"m" => PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P3)),
            b"u" => PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P6)),
            [0xB5] => PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P6)),  // deprecated micro symbol
            [0xCE, 0xBC] => PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P6)),  // greek mu
            b"n" => PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P9)),
            b"p" => PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P12)),
            b"f" => PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P15)),
            b"a" => PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P18)),
            b"z" => PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P21)),
            b"y" => PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P24)),
            b"r" => PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P27)),
            b"q" => PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P30)),

            b"ki" => PrefixMeaning::Binary(MilleniumPrefix::P3),
            b"Mi" => PrefixMeaning::Binary(MilleniumPrefix::P6),
            b"Gi" => PrefixMeaning::Binary(MilleniumPrefix::P9),
            b"Ti" => PrefixMeaning::Binary(MilleniumPrefix::P12),
            b"Pi" => PrefixMeaning::Binary(MilleniumPrefix::P15),
            b"Ei" => PrefixMeaning::Binary(MilleniumPrefix::P18),
            b"Zi" => PrefixMeaning::Binary(MilleniumPrefix::P21),
            b"Yi" => PrefixMeaning::Binary(MilleniumPrefix::P24),
            b"Ri" => PrefixMeaning::Binary(MilleniumPrefix::P27),
            b"Qi" => PrefixMeaning::Binary(MilleniumPrefix::P30),
            _ => return None,
        })
    }

    /// Get a UTF-8 representation of a prefix
    pub const fn name(self) -> &'static str {
        match self {
            PrefixMeaning::Absent => "",

            PrefixMeaning::DecimalBig(ExtendedPrefix::Low(LowPrefix::P1)) => "da",
            PrefixMeaning::DecimalBig(ExtendedPrefix::Low(LowPrefix::P2)) => "h",
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P3)) => "k",
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P6)) => "M",
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P9)) => "G",
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P12)) => "T",
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P15)) => "P",
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P18)) => "E",
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P21)) => "Z",
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P24)) => "Y",
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P27)) => "R",
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P30)) => "Q",

            PrefixMeaning::DecimalSmall(ExtendedPrefix::Low(LowPrefix::P1)) => "d",
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Low(LowPrefix::P2)) => "c",
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P3)) => "m",
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P6)) => "μ",
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P9)) => "n",
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P12)) => "p",
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P15)) => "f",
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P18)) => "a",
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P21)) => "z",
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P24)) => "y",
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P27)) => "r",
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P30)) => "q",

            PrefixMeaning::Binary(MilleniumPrefix::P3) => "ki",
            PrefixMeaning::Binary(MilleniumPrefix::P6) => "Mi",
            PrefixMeaning::Binary(MilleniumPrefix::P9) => "Gi",
            PrefixMeaning::Binary(MilleniumPrefix::P12) => "Ti",
            PrefixMeaning::Binary(MilleniumPrefix::P15) => "Pi",
            PrefixMeaning::Binary(MilleniumPrefix::P18) => "Ei",
            PrefixMeaning::Binary(MilleniumPrefix::P21) => "Zi",
            PrefixMeaning::Binary(MilleniumPrefix::P24) => "Yi",
            PrefixMeaning::Binary(MilleniumPrefix::P27) => "Ri",
            PrefixMeaning::Binary(MilleniumPrefix::P30) => "Qi",
        }
    }

    pub fn meaning(self) -> Rational {
        match self {
            PrefixMeaning::Absent => Rational::new_small(1, 1),

            PrefixMeaning::DecimalBig(ExtendedPrefix::Low(LowPrefix::P1)) => Rational::new_small(10, 1),
            PrefixMeaning::DecimalBig(ExtendedPrefix::Low(LowPrefix::P2)) => Rational::new_small(100, 1),
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P3)) => Rational::new_small(1000, 1),
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P6)) => Rational::new(1_000_000, 1),
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P9)) => Rational::new(1_000_000_000, 1),
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P12)) => Rational::new(1_000_000_000_000, 1),
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P15)) => Rational::new(1_000_000_000_000_000, 1),
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P18)) => Rational::new(1_000_000_000_000_000_000, 1),
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P21)) => Rational::new_i128(1_000_000_000_000_000_000_000, 1),
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P24)) => Rational::new_i128(1_000_000_000_000_000_000_000_000, 1),
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P27)) => Rational::new_i128(1_000_000_000_000_000_000_000_000_000, 1),
            PrefixMeaning::DecimalBig(ExtendedPrefix::Base(MilleniumPrefix::P30)) => Rational::new_i128(1_000_000_000_000_000_000_000_000_000_000, 1),

            PrefixMeaning::DecimalSmall(ExtendedPrefix::Low(LowPrefix::P1)) => Rational::new_small(1, 10),
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Low(LowPrefix::P2)) => Rational::new_small(1, 100),
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P3)) => Rational::new_small(1, 1000),
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P6)) => Rational::new(1, 1_000_000),
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P9)) => Rational::new(1, 1_000_000_000),
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P12)) => Rational::new(1, 1_000_000_000_000),
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P15)) => Rational::new(1, 1_000_000_000_000_000),
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P18)) => Rational::new(1, 1_000_000_000_000_000_000),
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P21)) => Rational::new_i128(1, 1_000_000_000_000_000_000_000),
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P24)) => Rational::new_i128(1, 1_000_000_000_000_000_000_000_000),
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P27)) => Rational::new_i128(1, 1_000_000_000_000_000_000_000_000_000),
            PrefixMeaning::DecimalSmall(ExtendedPrefix::Base(MilleniumPrefix::P30)) => Rational::new_i128(1, 1_000_000_000_000_000_000_000_000_000_000),

            PrefixMeaning::Binary(MilleniumPrefix::P3) => Rational::new_small(1024, 1),
            PrefixMeaning::Binary(MilleniumPrefix::P6) => Rational::new(1024*1024, 1),
            PrefixMeaning::Binary(MilleniumPrefix::P9) => Rational::new(1024*1024*1024, 1),
            PrefixMeaning::Binary(MilleniumPrefix::P12) => Rational::new(1024*1024*1024*1024, 1),
            PrefixMeaning::Binary(MilleniumPrefix::P15) => Rational::new(1024*1024*1024*1024*1024, 1),
            PrefixMeaning::Binary(MilleniumPrefix::P18) => Rational::new_i128(1024*1024*1024*1024*1024*1024, 1),
            PrefixMeaning::Binary(MilleniumPrefix::P21) => Rational::new_i128(1024*1024*1024*1024*1024*1024*1024, 1),
            PrefixMeaning::Binary(MilleniumPrefix::P24) => Rational::new_i128(1024*1024*1024*1024*1024*1024*1024*1024, 1),
            PrefixMeaning::Binary(MilleniumPrefix::P27) => Rational::new_i128(1024*1024*1024*1024*1024*1024*1024*1024*1024, 1),
            PrefixMeaning::Binary(MilleniumPrefix::P30) => Rational::new_i128(1024*1024*1024*1024*1024*1024*1024*1024*1024*1024, 1),
        }
    }
}


impl Prefix {
    /// No prefix
    pub const fn absent() -> Self {Self(PrefixMeaningPacked::new_absent())}

    /// Prefix from its string representation
    pub fn parse(s: &str) -> Option<Self> {Some(Self(PrefixMeaning::parse(s)?.pack()))}

    /// Get string representation
    pub fn name(self) -> &'static str {self.0.unpack().name()}

    /// The multiplier the prefix represents
    pub fn meaning(self) -> Rational {self.0.unpack().meaning()}
}

impl core::fmt::Debug for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Prefix(\"{}\")", self.meaning())
    }
}


#[test]
fn test_prefixes() {
    for prefix in [
        "",
        "da", "h", "d", "c",
        "k", "M", "G", "T", "P", "E", "Z", "Y", "R", "Q",
        "m", "μ", "n", "p", "f", "a", "z", "y", "r", "q",
        "ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi", "Yi", "Ri", "Qi",
    ] {
        assert_eq!(Prefix::parse(prefix).unwrap().name(), prefix);
    }

    let row_big = ["k", "M", "G", "T", "P", "E", "Z", "Y", "R", "Q"];
    let row_small = ["m", "μ", "n", "p", "f", "a", "z", "y", "r", "q"];
    let row_bin = ["ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi", "Yi", "Ri", "Qi"];

    let mut big = Rational::new(1000, 1);
    let mut small = Rational::new(1, 1000);
    let mut bin = Rational::new(1024, 1);
    for v in 0..row_big.len() {
        let pbig = Prefix::parse(row_big[v]).unwrap().meaning();
        let psmall = Prefix::parse(row_small[v]).unwrap().meaning();
        let pbin = Prefix::parse(row_bin[v]).unwrap().meaning();

        assert_eq!(pbig, big.clone());
        assert_eq!(psmall, small.clone());
        assert_eq!(pbin, bin.clone());

        big = big * Rational::new(1000, 1);
        small = small * Rational::new(1, 1000);
        bin = bin * Rational::new(1024, 1);
    }
}