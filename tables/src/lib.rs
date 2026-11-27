//! Tables - the experiment storage part of `verres`

#![warn(missing_docs)]

use crate::units::UnitSystem;
use crate::objects::ObjectGraph;


macro_rules! printer {
    (
        $(#[$attr:meta])*
        $funcname:ident($thisname:pat = &$Self:ty, $contextname:ident: &$contextty:ty, $fmtname:ident)
            $code:block
    ) => {
        $(#[$attr])*
        pub fn $funcname<'repr>(&'repr self, $contextname: &'repr $contextty) -> impl core::fmt::Display + 'repr {
            struct Repr<'repr>(&'repr $Self, &'repr $contextty);

            impl core::fmt::Display for Repr<'_> {
                fn fmt(&self, $fmtname: &mut core::fmt::Formatter) -> core::fmt::Result {
                    let Self($thisname, $contextname) = *self;
                    $code
                }
            }

            Repr(self, $contextname)
        }
    }
}


pub mod strings;
pub mod units;
pub mod objects;


/// Experimental data storage
pub struct Tables {
    /// Global unit system used in a type system for this engine
    pub units: UnitSystem,
    /// A directed acyclic graph of real world objects having non-formalizable but well known definitions
    pub objects: ObjectGraph,
}
