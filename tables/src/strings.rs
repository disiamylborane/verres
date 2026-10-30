//! String identifiers, entity names and localization

mod locale;
mod symbol;
mod name;

use std::ops::Deref;

pub use symbol::Symbol;
pub use name::Name;

use nimi_pona::{NimiPona, NimiPonaConvert};
pub use locale::{Language, LanguageResolution, Localized, Unlocalized};


/// Named id, containing a numeric unique ID and a localized name
#[derive(Clone)]
pub struct LocalizedId<Id: NimiPonaConvert, Name: Deref<Target=str>> {
    /// Unique-per-collection numeric ID
    pub id: NimiPona<Id>,

    /// Localized name of entity
    pub name: Localized<Name>,
}
