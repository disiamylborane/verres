//! Language identifiers and localized strings

use std::{collections::HashMap, ops::Deref};

/// An identifier of a language used for localized string selection
#[derive(Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct Language(u16);


impl Language {
    /// Create the language from two- or three-letter code
    ///
    /// Two-letter codes contain two lowercase letters ('en').
    /// Three-letter codes contain two lowercase and one uppercase ('enB').
    pub const fn new(s: &str) -> Option<Self> {
        match *s.as_bytes() {
            [s0, s1] if s0.is_ascii_lowercase() && s1.is_ascii_lowercase() => {
                Some(Self((s0 - b'a') as u16 + (s1 - b'a') as u16 * 26))
            }
            [s0, s1, s2] if s0.is_ascii_lowercase() && s1.is_ascii_lowercase() && s2.is_ascii_uppercase() => {
                Some(Self((s0 - b'a') as u16 + (s1 - b'a') as u16 * 26 + (s2 - b'A' + 1) as u16 * 26 * 26))
            }
            _ => None,
        }
    }

    const fn repr(self) -> (u8, u8, Option<u8>) {
        let sym1 = (self.0 % 26) as u8;
        let rest = self.0 / 26;

        let sym2 = (rest % 26) as u8;
        let sym3 = rest / 26;

        (sym1 + b'a', sym2 + b'a', if 1 <= sym3 && sym3 <= 26 {Some(sym3 as u8 - 1 + b'A')} else {None})
    }
}

impl core::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (s1, s2, sr) = self.repr();
        if let Some(sr) = sr {
            write!(f, "{}{}{}", s1 as char, s2 as char, sr as char)
        } else {
            write!(f, "{}{}", s1 as char, s2 as char)
        }
    }
}
impl core::fmt::Debug for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

/// A piece of settings representing the languages the user accepts
pub struct LanguageResolution {
    /// The preferred language
    pub primary: Language,

    /// The acceptable languages in the decreasing priority order
    pub secondary: Option<Box<[Language]>>,
}


/// Localized refined string
#[derive(Clone)]
pub struct Localized<String: Deref<Target = str>> {
    primary_lang: Language,
    primary_str: String,
    rest: HashMap<Language, String>,
}


impl<String: Deref<Target = str>> Localized<String> {
    /// Create a localized string from one option.
    ///
    /// Use method insert to add more languages
    pub fn new(lang: Language, str: String) -> Self {
        Self {
            primary_lang: lang,
            primary_str: str,
            rest: HashMap::new()
        }
    }

    /// Get a string guided by the language preferences
    pub fn get<'string>(&'string self, lang: &LanguageResolution) -> Unlocalized<'string> {
        if let Some(primary) = self.get_precise(lang.primary) {
            Unlocalized { str: primary, unnative: None }
        } else {
            for &lang in lang.secondary.as_deref().unwrap_or(&[]) {
                if let Some(primary) = self.get_precise(lang) {
                    return Unlocalized { str: primary, unnative: Some(lang) }
                }
            }
            Unlocalized { str: &self.primary_str, unnative: Some(self.primary_lang) }
        }
    }

    /// Get a string strictly from the language, if present
    pub fn get_precise(&self, lang: Language) -> Option<&String> {
        if self.primary_lang == lang {
            Some(&self.primary_str)
        } else {self.rest.get(&lang)}
    }

    /// Add a new localization to the same name
    pub fn insert(&mut self, lang: Language, str: String) {
        if self.primary_lang == lang {
            self.primary_str = str;
        }
        else { self.rest.insert(lang, str); }
    }

    /// Convert into the result of [`Self::insert`] application
    pub fn inserted(mut self, lang: Language, str: String) -> Self {
        self.insert(lang, str);
        self
    }

    /// Iterate over all stored languages and strings
    pub fn iter(&self) -> impl Iterator<Item = (Language, &String)> {
        [(self.primary_lang, &self.primary_str)].into_iter().chain(self.rest.iter().map(|(l, s)| (*l, s)))
    }

    /// The language that will be returned as the worst case of search
    pub fn primary_lang(&self) -> Language { self.primary_lang }

    /// Get internals
    pub fn destruct(self) -> (Language, String, HashMap<Language, String>) {
        let Self { primary_lang, primary_str, rest } = self;
        (primary_lang, primary_str, rest)
    }
}


/// The result got from [`Localized::get`], containing the string and language information
pub struct Unlocalized<'s> {
    /// If the language returned was the primary one in [`LanguageResolution`], contains the language
    pub unnative: Option<Language>,

    /// The string
    pub str: &'s str,
}


#[test]
fn test_languages() {
    assert_eq!(Language::new("en").unwrap().to_string(), "en");
    assert_eq!(Language::new("enU").unwrap().to_string(), "enU");
    assert_eq!(Language::new("enG").unwrap().to_string(), "enG");
    assert_eq!(Language::new("enA").unwrap().to_string(), "enA");
    assert_eq!(Language::new("enZ").unwrap().to_string(), "enZ");
    assert_eq!(Language::new("enx"), None);

    assert_eq!(Language::new("gr").unwrap().to_string(), "gr");
    assert_eq!(Language::new("grA").unwrap().to_string(), "grA");
    assert_eq!(Language::new("grB").unwrap().to_string(), "grB");
    assert_eq!(Language::new("grC").unwrap().to_string(), "grC");
    assert_eq!(Language::new("grx"), None);
}
